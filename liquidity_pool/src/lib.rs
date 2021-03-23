#![no_std]

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

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct PositionMetadata<BigUint: BigUintApi> {
	timestamp: u64
}

#[elrond_wasm_derive::contract(LiquidityPoolImpl)]
pub trait LiquidityPool {
    
    #[init]
    fn init(&self, asset:TokenIdentifier, lending_pool: Address) {
        self.pool_asset().set(&asset);
        self.set_lending_pool(lending_pool); 
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

        let lend_token = self.lend_token().get();

        let mut lend_reserve = self.reserves().get(&lend_token.clone()).unwrap_or(BigUint::zero());
        let mut asset_reserve = self.reserves().get(&pool_asset.clone()).unwrap_or(BigUint::zero());
        
        require!(lend_reserve != BigUint::zero(), "lend reserve empty");

        if !(lend_reserve.clone() > amount.clone()) {
            // mint more lend tokens
        }
        
        self.send().direct(&initial_caller, &lend_token, &amount, &[]);

        lend_reserve -= amount.clone();
        asset_reserve += amount.clone();

        self.reserves().insert(lend_token, lend_reserve);
        self.reserves().insert(pool_asset, asset_reserve);

        Ok(())
    }

    #[endpoint]
    fn borrow(
        &self,
        initial_caller : Address, 
        amount: BigUint
    ) -> SCResult<()> {

        require!(self.get_caller() == self.get_lending_pool(), "can only be called through lending pool");

        require!(amount > 0, "lend amount must be bigger then 0");
        
        require!(!initial_caller.is_zero(), "invalid address provided");

        let borrows_token = self.get_borrow_token();
        let asset = self.get_pool_asset();

        let mut borrows_reserve = self.reserves().get(&borrows_token.clone()).unwrap_or(BigUint::zero());
        let mut asset_reserve = self.reserves().get(&asset.clone()).unwrap_or(BigUint::zero());
        
        
        require!(borrows_reserve != BigUint::zero(), "borrow reserve is empty");
        require!(asset_reserve != BigUint::zero(), "asset reserve is empty");

        self.send().direct(&initial_caller, &borrows_token, &amount, &[]);
        self.send().direct(&initial_caller, &asset, &amount, &[]);

        borrows_reserve -= amount.clone();
        asset_reserve -= amount.clone();

        let mut total_borrow = self.get_total_borrow();
        total_borrow += amount.clone();
        self.set_total_borrow(total_borrow);

        self.reserves().insert(borrows_token, borrows_reserve);
        self.reserves().insert(asset, asset_reserve);

        Ok(())
    }

    #[payable]
    #[endpoint(addCollateral)]
    fn add_collateral(
        &self,
        #[payment_token] lend_token: TokenIdentifier,
        #[payment] amount: BigUint
    ) -> SCResult<()> {

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
            issue_data.name != Default::default(), 
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

    fn mint(&self, amount: BigUint, metadata: PositionMetadata<BigUint>, ticker: TokenIdentifier) {
        self.send().esdt_nft_create::<PositionMetadata<>>(
			self.get_gas_left(),
			ticker.as_esdt_identifier(),
			&amount,
			&BoxedBytes::empty(),
			&BigUint::zero(),
			&H256::zero(),
			&metadata,
			&[uri],
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
            
            issue_data;
        } else if prefix == BoxedBytes::from(BORROW_TOKEN_PREFIX) {
            issue_data.name = [BORROW_TOKEN_PREFIX, ticker.as_name()].concat();
            issue_data.existing_token = self.borrow_token().get();

            issue_data;
        }

        Default::default();
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

    /// pool asset

    #[storage_mapper("pool_asset")]
    fn pool_asset(&self) -> SingleValueMapper<Self::Storage, TokenIdentifier>;

    ///
    /// lend token supported for asset
    
    #[storage_mapper("lend_token")]
    fn lend_token(&self) -> SingleValueMapper<Self::Storage, TokenIdentifier>;

    ///  
    /// borrow token supported for collateral

    #[storage_mapper("borrow_token")]
    fn borrow_token(&self) -> SingleValueMapper<Self::Storage, TokenIdentifier>;
    
    ///
    /// pool reserves

    #[storage_mapper("reserves")]
    fn reserves(&self) -> MapMapper<Self::Storage, TokenIdentifier, BigUint>;

    ///
    /// last error
    #[storage_mapper("last_error")]
    fn last_error(&self) -> SingleValueMapper<Self::Storage, BoxedBytes>;


    #[storage_set("lendingPool")]
    fn set_lending_pool(&self, lending_pool:Address);
    
    #[view(getLendingPool)]
    #[storage_get("lendingPool")]
    fn get_lending_pool(&self) -> Address;

    //

    //total borrowing from pool

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

    //

    /// temporary token prefix for ESDT operation

    #[view(latestPrefix)]
    #[storage_mapper("latest_prefix")]
    fn latest_prefix(&self) -> SingleValueMapper<Self::Storage, BoxedBytes>;
}
