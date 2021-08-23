#![no_std]
#![allow(clippy::too_many_arguments)]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

pub mod library;
pub use library::*;
pub mod liquidity;
pub mod models;
pub mod tokens;
pub use models::*;

mod storage;
mod utils;

const LEND_TOKEN_PREFIX: &[u8] = b"L";
const BORROW_TOKEN_PREFIX: &[u8] = b"B";
const LEND_TOKEN_NAME: &[u8] = b"IntBearing";
const DEBT_TOKEN_NAME: &[u8] = b"DebtBearing";

#[elrond_wasm::contract]
pub trait LiquidityPool:
    storage::StorageModule
    + tokens::TokensModule
    + library::LibraryModule
    + liquidity::LiquidityModule
    + utils::UtilsModule
{
    #[init]
    fn init(
        &self,
        asset: TokenIdentifier,
        lending_pool: Address,
        r_base: Self::BigUint,
        r_slope1: Self::BigUint,
        r_slope2: Self::BigUint,
        u_optimal: Self::BigUint,
        reserve_factor: Self::BigUint,
    ) {
        self.pool_asset().set(&asset);
        self.lending_pool().set(&lending_pool);
        self.debt_nonce().set_if_empty(&1u64);
        self.pool_params().set(&PoolParams {
            r_base,
            r_slope1,
            r_slope2,
            u_optimal,
            reserve_factor,
        });
    }

    #[view(repayPositionsIds)]
    fn get_repay_positions_ids(&self) -> MultiResultVec<BoxedBytes> {
        self.repay_position().keys().collect()
    }

    #[view(repayPosition)]
    fn view_repay_position(&self, position_id: BoxedBytes) -> Option<RepayPostion<Self::BigUint>> {
        self.repay_position().get(&position_id)
    }

    #[view(debtPosition)]
    fn view_debt_position(&self, position_id: BoxedBytes) -> Option<DebtPosition<Self::BigUint>> {
        self.debt_positions().get(&position_id)
    }

    #[view(getDebtInterest)]
    fn view_debt_interest(&self, amount: Self::BigUint, timestamp: u64) -> SCResult<Self::BigUint> {
        self.get_debt_interest(amount, timestamp)
    }

    #[view(getPositionInterest)]
    fn get_debt_position_interest(&self, position_id: BoxedBytes) -> SCResult<Self::BigUint> {
        let debt_position = self.debt_positions().get(&position_id).unwrap_or_default();
        self.get_debt_interest(debt_position.size.clone(), debt_position.timestamp)
    }

    #[only_owner]
    #[endpoint(setHealthFactorThreshold)]
    fn endpoint_health_factor_threshold(&self, health_factor_threashdol: u32) {
        self.health_factor_threshold()
            .set(&health_factor_threashdol);
    }
}
