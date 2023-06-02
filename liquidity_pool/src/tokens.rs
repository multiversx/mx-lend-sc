multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use super::liq_math;
use super::liq_storage;
use super::liq_utils;

#[multiversx_sc::module]
pub trait TokensModule:
    liq_storage::StorageModule
    + liq_utils::UtilsModule
    + liq_math::MathModule
    + price_aggregator_proxy::PriceAggregatorModule
{
    #[proxy]
    fn lending_pool_proxy(
        &self,
        sc_address: ManagedAddress,
    ) -> lending_pool_proxy_mod::Proxy<Self::Api>;
}

// can't simply import, we would have a circular dependency
mod lending_pool_proxy_mod {
    multiversx_sc::imports!();

    #[multiversx_sc::proxy]
    pub trait LendingPool {
        #[endpoint(setTokenIdAfterIssue)]
        fn set_token_id_after_issue(&self, token_id: TokenIdentifier);
    }
}
