#![allow(clippy::too_many_arguments)]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use super::factory;
use super::proxy;

use common_structs::{BORROW_TOKEN_PREFIX, LEND_TOKEN_PREFIX};
use liquidity_pool::tokens::ProxyTrait as _;
use price_aggregator_proxy::ProxyTrait as _;

#[elrond_wasm::module]
pub trait RouterModule:
    proxy::ProxyModule + factory::FactoryModule + common_checks::ChecksModule
{
    #[only_owner]
    #[endpoint(createLiquidityPool)]
    fn create_liquidity_pool(
        &self,
        base_asset: TokenIdentifier,
        r_base: BigUint,
        r_slope1: BigUint,
        r_slope2: BigUint,
        u_optimal: BigUint,
        reserve_factor: BigUint,
        liquidation_threshold: BigUint,
    ) -> ManagedAddress {
        require!(
            !self.pools_map().contains_key(&base_asset),
            "asset already supported"
        );
        require!(base_asset.is_esdt(), "non-ESDT asset provided");

        let address = self.create_pool(
            base_asset.clone(),
            r_base,
            r_slope1,
            r_slope2,
            u_optimal,
            reserve_factor,
            liquidation_threshold,
        );

        self.require_non_zero_address(&address);

        self.pools_map().insert(base_asset, address.clone());
        self.pools_allowed().insert(address.clone());

        address
    }

    #[only_owner]
    #[endpoint(upgradeLiquidityPool)]
    fn upgrade_liquidity_pool(
        &self,
        base_asset: TokenIdentifier,
        r_base: BigUint,
        r_slope1: BigUint,
        r_slope2: BigUint,
        u_optimal: BigUint,
        reserve_factor: BigUint,
        liquidation_threshold: BigUint,
    ) {
        require!(
            self.pools_map().contains_key(&base_asset),
            "no pool found for this asset"
        );

        let pool_address = self.get_pool_address(&base_asset);
        self.upgrade_pool(
            pool_address,
            base_asset,
            r_base,
            r_slope1,
            r_slope2,
            u_optimal,
            reserve_factor,
            liquidation_threshold,
        );
    }

    #[only_owner]
    #[payable("EGLD")]
    #[endpoint(issueLendToken)]
    fn issue_lend_token(&self, pool_asset_id: TokenIdentifier, token_ticker: ManagedBuffer) {
        self.issue_common(pool_asset_id, LEND_TOKEN_PREFIX, token_ticker);
    }

    #[only_owner]
    #[payable("EGLD")]
    #[endpoint(issueBorrowToken)]
    fn issue_borrow_token(&self, pool_asset_id: TokenIdentifier, token_ticker: ManagedBuffer) {
        self.issue_common(pool_asset_id, BORROW_TOKEN_PREFIX, token_ticker);
    }

    fn issue_common(
        &self,
        pool_asset_id: TokenIdentifier,
        token_prefix: u8,
        token_ticker: ManagedBuffer,
    ) {
        let payment_amount = self.call_value().egld_value();
        let pool_address = self.get_pool_address(&pool_asset_id);
        let gas_limit = self.resolve_nested_async_gas_limit();

        self.liquidity_pool_proxy(pool_address)
            .issue(pool_asset_id, token_prefix, token_ticker)
            .with_egld_transfer(payment_amount)
            .with_gas_limit(gas_limit)
            .execute_on_dest_context_ignore_result();
    }

    #[only_owner]
    #[endpoint(setLendRoles)]
    fn set_lend_roles(&self, pool_asset_id: TokenIdentifier) {
        let pool_address = self.get_pool_address(&pool_asset_id);
        let gas_limit = self.resolve_nested_async_gas_limit();

        self.liquidity_pool_proxy(pool_address)
            .set_lend_token_roles()
            .with_gas_limit(gas_limit)
            .execute_on_dest_context_ignore_result();
    }

    #[only_owner]
    #[endpoint(setBorrowRoles)]
    fn set_borrow_roles(&self, pool_asset_id: TokenIdentifier) {
        let pool_address = self.get_pool_address(&pool_asset_id);
        let gas_limit = self.resolve_nested_async_gas_limit();

        self.liquidity_pool_proxy(pool_address)
            .set_borrow_token_roles()
            .with_gas_limit(gas_limit)
            .execute_on_dest_context_ignore_result();
    }

    #[only_owner]
    #[endpoint(setAggregator)]
    fn set_aggregator(&self, pool_asset_id: TokenIdentifier, aggregator: ManagedAddress) {
        let pool_address = self.get_pool_address(&pool_asset_id);

        self.liquidity_pool_proxy(pool_address)
            .set_price_aggregator_address(aggregator)
            .execute_on_dest_context_ignore_result();
    }

    #[only_owner]
    #[endpoint(setAssetLoanToValue)]
    fn set_asset_loan_to_value(&self, asset: TokenIdentifier, loan_to_value: BigUint) {
        self.asset_loan_to_value(&asset).set(&loan_to_value);
    }

    #[only_owner]
    #[endpoint(setAssetLiquidationBonus)]
    fn set_asset_liquidation_bonus(&self, asset: TokenIdentifier, liq_bonus: BigUint) {
        self.asset_liquidation_bonus(&asset).set(&liq_bonus);
    }

    #[endpoint(setTokenIdAfterIssue)]
    fn set_token_id_after_issue(&self, token_id: TokenIdentifier) {
        let caller = self.blockchain().get_caller();
        let is_pool_allowed = self.pools_allowed().contains(&caller);
        require!(is_pool_allowed, "access restricted: unknown caller address");
        require!(
            token_id.is_valid_esdt_identifier(),
            "invalid ticker provided"
        );
        self.pools_map().insert(token_id, caller);
    }

    #[view(getPoolAddress)]
    fn get_pool_address(&self, asset: &TokenIdentifier) -> ManagedAddress {
        match self.pools_map().get(asset) {
            Some(addr) => addr,
            None => sc_panic!("no pool address for asset"),
        }
    }

    fn get_liquidation_bonus_non_zero(&self, token_id: &TokenIdentifier) -> BigUint {
        let liq_bonus = self.asset_liquidation_bonus(token_id).get();
        require!(liq_bonus > 0, "no liquidation_bonus present for asset");

        liq_bonus
    }

    fn get_loan_to_value_exists_and_non_zero(&self, token_id: &TokenIdentifier) -> BigUint {
        require!(
            !self.asset_loan_to_value(token_id).is_empty(),
            "no loan_to_value value present for asset"
        );

        let loan_to_value = self.asset_loan_to_value(token_id).get();
        require!(loan_to_value > 0, "loan_to_value value can not be zero");

        loan_to_value
    }

    fn resolve_nested_async_gas_limit(&self) -> u64 {
        self.blockchain().get_gas_left() * 3 / 4
    }

    #[storage_mapper("pools_map")]
    fn pools_map(&self) -> MapMapper<TokenIdentifier, ManagedAddress>;

    #[view(getPoolAllowed)]
    #[storage_mapper("pool_allowed")]
    fn pools_allowed(&self) -> SetMapper<ManagedAddress>;

    #[view(getAssetLoanToValue)]
    #[storage_mapper("asset_loan_to_value")]
    fn asset_loan_to_value(&self, asset: &TokenIdentifier) -> SingleValueMapper<BigUint>;

    #[view(getAssetLiquidationBonus)]
    #[storage_mapper("asset_liquidation_bonus")]
    fn asset_liquidation_bonus(&self, asset: &TokenIdentifier) -> SingleValueMapper<BigUint>;
}
