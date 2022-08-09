#![no_std]
#![feature(generic_associated_types)]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use crate::elrond_codec::TopEncode;

#[elrond_wasm::module]
pub trait AccountTokenModule
// + token_send::TokenSendModule
// + pausable::PausableModule
// + admin_whitelist::AdminWhitelistModule
// + elrond_wasm_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    #[payable("EGLD")]
    #[endpoint(registerAccountToken)]
    fn register_account_token(
        &self,
        token_display_name: ManagedBuffer,
        token_ticker: ManagedBuffer,
        num_decimals: usize,
    ) {
        let payment_amount = self.call_value().egld_value();
        self.account_token().issue_and_set_all_roles(
            EsdtTokenType::NonFungible,
            payment_amount,
            token_display_name,
            token_ticker,
            num_decimals,
            None,
        );
    }

    fn mint_account_token<T: TopEncode>(&self, attributes: &T) {
        let big_zero = BigUint::zero();
        let big_one = BigUint::from(1u64);
        let empty_buffer = ManagedBuffer::new();
        let empty_vec = ManagedVec::from_raw_handle(empty_buffer.get_raw_handle());
        let account_token_id = self.account_token().get_token_id();

        let new_account_nonce = self.send().esdt_nft_create(
            &account_token_id,
            &big_one,
            &empty_buffer,
            &big_zero,
            &empty_buffer,
            &attributes,
            &empty_vec,
        );
        self.account_positions().insert(new_account_nonce);
    }

    fn burn_account_token(&self, token_id: &TokenIdentifier, account_nonce: u64) {
        let big_one = BigUint::from(1u64);

        self.send().esdt_local_burn(token_id, account_nonce, &big_one);
        self.account_positions().swap_remove(&account_nonce);
    }

    fn get_farm_token_attributes<T: TopDecode>(
        &self,
        token_id: &TokenIdentifier,
        token_nonce: u64,
    ) -> T {
        let token_info = self.blockchain().get_esdt_token_data(
            &self.blockchain().get_sc_address(),
            token_id,
            token_nonce,
        );

        token_info.decode_attributes()
    }

    #[view(getAccountToken)]
    #[storage_mapper("account_token")]
    fn account_token(&self) -> NonFungibleTokenMapper<Self::Api>;

    #[view(getAccountPositions)]
    #[storage_mapper("account_positions")]
    fn account_positions(&self) -> UnorderedSetMapper<u64>;
}
