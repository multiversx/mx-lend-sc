elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait ProxyModule {
    #[proxy]
    fn liquidity_pool_proxy(&self, sc_address: Address) -> liquidity_pool::Proxy<Self::SendApi>;
}
