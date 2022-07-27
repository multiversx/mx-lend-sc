elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use common_structs::{BorrowPosition, DepositPosition, PoolParams};

#[elrond_wasm::module]
pub trait StorageModule {
    #[view(getPoolAsset)]
    #[storage_mapper("pool_asset")]
    fn pool_asset(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getReserves)]
    #[storage_mapper("reserves")]
    fn reserves(&self) -> SingleValueMapper<BigUint>;

    #[view(getRewardsReserves)]
    #[storage_mapper("rewards_reserves")]
    fn rewards_reserves(&self) -> SingleValueMapper<BigUint>;

    #[view(getRewardsReservesAccumulatedNotDistributed)]
    #[storage_mapper("rewards_reserves_accumulated_not_distributed")]
    fn rewards_reserves_accumulated_not_distributed(&self) -> SingleValueMapper<BigUint>;

    #[view(getLendToken)]
    #[storage_mapper("lend_token")]
    fn lend_token(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(borrowToken)]
    #[storage_mapper("borrow_token")]
    fn borrow_token(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getDepositPosition)]
    #[storage_mapper("deposit_position")]
    fn deposit_position(&self, nonce: u64) -> SingleValueMapper<DepositPosition<Self::Api>>;

    #[view(getBorrowMetadata)]
    #[storage_mapper("borrow_position")]
    fn borrow_position(&self, nonce: u64) -> SingleValueMapper<BorrowPosition<Self::Api>>;

    #[view(getPoolParams)]
    #[storage_mapper("pool_params")]
    fn pool_params(&self) -> SingleValueMapper<PoolParams<Self::Api>>;

    #[view(getTotalBorrow)]
    #[storage_mapper("borrowed_amount")]
    fn borrowed_amount(&self) -> SingleValueMapper<BigUint>;

    #[view(getLiquidationThreshold)]
    #[storage_mapper("liquidation_threshold")]
    fn liquidation_threshold(&self) -> SingleValueMapper<BigUint>;

    #[view(getBorrowIndex)]
    #[storage_mapper("borrow_index")]
    fn borrow_index(&self) -> SingleValueMapper<BigUint>;

    #[view(getSupplyIndex)]
    #[storage_mapper("supply_index")]
    fn supply_index(&self) -> SingleValueMapper<BigUint>;

    #[view(borrowIndexLastUpdateRound)]
    #[storage_mapper("borrow_index_last_update_round")]
    fn borrow_index_last_update_round(&self) -> SingleValueMapper<u64>;
}
