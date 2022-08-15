elrond_wasm::imports!();

use common_structs::{BorrowPosition, DepositPosition};

#[elrond_wasm::module]
pub trait LendingStorageModule {
    #[view(getDepositPosition)]
    #[storage_mapper("deposit_position")]
    fn deposit_position(&self) -> UnorderedSetMapper<DepositPosition<Self::Api>>;

    #[view(getBorrowMetadata)]
    #[storage_mapper("borrow_position")]
    fn borrow_position(&self) -> UnorderedSetMapper<BorrowPosition<Self::Api>>;

}