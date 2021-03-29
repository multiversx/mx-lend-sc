#![no_std]

use core::{array::FixedSizeArray, char::REPLACEMENT_CHARACTER, time};

use elrond_wasm::{HexCallDataSerializer, only_owner, require, sc_error};

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

const ESDT_ISSUE_COST: u64 = 5000000000000000000;

const LEND_TOKEN_PREFIX: &[u8] = b"L";
const BORROW_TOKEN_PREFIX: &[u8] = b"B";
const LEND_TOKEN_NAME: &[u8] = b"IntBearing";
const DEBT_TOKEN_NAME: &[u8] = b"DebtBearing";

const EMPTY_TOKEN_ID: &[u8] = b"EGLD";


#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub struct IssueData<BigUint: BigUintApi> {
    pub name: BoxedBytes,
    pub ticker: TokenIdentifier,
    pub existing_token: TokenIdentifier
}

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub struct DebtPosition<BigUint: BigUintApi> {
    pub size: BigUint,
    pub health_factor: u32,
    pub is_liquidated: bool,
    pub collateral_amount: BigUint,
    pub collateral_identifier: TokenIdentifier
}

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct InterestMetadata<BigUint: BigUintApi> {
	pub timestamp: u64
}

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct DebtMetadata<BigUint: BigUintApi> {
    pub timestamp: u64,
    pub collateral_amount: BigUint,
    pub collateral_identifier: TokenIdentifier,
    pub colletareal_timestamp: u64
}

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct RepayPostion<BigUint: BigUintApi> {
    pub identifier: TokenIdentifier,
    pub amount: BigUint,
    pub nonce: u64,
    pub collateral_identifier: TokenIdentifier,
    pub collateral_amount: BigUint,
    pub collateral_timestamp: u64
}

#[elrond_wasm_derive::contract(LiquidityPoolImpl)]
pub trait LiquidityPool {
    
    #[init]
    fn init(&self, asset:TokenIdentifier, lending_pool: Address) {
        self.pool_asset().set(&asset);
        self.set_lending_pool(lending_pool);
        self.debt_nonce().set(1u64);
    }

    #[payable("*")]
    #[endpoint]
    fn deposit_asset(
        &self,
        initial_caller: Address,
        #[payment_token] asset: TokenIdentifier, 
        #[payment] amount: BigUint
    ) -> SCResult<()> {
        require!(amount > 0, "payment must be greater than 0");
        require!(!initial_caller.is_zero(), "invalid address provided");

        let pool_asset = self.pool_asset().get();
        require!(
            asset.clone() == pool_asset.clone(), 
            "asset not supported for this liquidity pool"
        );

        let interest_metadata = InterestMetadata{
            timestamp: self.get_block_timestamp()
        };

        self.mint_interest(amount, interest_metadata);

        let lend_token = self.lend_token().get();
        let nonce = self.get_current_esdt_nft_nonce(
            &self.get_sc_address(), 
            lend_token.as_esdt_identifier()
        );
        
        self.send().direct_esdt_nft_via_transfer_exec(
            &initial_caller, 
            &lend_token.as_esdt_identifier(), 
            &nonce, 
            amount,
            &[]
        );
        
        let mut asset_reserve = self.reserves().get(&pool_asset.clone()).unwrap_or(BigUint::zero());
        asset_reserve += amount.clone();
        
        self.reserves().insert(pool_asset, asset_reserve);

        Ok(())
    }

    #[endpoint]
    fn borrow(
        &self,
        initial_caller : Address,
        lend_token: TokenIdentifier, 
        amount: BigUint,
        timstamp: u64
    ) -> SCResult<()> {

        require!(self.get_caller() == self.get_lending_pool(), "can only be called through lending pool");

        require!(amount > 0, "lend amount must be bigger then 0");
        
        require!(!initial_caller.is_zero(), "invalid address provided");

        let borrows_token = self.get_borrow_token();
        let asset = self.get_pool_asset();

        let mut borrows_reserve = self.reserves().get(&borrows_token.clone()).unwrap_or(BigUint::zero());
        let mut asset_reserve = self.reserves().get(&asset.clone()).unwrap_or(BigUint::zero());
                
        require!(asset_reserve != BigUint::zero(), "asset reserve is empty");

        // TODO: serialize token data + nonce and hash & extract to separate func 
        let mut debt_nonce = self.debt_nonce().get();
        let position_id = self.keccak256(Vec<u8>::from(debt_nonce.clone()));
        self.increment_debt_nonce(debt_nonce);
        
        let debt_metadata = DebtMetadata{
            timestamp: self.get_block_timestamp(),
            collateral_amount: amount,
            collateral_identifier: lend_token,
            colletareal_timestamp: timestamp
        };
        self.mint_debt(amount, debt_metadata, position_id);
        
        let nonce = self.get_current_esdt_nft_nonce(
            &self.get_sc_address(),
            borrows_token.as_esdt_identifier(),

        );

        // send debt position tokens

        self.send().direct_esdt_nft_via_transfer_exec(
            &initial_caller, 
            &borrows_token.as_esdt_identifier(),
            &nonce, 
            &amount, 
            &[]
        );
        
        // send collateral requested to the user
        
        self.send().direct(&initial_caller, &asset, &amount, &[]);

        borrows_reserve += amount.clone();
        asset_reserve -= amount.clone();

        let mut total_borrow = self.get_total_borrow();
        total_borrow += amount.clone();
        self.set_total_borrow(total_borrow);

        self.reserves().insert(borrows_token, borrows_reserve);
        self.reserves().insert(asset, asset_reserve);

        let current_health = self.compute_health_factor();
        let debt_position = DebtPosition{
            size: amount,
            health_factor: current_health,
            is_liquidated: false,
            collateral_amount: amount,
            collateral_identifier: lend_token
        };
        self.debt_positions().insert(position_id, debt_position);

        Ok(())
    }

    #[payable("*")]
    #[endpoint(lockBTokens)]
    fn lock_b_tokens(
        &self,
        initial_caller: Address,
        #[payment_token] borrow_token: TokenIdentifier,
        #[payment] amount: BigUint
    ) -> SCResult<H256> {
        require!(self.get_caller() == self.get_lending_pool(), "can only be called by lending pool");
        require!(amount>0 "amount must be greater then 0");
        require!(!initial_caller.is_zero(),"invalid address");

        require!(borrow_token == self.get_borrow_token(), "borrow token not supported by this pool");

        let nft_nonce = self.call_value().esdt_token_nonce();

        let esdt_nft_data = self.get_esdt_token_data(
            &self.get_sc_address(),
            borrow_token.as_esdt_identifier(),
            nft_nonce
        );

        let metadata: DebtMetadata::<BigUint>;
        match DebtMetadata::<BigUint>::top_decode(esdt_nft_data.attributes.clone().as_slice()) {
			Result::Ok(decoded) => {
				metadata = decoded;
			}
			Result::Err(_) => {
				return sc_error!("could not parse token metadata");
			}
		}
        let data = [borrow_token.as_esdt_identifier().as_slice(), amount ,nft_nonce.as_ne_bytes().as_slice()].concat();
        let unique_repay_id = self.keccak256(data);
        let repay_position = RepayPostion{
            borrow_token.as_esdt_identifier(),
            amount,
            nft_nonce,
            metadata.collateral_identifier,
            metadata.collateral_amount,
            metadata.collateral_timestamp
        };
        self.repay_position().insert(unique_repay_id, repay_position);

        Ok(unique_repay_id)
    }

    #[payable("*")]
    #[endpoint(repay)]
    fn repay(
        &self,
        intitial_caller: Address,
        unique_id: H256,
        #[payment_token] asset: TokendIdentifier,
        #[payment] amount:BigUint
    ) -> SCResult<RepayPostion<BigUint>> {
        require!(self.get_caller() == self.get_lending_pool());
        require!(amount >0, "amount be greater then 0");
        require!(asset == self.get_pool_asset(), "asset is not supported by this pool");

        require!(self.repay_position().contains_key(unique_id.clone()), "there are no locked borrowed token for this id, lock b tokens first");
        let mut repay_position = self.repay_position().get(&unique_id.clone()).unwrap_or(BigUint::zero());

        require!(repay_position.amount >= amount,"b tokens amount locked must be equal with the amount of asset token send");

        self.burn(amount, repay_position.nonce, repay_position.identifier);

        if repay_position.amount == amount {
            self.repay_position().remove(&unique_id.clone());
        } else if repay_position.amount > amount {
            repay_position.amount -= amount;
            self.repay_position().insert(unique_id, repay_position);
        }

        let mut result = RepayPostion{
             identifier: repay_position.identifier,
             amount: amount,
             nonce: repay_position.nonce,
             collateral_identifier: repay_position.collateral_identifier,
             collateral_amount: repay_position.collateral_amount,
             collateral_timestamp: repay_position.collateral_timestamp
        };       

        Ok(result)
    }

    #[payable("*")]
    #[endpoint(mintLTokens)]
    fn mint_l_tokens(
        &self,
        initial_caller: Address,
        lend_token: TokenIdentifier,
        amount: BigUint,
        interest_timestamp: u64
    ) -> SCResult<()> {
        require!(self.get_caller() == self.get_lending_pool(), "can only by called by lending pool");

        require!(lend_token == self.get_lend_token(), "asset is not supported by this pool");
        require!(amount >0, amount must be greater then 0);
        require!(!initial_caller.is_zero(), "invalid address");

        let interest_metadata = InterestMetadata{
            timestamp: interest_timestamp
        };

        self.mint_interest(amount, interest_metadata);

        let nonce = self.get_current_esdt_nft_nonce(
            &self.get_sc_address(),
            borrows_token.as_esdt_identifier()
        );

        self.send().direct_esdt_nft_via_transfer_exec(
            &initial_caller, 
            &lend_token.as_esdt_identifier(),
            &nonce, 
            &amount, 
            &[]
        );

        Ok(())
    }

    #[payable("*")]
    #[endpoint(addCollateral)]
    fn add_collateral(
        &self,
        #[payment_token] lend_token: TokenIdentifier,
        #[payment] amount: BigUint
    ) -> SCResult<()> {
        // TODO: check if this is not a duplicate impl of deposit_asset ???
        require!(self.get_lending_pool() == self.get_caller(), "can only be called by lending pool");
        require!(amount > 0, "amount must be bigger then 0");
        require!(lend_token == self.get_lend_token(), "lend token is not supported by this pool");
        
        let mut lend_reserve = self.reserves().get(&lend_token.clone()).unwrap_or(BigUint::zero());

        lend_reserve += amount.clone();

        let mut total_collateral = self.get_total_collateral();
        total_collateral += amount.clone();
        self.set_total_collateral(amount);
        
        self.reserves().insert(lend_token, lend_reserve);

        Ok(())
    }

    #[payable("*")]
    #[endpoint]
    fn withdraw(
        &self,
        initial_caller: Address,
        #[payment_token] lend_token: TokenIdentifier,
        #[payment] amount: BigUint
    ) -> SCResult<()>{
        require!(self.get_caller() == self.get_lending_pool(), "can only be called by lending pool");
        require!(amount >0 "amount must be bigger then 0");
        require!(lend_token == self.get_lend_token(), "lend token is not supported by this pool");
        require!(!initial_caller.is_zero(), "invalid address");

        let pool_asset = self.get_pool_asset();

        let mut lend_reserve = self.reserves().get(&lend_token.clone()).unwrap_or(BigUint::zero());
        let mut asset_reserve = self.reserves().get(&pool_asset.clone()).unwrap_or(BigUint::zero());

        require!(asset_reserve != BigUint::zero(), "asset reserve is empty");

        self.send().direct(&initial_caller, &pool_asset, &amount, &[]);

        lend_reserve += amount;
        asset_reserve -= amount;

        self.reserves().insert(lend_token, lend_reserve);
        self.reserves().insert(pool_asset, asset_reserve);
        
        Ok(())
    }
    

    #[payable("EGLD")]
    #[endpoint]
    fn issue(
        &self,
        token_ticker: TokenIdentifier,
        token_prefix: BoxedBytes,
        #[payment] issue_cost: BigUint
    ) -> AsyncCall<BigUint> {

        only_owner!(self, "only owner can issue new tokens");
        require!(
            issue_cost == BigUint::from(ESDT_ISSUE_COST),
            "payment should be exactly 5 EGLD"
        );
        let pool_asset = self.pool_asset().get();
        require!(
            token_ticker.clone() == pool_asset.clone(),
            "wrong ESDT asset identifier"
        );
        
        let issue_data = self.prepare_issue_data(token_prefix, token_ticker);
        
        require!(
            issue_data.name != BoxedBytes::zeros(0), 
            "invalid input. could not prepare issue data"
        );
        require!(
            issue_data.ticker == EMPTY_TOKEN_ID,
            "token already issued for this identifier"
        );

        ESDTSystemSmartContractProxy::new()
            .issue_non_fungible(
                issue_cost,
                &issue_data.name,
                &issue_data.ticker,
                NonFungibleTokenProperties {
                    can_freeze: true,
                    can_wipe: true,
                    can_pause: true,
                    can_change_owner: true,
                    can_upgrade: true,
                    can_add_special_roles: true,
                },
            )
            .async_call()
            .with_callback(self.callbacks().issue_callback(&token_prefix))
    }

    #[callback]
    fn issue_callback(
        &self, 
        prefix: &BoxedBytes,
		#[payment_token] ticker: TokenIdentifier,
		#[payment] amount: BigUint,
		#[call_result] result: AsyncCallResult<()>
    ) {
        match result {
            AsyncCallResult::Ok(()) => {
                if prefix == BoxedBytes::from(LEND_TOKEN_PREFIX) {
                    self.lend_token().set(&ticker);
                    self.send_callback_result(ticker, b"setLendTokenAddress");
                } else {
                    self.borrow_token().set(&ticker);
                    self.send_callback_result(ticker, b"setBorrowTokenAddress");
                }
            },
            AsyncCallResult::Err(message) => {
                let caller = self.get_owner_address();
                if ticker.is_egld() && amount > 0 {
                    self.send().direct_egld(&caller, &amount, &[]);
                }
                self.last_error().set(&message.err_msg);
            }
        }
    }

    fn mint_interest(&self, amount: BigUint, metadata: InterestMetadata<BigUint>) {
        self.send().esdt_nft_create::<InterestMetadata<BigUint>>(
            self.get_gas_left(),
            self.lend_token().get().as_esdt_identifier(),
            &amount,
            &BoxedBytes::empty(),
            &BigUint::zero(),
            &H256::zero(),
            &metadata,
            &[]
        )
    }

    fn mint_debt(&self, amount: BigUint, metadata: DebtMetadata<BigUint>, position_id: H256) {
        self.send().esdt_nft_create::<PositionMetadata<>>(
			self.get_gas_left(),
			ticker.as_esdt_identifier(),
			&amount,
			&BoxedBytes::empty(),
			&BigUint::zero(),
			position_id,
			&metadata,
			&[],
		);
    }

    fn burn(&self, amount: BigUint, nonce: u64, ticker: TokenIdentifier) {
		self.send().esdt_nft_burn(
			self.get_gas_left(),
			ticker.as_esdt_identifier(),
			nonce,
			&amount,
		);
    }

    fn send_callback_result(&self, token: TokenIdentifier, endpoint: &[u8]) -> Vec<BoxedBytes> {
        let owner = self.get_owner_address();

        let mut args = ArgBuffer::new();
        args.push_argument_bytes(token.as_esdt_identifier());

        self.send().execute_on_dest_context_raw(
            self.get_gas_left(),
            &owner,
            &BigUint::zero(),
            endpoint,
            &args
        );
    }

    fn prepare_issue_data(
        &self,
        prefix: BoxedBytes,
        ticker: TokenIdentifier
    ) -> IssueData<BigUint> {

        let prefixed_ticker = [prefix.as_slice(), ticker.as_esdt_identifier()].concat();
        let mut issue_data = IssueData{
            name: BoxedBytes::zeros(0),
            ticker: TokenIdentifier::from(BoxedBytes::from(prefixed_ticker)),
            existing_token: TokenIdentifier::from(BoxedBytes::zeros(0))
        };
        
        if prefix == BoxedBytes::from(LEND_TOKEN_PREFIX) {
            issue_data.name = [LEND_TOKEN_NAME, ticker.as_name()].concat();
            issue_data.existing_token = self.lend_token().get();
        } else if prefix == BoxedBytes::from(BORROW_TOKEN_PREFIX) {
            issue_data.name = [BORROW_TOKEN_PREFIX, ticker.as_name()].concat();
            issue_data.existing_token = self.borrow_token().get();
        }

        return issue_data;
    }

    /// VIEWS

    #[view(getReserve)]
    fn get_reserve(&self, token: TokenIdentifier) -> BigUint {
        self.reserves().get(&token).unwrap_or(BigUint::zero())
    }

    #[view(poolAsset)]
    fn get_pool_asset(&self) -> TokenIdentifier {
        self.pool_asset().get()
    }

    #[view(lendToken)]
    fn get_lend_token(&self) -> TokenIdentifier {
        self.lend_token().get()
    }

    #[view(borrowToken)]
    fn get_borrow_token(&self) -> TokenIdentifier {
        self.borrow_token().get()
    }

    fn increment_debt_nonce(&self, current: u64) {
        self.debt_nonce().set(&u64::from(current + 1));
    }

    //
    /// pool asset

    #[storage_mapper("pool_asset")]
    fn pool_asset(&self) -> SingleValueMapper<Self::Storage, TokenIdentifier>;

    //
    /// lend token supported for asset
    
    #[storage_mapper("lend_token")]
    fn lend_token(&self) -> SingleValueMapper<Self::Storage, TokenIdentifier>;

    //
    /// borrow token supported for collateral

    #[storage_mapper("borrow_token")]
    fn borrow_token(&self) -> SingleValueMapper<Self::Storage, TokenIdentifier>;
    
    //
    /// pool reserves

    #[storage_mapper("reserves")]
    fn reserves(&self) -> MapMapper<Self::Storage, TokenIdentifier, BigUint>;

    //
    /// last error
    #[storage_mapper("last_error")]
    fn last_error(&self) -> SingleValueMapper<Self::Storage, BoxedBytes>;

    //
    /// debt positions
    #[storage_mapper("debt_positions")]
    fn debt_positions(&self) -> MapMapper<Self::Storage, H256, DebtPosition<BigUint>>;

    //
    /// debt nonce
    #[storage_mapper("debt_nonce")]
    fn debt_nonce(&self) -> SingleValueMapper<Self::Storage, u64>;

    //
    // repay position
    #[storage_mapper("repay_position")]
    fn repay_position(&self) -> MapMapper<Self::Storage, H256, RepayPostion<BigUint>>;

    //
    /// lending pool address 
    #[storage_set("lendingPool")]
    fn set_lending_pool(&self, lending_pool:Address);
    
    #[view(getLendingPool)]
    #[storage_get("lendingPool")]
    fn get_lending_pool(&self) -> Address;

    //
    // total borrowing from pool

    #[storage_set("totalBorrow")]
    fn set_total_borrow(&self, total: BigUint);

    #[view(totalBorrow)]
    #[storage_get("totalBorrow")]
    fn get_total_borrow(&self) -> BigUint;

    //
    // total collateral from pool
    #[storage_set("totalCollateral")]
    fn set_total_collateral(&self, amount:BigUint);

    #[view(totalCollateral)]
    #[storage_get("totalCollateral")]
    fn get_total_collateral(&self) -> BigUint;
}
