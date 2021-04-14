#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use elrond_wasm::{only_owner, require, sc_error};

pub mod pool_factory;
pub use pool_factory::*;

const LEND_TOKEN_PREFIX: &[u8] = b"L";
const BORROW_TOKEN_PREFIX: &[u8] = b"B";

const ISSUE_EXPECTED_GAS_COST: u64 = 90000000 + 30000000;

#[elrond_wasm_derive::callable(LiquidityPoolProxy)]
pub trait LiquidityPool {
    fn issue(
        &self,
        plain_ticker: BoxedBytes,
        token_ticker: TokenIdentifier,
        token_prefix: BoxedBytes,
    ) -> ContractCall<BigUint, ()>;

    fn set_lend_token_roles(
        &self,
        #[var_args] roles: VarArgs<EsdtLocalRole>,
    ) -> ContractCall<BigUint, ()>;

    fn set_borrow_token_roles(
        &self,
        #[var_args] roles: VarArgs<EsdtLocalRole>,
    ) -> ContractCall<BigUint, ()>;
}

#[elrond_wasm_derive::contract(RouterImpl)]
pub trait Router {
    #[module(PoolFactoryModuleImpl)]
    fn pool_factory(&self) -> PoolFactoryModuleImpl<T, BigInt, BigUint>;

    #[init]
    fn init(&self) {
        self.pool_factory().init();
    }

    /// ENDPOINTS

    #[endpoint(createLiquidityPool)]
    fn create_liquidity_pool(
        &self,
        base_asset: TokenIdentifier,
        lending_pool_address: Address,
        r_base: BigUint,
        r_slope1: BigUint,
        r_slope2: BigUint,
        u_optimal: BigUint,
        reserve_factor: BigUint,
        pool_bytecode: BoxedBytes,
    ) -> SCResult<Address> {
        only_owner!(self, "only owner can create new pools");
        require!(
            !self.pools_map().contains_key(&base_asset),
            "asset already supported"
        );
        require!(base_asset.is_esdt(), "non-ESDT asset provided");

        let address = self.pool_factory().create_pool(
            &base_asset,
            &lending_pool_address,
            r_base,
            r_slope1,
            r_slope2,
            u_optimal,
            reserve_factor,
            &pool_bytecode,
        );

        if !address.is_zero() {
            self.pools_map().insert(base_asset, address.clone());
            self.pools_allowed().insert(address.clone(), true);
        }

        Ok(address)
    }

    #[endpoint(upgradeLiquidityPool)]
    fn upgrade_liquidity_pool(
        &self,
        base_asset: TokenIdentifier,
        new_bytecode: BoxedBytes,
    ) -> SCResult<()> {
        only_owner!(self, "only owner can upgrade existing pools");

        require!(
            self.pools_map().contains_key(&base_asset),
            "no pool found for this asset"
        );

        let pool_address = self.pools_map().get(&base_asset).unwrap();

        let success = self
            .pool_factory()
            .upgrade_pool(&pool_address, &new_bytecode);

        if !success {
            return sc_error!("pair upgrade failed");
        }

        Ok(())
    }

    #[payable("EGLD")]
    #[endpoint(issueLendToken)]
    fn issue_lend_token(
        &self,
        plain_ticker: BoxedBytes,
        token_ticker: TokenIdentifier,
        #[payment] amount: BigUint,
    ) -> SCResult<()> {
        only_owner!(self, "only owner may call this function");
        let pool_address = self.pools_map().get(&token_ticker).unwrap();
        Ok(contract_call!(self, pool_address, LiquidityPoolProxy)
            .with_token_transfer(TokenIdentifier::egld(), amount)
            .issue(
                plain_ticker,
                token_ticker,
                BoxedBytes::from(LEND_TOKEN_PREFIX),
            )
            .execute_on_dest_context(ISSUE_EXPECTED_GAS_COST, self.send()))
    }

    #[payable("EGLD")]
    #[endpoint(issueBorrowToken)]
    fn issue_borrow_token(
        &self,
        plain_ticker: BoxedBytes,
        token_ticker: TokenIdentifier,
        #[payment] amount: BigUint,
    ) -> SCResult<()> {
        only_owner!(self, "only owner may call this function");
        let pool_address = self.pools_map().get(&token_ticker).unwrap();
        Ok(contract_call!(self, pool_address, LiquidityPoolProxy)
            .with_token_transfer(TokenIdentifier::egld(), amount)
            .issue(
                plain_ticker,
                token_ticker,
                BoxedBytes::from(BORROW_TOKEN_PREFIX),
            )
            .execute_on_dest_context(ISSUE_EXPECTED_GAS_COST, self.send()))
    }

    #[endpoint(setLendRoles)]
    fn set_lend_roles(
        &self,
        asset_ticker: TokenIdentifier,
        #[var_args] roles: VarArgs<EsdtLocalRole>,
    ) -> SCResult<()> {
        only_owner!(self, "only owner may call this function");
        let pool_address = self.pools_map().get(&asset_ticker).unwrap();
        contract_call!(self, pool_address, LiquidityPoolProxy)
            .set_lend_token_roles(roles)
            .execute_on_dest_context(self.get_gas_left(), self.send());

        Ok(())
    }

    #[endpoint(setBorrowRoles)]
    fn set_borrow_roles(
        &self,
        asset_ticker: TokenIdentifier,
        #[var_args] roles: VarArgs<EsdtLocalRole>,
    ) -> SCResult<()> {
        only_owner!(self, "only owner may call this function");
        let pool_address = self.pools_map().get(&asset_ticker).unwrap();
        contract_call!(self, pool_address, LiquidityPoolProxy)
            .set_lend_token_roles(roles)
            .execute_on_dest_context(self.get_gas_left(), self.send());

        Ok(())
    }

    #[endpoint(setTickerAfterIssue)]
    fn set_ticker_after_issue(&self, token_ticker: TokenIdentifier) -> SCResult<()> {
        let caller = self.get_caller();
        let is_pool_allowed = self.pools_allowed().get(&caller).unwrap_or_default();
        require!(is_pool_allowed, "access restricted: unknown caller address");
        require!(!token_ticker.is_egld(), "invalid ticker provided");
        self.pools_map().insert(token_ticker, caller);
        Ok(())
    }

    /// VIEWS

    #[view(getPoolAddress)]
    fn get_pool_address(&self, asset: TokenIdentifier) -> Address {
        self.pools_map().get(&asset).unwrap_or_else(Address::zero)
    }

    //
    /// STORAGE

    #[storage_mapper("pools_map")]
    fn pools_map(&self) -> MapMapper<Self::Storage, TokenIdentifier, Address>;

    #[storage_mapper("pool_allowed")]
    fn pools_allowed(&self) -> MapMapper<Self::Storage, Address, bool>;
}
