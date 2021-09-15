elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use common_structs::{DebtPosition, InterestMetadata, PoolParams};

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

    #[storage_mapper("interest_metadata")]
    fn interest_metadata(&self, nonce: u64) -> SingleValueMapper<Self::Storage, InterestMetadata>;

    #[storage_mapper("debt_metadata")]
    fn debt_metadata(&self, nonce: u64) -> SingleValueMapper<Self::Storage, InterestMetadata>;

    #[view(getLastError)]
    #[storage_mapper("last_error")]
    fn last_error(&self) -> SingleValueMapper<Self::Storage, BoxedBytes>;

    #[storage_mapper("debt_positions")]
    fn debt_positions(
        &self,
    ) -> SafeMapMapper<Self::Storage, BoxedBytes, DebtPosition<Self::BigUint>>;

    #[view(getPoolParams)]
    #[storage_mapper("pool_params")]
    fn pool_params(&self) -> SingleValueMapper<Self::Storage, PoolParams<Self::BigUint>>;

    #[view(getHealthFactorThreshold)]
    #[storage_mapper("health_factor_threshold")]
    fn health_factor_threshold(&self) -> SingleValueMapper<Self::Storage, u32>;

    #[view(getLendingPool)]
    #[storage_mapper("lending_pool")]
    fn lending_pool(&self) -> SingleValueMapper<Self::Storage, Address>;

    #[view(getTotalBorrow)]
    #[storage_mapper("borrowed_amount")]
    fn borrowed_amount(&self) -> SingleValueMapper<Self::Storage, Self::BigUint>;
}
