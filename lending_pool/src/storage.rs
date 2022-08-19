elrond_wasm::imports!();

use common_structs::{BorrowPosition, DepositPosition};

#[elrond_wasm::module]
pub trait LendingStorageModule {
    #[view(getDepositPositions)]
    #[storage_mapper("deposit_positions")]
    // fn deposit_position(&self) -> UnorderedSetMapper<DepositPosition<Self::Api>>;
    fn deposit_positions(
        &self,
        owner_nonce: u64,
    ) -> MapMapper<TokenIdentifier, DepositPosition<Self::Api>>;

    #[view(getBorrowPositions)]
    #[storage_mapper("borrow_positions")]
    // fn borrow_position(&self) -> UnorderedSetMapper<BorrowPosition<Self::Api>>;
    fn borrow_positions(
        &self,
        owner_nonce: u64,
    ) -> MapMapper<TokenIdentifier, BorrowPosition<Self::Api>>;
}
