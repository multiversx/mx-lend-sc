#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait ChecksModule {
    fn require_amount_greater_than_zero(&self, amount: &BigUint) {
        require!(amount > &0, "amount must be greater than 0");
    }

    fn require_non_zero_address(&self, address: &ManagedAddress) {
        require!(!address.is_zero(), "address is zero");
    }
}
