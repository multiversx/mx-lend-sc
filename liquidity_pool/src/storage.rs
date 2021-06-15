elrond_wasm::imports!();


use elrond_wasm::*;
use elrond_wasm::storage::mappers::{SingleValueMapper, MapMapper};
use elrond_wasm::types::{TokenIdentifier, BoxedBytes, H256, Address};
use crate::{DebtPosition, ReserveData, RepayPostion};

#[elrond_wasm_derive::module]
pub trait StorageModule {


    //
    /// pool asset
    #[storage_mapper("pool_asset")]
    fn pool_asset(&self) -> SingleValueMapper<Self::Storage, TokenIdentifier>;

    //
    /// lend token supported for asset
    #[storage_mapper("lend_token")]
    fn lend_token(&self) -> SingleValueMapper<Self::Storage, TokenIdentifier>;

    //
    /// borrow token supported for collateral
    #[storage_mapper("borrow_token")]
    fn borrow_token(&self) -> SingleValueMapper<Self::Storage, TokenIdentifier>;

    //
    /// pool reserves
    #[storage_mapper("reserves")]
    fn reserves(&self) -> MapMapper<Self::Storage, TokenIdentifier, Self::BigUint>;

    //
    /// last error
    #[storage_mapper("last_error")]
    fn last_error(&self) -> SingleValueMapper<Self::Storage, BoxedBytes>;

    //
    /// debt positions
    #[storage_mapper("debt_positions")]
    fn debt_positions(&self) -> MapMapper<Self::Storage, BoxedBytes, DebtPosition<Self::BigUint>>;

    //
    /// debt nonce
    #[storage_mapper("debt_nonce")]
    fn debt_nonce(&self) -> SingleValueMapper<Self::Storage, u64>;

    //
    /// repay position
    #[storage_mapper("repay_position")]
    fn repay_position(&self) -> MapMapper<Self::Storage, BoxedBytes, RepayPostion<Self::BigUint>>;

    //
    /// reserve data
    #[storage_mapper("reserve_data")]
    fn reserve_data(&self) -> SingleValueMapper<Self::Storage, ReserveData<Self::BigUint>>;

    //
    /// health factor threshold
    #[endpoint(setHealthFactorThreshold)]
    #[storage_set("healthFactorThreshold")]
    fn set_health_factor_threshold(&self, health_factor_threashdol: u32);

    #[view(healthFactorThreshold)]
    #[storage_get("healthFactorThreshold")]
    fn get_health_factor_threshold(&self) -> u32;

    //
    /// lending pool address
    #[storage_set("lendingPool")]
    fn set_lending_pool(&self, lending_pool: Address);

    #[view(getLendingPool)]
    #[storage_get("lendingPool")]
    fn get_lending_pool(&self) -> Address;

    //
    // total borrowing from pool
    #[storage_set("totalBorrow")]
    fn set_total_borrow(&self, total: Self::BigUint);

    #[view(totalBorrow)]
    #[storage_get("totalBorrow")]
    fn get_total_borrow(&self) -> Self::BigUint;

    #[storage_set("assetReserve")]
    fn set_asset_reserve(&self, reserve: Self::BigUint);

    #[view(assetReserve)]
    #[storage_get("assetReserve")]
    fn get_asset_reserve(&self) -> Self::BigUint;

    #[storage_set("withdrawAmount")]
    fn set_withdraw_amount(&self, amount: Self::BigUint);

    #[view(withdrawAmount)]
    #[storage_get("withdrawAmount")]
    fn get_withdraw_amount(&self) -> Self::BigUint;
    

    #[storage_set("repayPositionAmount")]
    fn set_repay_position_amount(&self, amount: Self::BigUint);

    #[view(repayPositionAmount)]
    #[storage_get("repayPositionAmount")]
    fn get_repay_position_amount(&self) -> Self::BigUint;

    #[storage_set("repayPositionIdentifier")]
    fn set_repay_position_id(&self, id:TokenIdentifier);

    #[view(repayPositionIdentifier)]
    #[storage_get("repayPositionIdentifier")]
    fn get_repay_position_id(&self) -> TokenIdentifier;

    #[storage_set("repayPositionNonce")]
    fn set_repay_position_nonce(&self, nonce:u64);

    #[view(repayPositionNonce)]
    #[storage_get("repayPositionNonce")]
    fn get_repay_position_nonce(&self) -> u64;

}