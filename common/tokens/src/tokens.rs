#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[elrond_wasm::module]
pub trait AccountTokenModule {
    #[view(getAccountToken)]
    #[storage_mapper("account_token")]
    fn account_token(&self) -> NonFungibleTokenMapper<Self::Api>;

    #[view(getAccountPositions)]
    #[storage_mapper("account_positions")]
    fn account_positions(&self) -> UnorderedSetMapper<u64>;
}
