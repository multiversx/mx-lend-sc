#![no_std]

pub mod library;
pub use library::*;

pub mod models;
pub use models::*;

use elrond_wasm::{only_owner, require, sc_error};

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

const ESDT_ISSUE_COST: u64 = 5000000000000000000;

const LEND_TOKEN_PREFIX: &[u8] = b"L";
const BORROW_TOKEN_PREFIX: &[u8] = b"B";
const LEND_TOKEN_NAME: &[u8] = b"IntBearing";
const DEBT_TOKEN_NAME: &[u8] = b"DebtBearing";

#[elrond_wasm_derive::contract(LiquidityPoolImpl)]
pub trait LiquidityPool {
    #[module(LibraryModuleImpl)]
    fn library_module(&self) -> LibraryModuleImpl<T, BigInt, BigUint>;

    #[init]
    fn init(
        &self,
        asset: TokenIdentifier,
        lending_pool: Address,
        r_base: BigUint,
        r_slope1: BigUint,
        r_slope2: BigUint,
        u_optimal: BigUint,
        reserve_factor: BigUint,
    ) {
        self.library_module().init();
        self.pool_asset().set(&asset);
        self.set_lending_pool(lending_pool);
        self.debt_nonce().set(&1u64);
        self.reserve_data().set(&ReserveData {
            r_base,
            r_slope1,
            r_slope2,
            u_optimal,
            reserve_factor,
        });
    }

    #[payable("*")]
    #[endpoint]
    fn deposit_asset(
        &self,
        initial_caller: Address,
        #[payment_token] asset: TokenIdentifier,
        #[payment] amount: BigUint,
    ) -> SCResult<()> {
        require!(amount > 0, "payment must be greater than 0");
        require!(!initial_caller.is_zero(), "invalid address provided");

        let pool_asset = self.pool_asset().get();
        require!(
            asset == pool_asset,
            "asset not supported for this liquidity pool"
        );

        let interest_metadata = InterestMetadata {
            timestamp: self.get_block_timestamp(),
        };

        self.mint_interest(amount.clone(), interest_metadata);

        let lend_token = self.lend_token().get();
        let nonce = self
            .get_current_esdt_nft_nonce(&self.get_sc_address(), lend_token.as_esdt_identifier());

        self.send().direct_esdt_nft_via_transfer_exec(
            &initial_caller,
            &lend_token.as_esdt_identifier(),
            nonce,
            &amount,
            &[],
        );

        let mut asset_reserve = self
            .reserves()
            .get(&pool_asset)
            .unwrap_or_else(BigUint::zero);
        asset_reserve += amount;

        self.reserves().insert(pool_asset, asset_reserve);

        Ok(())
    }

    #[endpoint]
    fn borrow(
        &self,
        initial_caller: Address,
        lend_token: TokenIdentifier,
        amount: BigUint,
        timestamp: u64,
    ) -> SCResult<()> {
        require!(
            self.get_caller() == self.get_lending_pool(),
            "can only be called through lending pool"
        );
        require!(amount > 0, "lend amount must be bigger then 0");
        require!(!initial_caller.is_zero(), "invalid address provided");

        let borrows_token = self.get_borrow_token();
        let asset = self.get_pool_asset();

        let mut borrows_reserve = self
            .reserves()
            .get(&borrows_token)
            .unwrap_or_else(BigUint::zero);
        let mut asset_reserve = self.reserves().get(&asset).unwrap_or_else(BigUint::zero);

        require!(asset_reserve != BigUint::zero(), "asset reserve is empty");

        let position_id = self.get_nft_hash();
        let debt_metadata = DebtMetadata {
            timestamp: self.get_block_timestamp(),
            collateral_amount: amount.clone(),
            collateral_identifier: lend_token.clone(),
            collateral_timestamp: timestamp,
        };

        self.mint_debt(amount.clone(), debt_metadata.clone(), position_id.clone());

        let nonce = self
            .get_current_esdt_nft_nonce(&self.get_sc_address(), borrows_token.as_esdt_identifier());

        // send debt position tokens

        self.send().direct_esdt_nft_via_transfer_exec(
            &initial_caller,
            &borrows_token.as_esdt_identifier(),
            nonce,
            &amount,
            &[],
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
        let debt_position = DebtPosition::<BigUint> {
            size: amount.clone(), // this will be initial L tokens amount
            health_factor: current_health,
            is_liquidated: false,
            timestamp: debt_metadata.timestamp,
            collateral_amount: amount,
            collateral_identifier: lend_token,
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
        #[payment] amount: BigUint,
    ) -> SCResult<H256> {
        require!(
            self.get_caller() == self.get_lending_pool(),
            "can only be called by lending pool"
        );
        require!(amount > 0, "amount must be greater then 0");
        require!(!initial_caller.is_zero(), "invalid address");

        require!(
            borrow_token == self.get_borrow_token(),
            "borrow token not supported by this pool"
        );

        let nft_nonce = self.call_value().esdt_token_nonce();

        let esdt_nft_data = self.get_esdt_token_data(
            &self.get_sc_address(),
            borrow_token.as_esdt_identifier(),
            nft_nonce,
        );

        let debt_position_id = esdt_nft_data.hash;
        let debt_position: DebtPosition<BigUint> = self
            .debt_positions()
            .get(&debt_position_id)
            .unwrap_or_default();

        require!(
            debt_position != DebtPosition::default(),
            "invalid debt position"
        );
        require!(!debt_position.is_liquidated, "position is liquidated");

        let metadata: DebtMetadata<BigUint>;
        match DebtMetadata::<BigUint>::top_decode(esdt_nft_data.attributes.as_slice()) {
            Result::Ok(decoded) => {
                metadata = decoded;
            }
            Result::Err(_) => {
                return sc_error!("could not parse token metadata");
            }
        }
        let data = [
            borrow_token.as_esdt_identifier(),
            amount.to_bytes_be().as_slice(),
            &nft_nonce.to_be_bytes()[..],
        ]
        .concat();

        let unique_repay_id = self.keccak256(&data);
        let repay_position = RepayPostion {
            identifier: borrow_token,
            amount,
            nonce: nft_nonce,
            borrow_timestamp: metadata.timestamp,
            collateral_identifier: metadata.collateral_identifier,
            collateral_amount: metadata.collateral_amount,
            collateral_timestamp: metadata.collateral_timestamp,
        };
        self.repay_position()
            .insert(unique_repay_id.clone(), repay_position);

        Ok(unique_repay_id)
    }

    #[payable("*")]
    #[endpoint(repay)]
    fn repay(
        &self,
        unique_id: H256,
        #[payment_token] asset: TokenIdentifier,
        #[payment] amount: BigUint,
    ) -> SCResult<RepayPostion<BigUint>> {
        require!(
            self.get_caller() == self.get_lending_pool(),
            "function can only be called by lending pool"
        );
        require!(amount > 0, "amount must be greater then 0");
        require!(
            asset == self.get_pool_asset(),
            "asset is not supported by this pool"
        );

        require!(
            self.repay_position().contains_key(&unique_id),
            "there are no locked borrowed token for this id, lock b tokens first"
        );
        let mut repay_position = self.repay_position().get(&unique_id).unwrap_or_default();

        require!(
            repay_position.amount >= amount,
            "b tokens amount locked must be equal with the amount of asset token send"
        );

        let esdt_nft_data = self.get_esdt_token_data(
            &self.get_sc_address(),
            repay_position.identifier.as_esdt_identifier(),
            repay_position.nonce,
        );

        let debt_position_id = esdt_nft_data.hash;

        require!(
            self.debt_positions().contains_key(&debt_position_id),
            "invalid debt position id"
        );
        let debt_position = self
            .debt_positions()
            .get(&debt_position_id)
            .unwrap_or_default();

        require!(!debt_position.is_liquidated, "position is liquidated");

        let interest = self.get_debt_interest(
            repay_position.amount.clone(),
            repay_position.borrow_timestamp,
        );

        if repay_position.amount.clone() + interest == amount {
            self.repay_position().remove(&unique_id);
        } else if repay_position.amount > amount {
            repay_position.amount -= amount.clone();
            self.repay_position()
                .insert(unique_id, repay_position.clone());
        }

        self.burn(
            amount.clone(),
            repay_position.nonce,
            repay_position.identifier.clone(),
        );

        repay_position.amount = amount;

        Ok(repay_position)
    }

    #[payable("*")]
    #[endpoint(mintLTokens)]
    fn mint_l_tokens(
        &self,
        initial_caller: Address,
        lend_token: TokenIdentifier,
        amount: BigUint,
        interest_timestamp: u64,
    ) -> SCResult<()> {
        require!(
            self.get_caller() == self.get_lending_pool(),
            "can only by called by lending pool"
        );

        require!(
            lend_token == self.get_lend_token(),
            "asset is not supported by this pool"
        );
        require!(amount > 0, "amount must be greater then 0");
        require!(!initial_caller.is_zero(), "invalid address");

        let interest_metadata = InterestMetadata {
            timestamp: interest_timestamp,
        };

        self.mint_interest(amount.clone(), interest_metadata);

        let nonce = self
            .get_current_esdt_nft_nonce(&self.get_sc_address(), lend_token.as_esdt_identifier());

        self.send().direct_esdt_nft_via_transfer_exec(
            &initial_caller,
            &lend_token.as_esdt_identifier(),
            nonce,
            &amount,
            &[],
        );

        Ok(())
    }

    #[payable("*")]
    #[endpoint]
    fn withdraw(
        &self,
        initial_caller: Address,
        #[payment_token] lend_token: TokenIdentifier,
        #[payment] amount: BigUint,
    ) -> SCResult<()> {
        require!(
            self.get_caller() == self.get_lending_pool(),
            "liquidity pool can only be called by lending pool"
        );
        require!(
            lend_token == self.get_lend_token(),
            "lend token is not supported by this pool"
        );
        require!(!initial_caller.is_zero(), "invalid address");
        require!(amount > 0, "amount must be bigger then 0");

        let pool_asset = self.get_pool_asset();
        let mut asset_reserve = self
            .reserves()
            .get(&pool_asset)
            .unwrap_or_else(BigUint::zero);

        require!(asset_reserve != BigUint::zero(), "asset reserve is empty");

        let nonce = self.call_value().esdt_token_nonce();
        self.burn(amount.clone(), nonce, lend_token);

        self.send()
            .direct(&initial_caller, &pool_asset, &amount, &[]);

        asset_reserve -= amount;
        self.reserves().insert(pool_asset, asset_reserve);

        Ok(())
    }

    #[payable("*")]
    #[endpoint(liquidate)]
    fn liquidate(
        &self,
        position_id: H256,
        #[payment_token] token: TokenIdentifier,
        #[payment] amount: BigUint,
    ) -> SCResult<LiquidateData<BigUint>> {
        require!(
            self.get_caller() == self.get_lending_pool(),
            "function can only be called by lending pool"
        );
        require!(amount > 0, "amount must be bigger then 0");
        require!(
            token == self.get_pool_asset(),
            "asset is not supported by this pool"
        );

        let mut debt_position = self.debt_positions().get(&position_id).unwrap_or_default();

        require!(
            debt_position != DebtPosition::default(),
            "invalid debt position id"
        );
        require!(
            !debt_position.is_liquidated,
            "position is already liquidated"
        );
        require!(
            debt_position.health_factor < self.get_health_factor_threshold(),
            "the health factor is not low enough"
        );

        let interest = self.get_debt_interest(debt_position.size.clone(), debt_position.timestamp);

        require!(
            debt_position.size.clone() + interest == amount,
            "position can't be liquidated, not enough or to much tokens send"
        );

        debt_position.is_liquidated = true;

        self.debt_positions()
            .insert(position_id, debt_position.clone());

        let liquidate_data = LiquidateData {
            collateral_token: debt_position.collateral_identifier,
            amount,
        };

        Ok(liquidate_data)
    }

    #[payable("EGLD")]
    #[endpoint]
    fn issue(
        &self,
        plain_ticker: BoxedBytes,
        token_ticker: TokenIdentifier,
        token_prefix: BoxedBytes,
        #[payment] issue_cost: BigUint,
    ) -> SCResult<AsyncCall<BigUint>> {
        only_owner!(self, "only owner can issue new tokens");
        require!(
            issue_cost == BigUint::from(ESDT_ISSUE_COST),
            "payment should be exactly 5 EGLD"
        );
        require!(
            token_ticker == self.pool_asset().get(),
            "wrong ESDT asset identifier"
        );

        let issue_data = self.prepare_issue_data(token_prefix.clone(), plain_ticker);
        require!(
            issue_data.name != BoxedBytes::zeros(0),
            "invalid input. could not prepare issue data"
        );
        require!(
            issue_data.is_empty_ticker,
            "token already issued for this identifier"
        );

        Ok(ESDTSystemSmartContractProxy::new()
            .issue_semi_fungible(
                issue_cost,
                &issue_data.name,
                &BoxedBytes::from(issue_data.ticker.as_esdt_identifier()),
                SemiFungibleTokenProperties {
                    can_freeze: true,
                    can_wipe: true,
                    can_pause: true,
                    can_change_owner: true,
                    can_upgrade: true,
                    can_add_special_roles: true,
                },
            )
            .async_call()
            .with_callback(self.callbacks().issue_callback(&token_prefix)))
    }

    #[endpoint(setLendTokenRoles)]
    fn set_lend_token_roles(
        &self,
        #[var_args] roles: VarArgs<EsdtLocalRole>,
    ) -> SCResult<AsyncCall<BigUint>> {
        only_owner!(self, "only owner can set roles");
        require!(!self.lend_token().is_empty(), "token not issued yet");
        Ok(self.set_roles(self.lend_token().get(), roles))
    }

    #[endpoint(setBorrowTokenRoles)]
    fn set_borrow_token_roles(
        &self,
        #[var_args] roles: VarArgs<EsdtLocalRole>,
    ) -> SCResult<AsyncCall<BigUint>> {
        only_owner!(self, "only owner can set roles");
        require!(!self.borrow_token().is_empty(), "token not issued yet");
        Ok(self.set_roles(self.borrow_token().get(), roles))
    }

    fn set_roles(
        &self,
        token: TokenIdentifier,
        #[var_args] roles: VarArgs<EsdtLocalRole>,
    ) -> AsyncCall<BigUint> {
        ESDTSystemSmartContractProxy::new()
            .set_special_roles(
                &self.get_sc_address(),
                token.as_esdt_identifier(),
                roles.as_slice(),
            )
            .async_call()
            .with_callback(self.callbacks().set_roles_callback())
    }

    #[callback]
    fn issue_callback(
        &self,
        prefix: &BoxedBytes,
        #[call_result] result: AsyncCallResult<TokenIdentifier>,
    ) {
        match result {
            AsyncCallResult::Ok(ticker) => {
                if prefix == &BoxedBytes::from(LEND_TOKEN_PREFIX) {
                    self.lend_token().set(&ticker);
                } else {
                    self.borrow_token().set(&ticker);
                }
                self.last_error().clear();
                self.send_callback_result(ticker, b"setTickerAfterIssue");
            }
            AsyncCallResult::Err(message) => {
                let caller = self.get_owner_address();
                let (returned_tokens, token_id) = self.call_value().payment_token_pair();
                if token_id.is_egld() && returned_tokens > 0 {
                    self.send().direct_egld(&caller, &returned_tokens, &[]);
                }
                self.last_error().set(&message.err_msg);
            }
        }
    }

    #[callback]
    fn set_roles_callback(&self, #[call_result] result: AsyncCallResult<()>) {
        match result {
            AsyncCallResult::Ok(()) => {
                self.last_error().clear();
            }
            AsyncCallResult::Err(message) => {
                self.last_error().set(&message.err_msg);
            }
        }
    }

    fn mint_interest(&self, amount: BigUint, metadata: InterestMetadata) {
        self.send().esdt_nft_create::<InterestMetadata>(
            self.get_gas_left(),
            self.lend_token().get().as_esdt_identifier(),
            &amount,
            &BoxedBytes::empty(),
            &BigUint::zero(),
            &H256::zero(),
            &metadata,
            &[BoxedBytes::empty()],
        )
    }

    fn mint_debt(&self, amount: BigUint, metadata: DebtMetadata<BigUint>, position_id: H256) {
        self.send().esdt_nft_create::<DebtMetadata<BigUint>>(
            self.get_gas_left(),
            self.borrow_token().get().as_esdt_identifier(),
            &amount,
            &BoxedBytes::empty(),
            &BigUint::zero(),
            &position_id,
            &metadata,
            &[BoxedBytes::empty()],
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

    fn send_callback_result(&self, token: TokenIdentifier, endpoint: &[u8]) {
        let owner = self.get_owner_address();

        let mut args = ArgBuffer::new();
        args.push_argument_bytes(token.as_esdt_identifier());

        self.send().execute_on_dest_context_raw(
            self.get_gas_left(),
            &owner,
            &BigUint::zero(),
            endpoint,
            &args,
        );
    }

    /// VIEWS

    #[view(getBorrowRate)]
    fn get_borrow_rate(&self) -> BigUint {
        let reserve_data = self.reserve_data().get();
        self._get_borrow_rate(reserve_data, OptionalArg::None)
    }

    #[view(getDepositRate)]
    fn get_deposit_rate(&self) -> BigUint {
        let utilisation = self.get_capital_utilisation();
        let reserve_data = self.reserve_data().get();
        let reserve_factor = reserve_data.reserve_factor.clone();
        let borrow_rate =
            self._get_borrow_rate(reserve_data, OptionalArg::Some(utilisation.clone()));

        self.library_module()
            .compute_deposit_rate(utilisation, borrow_rate, reserve_factor)
    }

    #[view(getDebtInterest)]
    fn get_debt_interest(&self, amount: BigUint, timestamp: u64) -> BigUint {
        let now = self.get_block_timestamp();
        let time_diff = BigUint::from(now - timestamp);

        let borrow_rate = self.get_borrow_rate();

        self.library_module()
            .compute_debt(amount, time_diff, borrow_rate)
    }

    #[view(getCapitalUtilisation)]
    fn get_capital_utilisation(&self) -> BigUint {
        let reserve_amount = self.get_reserve();
        let borrowed_amount = self.get_total_borrow();

        self.library_module()
            .compute_capital_utilisation(borrowed_amount, reserve_amount)
    }

    #[view(getReserve)]
    fn get_reserve(&self) -> BigUint {
        self.reserves()
            .get(&self.pool_asset().get())
            .unwrap_or_else(BigUint::zero)
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

    //
    /// UTILS
    fn prepare_issue_data(&self, prefix: BoxedBytes, ticker: BoxedBytes) -> IssueData {
        let prefixed_ticker = [prefix.as_slice(), ticker.as_slice()].concat();
        let mut issue_data = IssueData {
            name: BoxedBytes::zeros(0),
            ticker: TokenIdentifier::from(BoxedBytes::from(prefixed_ticker)),
            is_empty_ticker: true,
        };

        if prefix == BoxedBytes::from(LEND_TOKEN_PREFIX) {
            let name = [LEND_TOKEN_NAME, ticker.as_slice()].concat();
            issue_data.name = BoxedBytes::from(name.as_slice());
            issue_data.is_empty_ticker = self.lend_token().is_empty();
        } else if prefix == BoxedBytes::from(BORROW_TOKEN_PREFIX) {
            let name = [DEBT_TOKEN_NAME, ticker.as_slice()].concat();
            issue_data.name = BoxedBytes::from(name.as_slice());
            issue_data.is_empty_ticker = self.borrow_token().is_empty();
        }

        issue_data
    }

    fn get_nft_hash(&self) -> H256 {
        let debt_nonce = self.debt_nonce().get();
        let hash = self.keccak256(&debt_nonce.to_be_bytes()[..]);
        self.debt_nonce().set(&(debt_nonce + 1));
        hash
    }

    fn compute_health_factor(&self) -> u32 {
        0u32
    }

    fn _get_borrow_rate(
        &self,
        reserve_data: ReserveData<BigUint>,
        #[var_args] utilisation: OptionalArg<BigUint>,
    ) -> BigUint {
        let u_current = utilisation
            .into_option()
            .unwrap_or_else(|| self.get_capital_utilisation());

        self.library_module().compute_borrow_rate(
            reserve_data.r_base,
            reserve_data.r_slope1,
            reserve_data.r_slope2,
            reserve_data.u_optimal,
            u_current,
        )
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
    /// repay position
    #[storage_mapper("repay_position")]
    fn repay_position(&self) -> MapMapper<Self::Storage, H256, RepayPostion<BigUint>>;

    //
    /// reserve data
    #[storage_mapper("reserve_data")]
    fn reserve_data(&self) -> SingleValueMapper<Self::Storage, ReserveData<BigUint>>;

    //
    /// health factor threshold
    #[storage_set("healthFactorThreshold")]
    fn set_health_factor_threshold(&self, health_factor_threashdol: u32);

    #[view(healthFactorThreshold)]
    #[storage_get("healthFactorThreshold")]
    fn get_health_factor_threshold(&self) -> u32;

    //
    /// lending pool address
    #[storage_set("lendingPool")]
    fn set_lending_pool(&self, lending_pool: Address);

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
}
