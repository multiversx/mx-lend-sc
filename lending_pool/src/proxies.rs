elrond_wasm::imports!();
use crate::LiquidateData;
use crate::RepayPostion;
use elrond_wasm::types::Address;


#[elrond_wasm_derive::module]
pub trait ProxiesModule {

    #[proxy]
    fn liquidity_pool_proxy(&self, sc_address: Address) -> liquidity_pool::Proxy<Self::SendApi>;

    #[proxy]
    fn router_proxy(&self, sc_address: Address) -> router::Proxy<Self::SendApi>;

}


