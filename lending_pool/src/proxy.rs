multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait ProxyModule {
    #[proxy]
    fn liquidity_pool_proxy(&self, sc_address: ManagedAddress) -> liquidity_pool::Proxy<Self::Api>;
}
