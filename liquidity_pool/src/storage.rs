elrond_wasm::imports!();

use crate::{DebtPosition, PoolParams, RepayPostion};
use elrond_wasm::storage::mappers::{SafeMapMapper, SingleValueMapper};
use elrond_wasm::types::{Address, BoxedBytes, TokenIdentifier};

#[elrond_wasm::module]
pub trait StorageModule {
    #[storage_mapper("pool_asset")]
    fn pool_asset(&self) -> SingleValueMapper<Self::Storage, TokenIdentifier>;

    #[storage_mapper("lend_token")]
    fn lend_token(&self) -> SingleValueMapper<Self::Storage, TokenIdentifier>;

    #[storage_mapper("borrow_token")]
    fn borrow_token(&self) -> SingleValueMapper<Self::Storage, TokenIdentifier>;

    #[storage_mapper("reserves")]
    fn reserves(
        &self,
        token_id: &TokenIdentifier,
    ) -> SingleValueMapper<Self::Storage, Self::BigUint>;

    #[storage_mapper("last_error")]
    fn last_error(&self) -> SingleValueMapper<Self::Storage, BoxedBytes>;

    #[storage_mapper("debt_positions")]
    fn debt_positions(
        &self,
    ) -> SafeMapMapper<Self::Storage, BoxedBytes, DebtPosition<Self::BigUint>>;

    #[storage_mapper("debt_nonce")]
    fn debt_nonce(&self) -> SingleValueMapper<Self::Storage, u64>;

    #[storage_mapper("repay_position")]
    fn repay_position(
        &self,
    ) -> SafeMapMapper<Self::Storage, BoxedBytes, RepayPostion<Self::BigUint>>;

    #[storage_mapper("pool_params")]
    fn pool_params(&self) -> SingleValueMapper<Self::Storage, PoolParams<Self::BigUint>>;

    #[storage_mapper("healthFactorThreshold")]
    fn health_factor_threshold(&self) -> SingleValueMapper<Self::Storage, u32>;

    #[storage_mapper("lendingPool")]
    fn lending_pool(&self) -> SingleValueMapper<Self::Storage, Address>;

    #[storage_mapper("totalBorrow")]
    fn total_borrow(&self) -> SingleValueMapper<Self::Storage, Self::BigUint>;

    #[storage_mapper("repayPositionAmount")]
    fn repay_position_amount(&self) -> SingleValueMapper<Self::Storage, Self::BigUint>;

    #[storage_mapper("repayPositionIdentifier")]
    fn repay_position_id(&self) -> SingleValueMapper<Self::Storage, TokenIdentifier>;

    #[storage_mapper("repayPositionNonce")]
    fn repay_position_nonce(&self) -> SingleValueMapper<Self::Storage, u64>;
}
