#![allow(clippy::too_many_arguments)]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use super::factory;
use super::liq_pools;

use common_structs::{BORROW_TOKEN_PREFIX, LEND_TOKEN_PREFIX};
use liquidity_pool::tokens::ProxyTrait as _;

#[elrond_wasm::module]
pub trait RouterModule: liq_pools::LiqPoolsModule + factory::FactoryModule {
    #[only_owner]
    #[endpoint(createLiquidityPool)]
    fn create_liquidity_pool(
        &self,
        base_asset: TokenIdentifier,
        lending_pool_address: Address,
        r_base: Self::BigUint,
        r_slope1: Self::BigUint,
        r_slope2: Self::BigUint,
        u_optimal: Self::BigUint,
        reserve_factor: Self::BigUint,
        pool_bytecode: BoxedBytes,
    ) -> SCResult<Address> {
        require!(
            !self.pools_map().contains_key(&base_asset),
            "asset already supported"
        );
        require!(base_asset.is_esdt(), "non-ESDT asset provided");

        let address = self.create_pool(
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
            self.pools_allowed().insert(address.clone());
        }

        Ok(address)
    }

    #[only_owner]
    #[endpoint(upgradeLiquidityPool)]
    fn upgrade_liquidity_pool(
        &self,
        base_asset: TokenIdentifier,
        new_bytecode: BoxedBytes,
    ) -> SCResult<()> {
        require!(
            self.pools_map().contains_key(&base_asset),
            "no pool found for this asset"
        );

        let pool_address = self.pools_map().get(&base_asset).unwrap();
        let success = self.upgrade_pool(&pool_address, &new_bytecode);
        require!(success, "pair upgrade failed");

        Ok(())
    }

    #[only_owner]
    #[payable("EGLD")]
    #[endpoint(issueLendToken)]
    fn issue_lend_token(
        &self,
        plain_ticker: BoxedBytes,
        token_ticker: TokenIdentifier,
        #[payment_amount] amount: Self::BigUint,
    ) -> SCResult<()> {
        let pool_address = self.pools_map().get(&token_ticker).unwrap();
        self.liquidity_pool_proxy(pool_address)
            .issue(
                plain_ticker,
                token_ticker,
                BoxedBytes::from(LEND_TOKEN_PREFIX),
                amount,
            )
            .execute_on_dest_context();

        Ok(())
    }

    #[only_owner]
    #[payable("EGLD")]
    #[endpoint(issueBorrowToken)]
    fn issue_borrow_token(
        &self,
        plain_ticker: BoxedBytes,
        token_ticker: TokenIdentifier,
        #[payment_amount] amount: Self::BigUint,
    ) -> SCResult<()> {
        let pool_address = self.pools_map().get(&token_ticker).unwrap();
        self.liquidity_pool_proxy(pool_address)
            .issue(
                plain_ticker,
                token_ticker,
                BoxedBytes::from(BORROW_TOKEN_PREFIX),
                amount,
            )
            .execute_on_dest_context();

        Ok(())
    }

    #[only_owner]
    #[endpoint(setLendRoles)]
    fn set_lend_roles(
        &self,
        asset_ticker: TokenIdentifier,
        #[var_args] roles: VarArgs<EsdtLocalRole>,
    ) -> SCResult<()> {
        let pool_address = self.pools_map().get(&asset_ticker).unwrap();
        self.liquidity_pool_proxy(pool_address)
            .set_lend_token_roles(roles.into_vec())
            .execute_on_dest_context();

        Ok(())
    }

    #[only_owner]
    #[endpoint(setBorrowRoles)]
    fn set_borrow_roles(
        &self,
        asset_ticker: TokenIdentifier,
        #[var_args] roles: VarArgs<EsdtLocalRole>,
    ) -> SCResult<()> {
        let pool_address = self.pools_map().get(&asset_ticker).unwrap();
        self.liquidity_pool_proxy(pool_address)
            .set_borrow_token_roles(roles.into_vec())
            .execute_on_dest_context();

        Ok(())
    }

    #[endpoint(setTickerAfterIssue)]
    fn set_ticker_after_issue(&self, token_ticker: TokenIdentifier) -> SCResult<()> {
        let caller = self.blockchain().get_caller();
        let is_pool_allowed = self.pools_allowed().contains(&caller);
        require!(is_pool_allowed, "access restricted: unknown caller address");
        require!(
            token_ticker.is_valid_esdt_identifier(),
            "invalid ticker provided"
        );
        self.pools_map().insert(token_ticker, caller);
        Ok(())
    }

    #[view(getPoolAddress)]
    fn get_pool_address(&self, asset: &TokenIdentifier) -> Address {
        self.pools_map().get(asset).unwrap_or_else(Address::zero)
    }
}
