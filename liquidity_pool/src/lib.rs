#![no_std]
#![allow(clippy::too_many_arguments)]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

pub mod math;
pub use math::*;
pub mod liquidity;
pub mod tokens;
pub use common_structs::*;

mod storage;
mod utils;

#[elrond_wasm::contract]
pub trait LiquidityPool:
    storage::StorageModule
    + tokens::TokensModule
    + math::MathModule
    + liquidity::LiquidityModule
    + utils::UtilsModule
    + price_aggregator_proxy::PriceAggregatorModule
    + common_checks::ChecksModule
    + token_send::TokenSendModule
{
    #[init]
    fn init(
        &self,
        asset: TokenIdentifier,
        r_base: BigUint,
        r_slope1: BigUint,
        r_slope2: BigUint,
        u_optimal: BigUint,
        reserve_factor: BigUint,
        liquidation_threshold: BigUint,
    ) {
        self.pool_asset().set(&asset);
        self.pool_params().set(&PoolParams {
            r_base,
            r_slope1,
            r_slope2,
            u_optimal,
            reserve_factor,
        });
        self.liquidation_threshold().set(&liquidation_threshold);
    }

    #[only_owner]
    #[endpoint(setTransferExecGasLimit)]
    fn set_transfer_exec_gas_limit(&self, gas_limit: u64) -> SCResult<()> {
        self.transfer_exec_gas_limit().set(gas_limit);
        Ok(())
    }
}
