#![no_std]
#![allow(clippy::too_many_arguments)]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

pub mod library;
pub use library::*;
pub mod liquidity;
pub mod multi_transfer;
pub mod tokens;
pub use common_structs::*;

mod storage;
mod utils;

#[elrond_wasm::contract]
pub trait LiquidityPool:
    storage::StorageModule
    + tokens::TokensModule
    + library::LibraryModule
    + liquidity::LiquidityModule
    + multi_transfer::MultiTransferModule
    + utils::UtilsModule
    + price_aggregator_proxy::PriceAggregatorModule
{
    #[init]
    fn init(
        &self,
        asset: TokenIdentifier,
        r_base: Self::BigUint,
        r_slope1: Self::BigUint,
        r_slope2: Self::BigUint,
        u_optimal: Self::BigUint,
        reserve_factor: Self::BigUint,
    ) {
        self.pool_asset().set(&asset);
        self.pool_params().set(&PoolParams {
            r_base,
            r_slope1,
            r_slope2,
            u_optimal,
            reserve_factor,
        });
    }

    #[only_owner]
    #[endpoint(setHealthFactorThreshold)]
    fn endpoint_health_factor_threshold(&self, health_factor_threashdol: u32) {
        self.health_factor_threshold()
            .set(&health_factor_threashdol);
    }
}
