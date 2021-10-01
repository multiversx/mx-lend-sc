elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use common_structs::{BorrowPosition, DepositPosition, PoolParams};

#[elrond_wasm::module]
pub trait StorageModule {
    #[view(getPoolAsset)]
    #[storage_mapper("pool_asset")]
    fn pool_asset(&self) -> SingleValueMapper<Self::Storage, TokenIdentifier>;

    #[view(getLendToken)]
    #[storage_mapper("lend_token")]
    fn lend_token(&self) -> SingleValueMapper<Self::Storage, TokenIdentifier>;

    #[view(borrowToken)]
    #[storage_mapper("borrow_token")]
    fn borrow_token(&self) -> SingleValueMapper<Self::Storage, TokenIdentifier>;

    #[view(getReserves)]
    #[storage_mapper("reserves")]
    fn reserves(
        &self,
        token_id: &TokenIdentifier,
    ) -> SingleValueMapper<Self::Storage, Self::BigUint>;

    #[view(getDepositPosition)]
    #[storage_mapper("deposit_position")]
    fn deposit_position(
        &self,
        nonce: u64,
    ) -> SingleValueMapper<Self::Storage, DepositPosition<Self::BigUint>>;

    #[view(getBorrowMetadata)]
    #[storage_mapper("borrow_position")]
    fn borrow_position(
        &self,
        nonce: u64,
    ) -> SingleValueMapper<Self::Storage, BorrowPosition<Self::BigUint>>;

    #[view(getLastError)]
    #[storage_mapper("last_error")]
    fn last_error(&self) -> SingleValueMapper<Self::Storage, BoxedBytes>;

    #[view(getPoolParams)]
    #[storage_mapper("pool_params")]
    fn pool_params(&self) -> SingleValueMapper<Self::Storage, PoolParams<Self::BigUint>>;

    #[view(getTotalBorrow)]
    #[storage_mapper("borrowed_amount")]
    fn borrowed_amount(&self) -> SingleValueMapper<Self::Storage, Self::BigUint>;

    #[view(getLiquidationThreshold)]
    #[storage_mapper("liquidation_threshold")]
    fn liquidation_threshold(&self) -> SingleValueMapper<Self::Storage, Self::BigUint>;
}
