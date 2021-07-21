elrond_wasm::imports!();


use elrond_wasm::*;
use elrond_wasm::storage::mappers::{SingleValueMapper, MapMapper};
use elrond_wasm::types::{TokenIdentifier, BoxedBytes, Address};
use crate::{DebtPosition, ReserveData, RepayPostion};

#[elrond_wasm_derive::module]
pub trait StorageModule {

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


    #[storage_mapper("healthFactorThreshold")]
    fn health_factor_threshold(&self) -> SingleValueMapper<Self::Storage, u32>;

    //
    /// lending pool address
    #[storage_mapper("lendingPool")]
    fn lending_pool(&self) -> SingleValueMapper<Self::Storage, Address>;

    //
    // total borrowing from pool
    #[storage_mapper("totalBorrow")]
    fn total_borrow(&self) -> SingleValueMapper<Self::Storage, Self::BigUint>;


    #[storage_mapper("assetReserve")]
    fn asset_reserve(&self) -> SingleValueMapper<Self::Storage, Self::BigUint>;

    #[storage_mapper("withdrawAmount")]
    fn withdraw_amount(&self) ->SingleValueMapper<Self::Storage, Self::BigUint>;

    #[storage_mapper("repayPositionAmount")]
    fn repay_position_amount(&self) -> SingleValueMapper< Self::Storage, Self::BigUint>;

    #[storage_mapper("repayPositionIdentifier")]
    fn repay_position_id(&self) -> SingleValueMapper<Self::Storage, TokenIdentifier>;

    #[storage_mapper("repayPositionNonce")]
    fn repay_position_nonce(&self) -> SingleValueMapper<Self::Storage, u64>;

}