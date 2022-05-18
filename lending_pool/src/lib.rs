#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

pub mod factory;
mod proxy;
pub mod router;

pub use common_structs::*;
use liquidity_pool::liquidity::ProxyTrait as _;

#[elrond_wasm::contract]
pub trait LendingPool:
    factory::FactoryModule + router::RouterModule + common_checks::ChecksModule + proxy::ProxyModule
{
    #[init]
    fn init(&self, lp_template_address: ManagedAddress) {
        self.liq_pool_template_address().set(&lp_template_address);
    }

    #[payable("*")]
    #[endpoint]
    fn deposit(&self, caller: OptionalValue<ManagedAddress>) {
        let (amount, asset) = self.call_value().payment_token_pair();
        let initial_caller = self.caller_from_option_or_sender(caller);

        self.require_amount_greater_than_zero(&amount);
        self.require_non_zero_address(&initial_caller);

        let pool_address = self.get_pool_address(&asset);

        self.liquidity_pool_proxy(pool_address)
            .deposit_asset(initial_caller)
            .add_token_transfer(asset, 0, amount)
            .execute_on_dest_context_ignore_result();
    }

    #[payable("*")]
    #[endpoint]
    fn withdraw(&self, caller: OptionalValue<ManagedAddress>) {
        let (amount, lend_token) = self.call_value().payment_token_pair();
        let token_nonce = self.call_value().esdt_token_nonce();
        let initial_caller = self.caller_from_option_or_sender(caller);

        self.require_amount_greater_than_zero(&amount);
        self.require_non_zero_address(&initial_caller);

        let pool_address = self.get_pool_address(&lend_token);
        self.liquidity_pool_proxy(pool_address)
            .withdraw(initial_caller)
            .add_token_transfer(lend_token, token_nonce, amount)
            .execute_on_dest_context_ignore_result();
    }

    #[payable("*")]
    #[endpoint]
    fn borrow(&self, asset_to_borrow: TokenIdentifier, caller: OptionalValue<ManagedAddress>) {
        let (payment_amount, payment_lend_id) = self.call_value().payment_token_pair();
        let payment_nonce = self.call_value().esdt_token_nonce();
        let initial_caller = self.caller_from_option_or_sender(caller);

        self.require_amount_greater_than_zero(&payment_amount);
        self.require_non_zero_address(&initial_caller);

        let borrow_token_pool_address = self.get_pool_address(&asset_to_borrow);
        let loan_to_value = self.get_loan_to_value_exists_and_non_zero(&payment_lend_id);

        //L tokens for a specific token X are 1:1 with deposited X tokens
        self.liquidity_pool_proxy(borrow_token_pool_address)
            .borrow(initial_caller, loan_to_value)
            .add_token_transfer(payment_lend_id, payment_nonce, payment_amount)
            .execute_on_dest_context_ignore_result();
    }

    #[payable("*")]
    #[endpoint]
    fn repay(&self, asset_to_repay: TokenIdentifier, caller: OptionalValue<ManagedAddress>) {
        let transfers = self.call_value().all_esdt_transfers();
        let initial_caller = self.caller_from_option_or_sender(caller);

        let asset_address = self.get_pool_address(&asset_to_repay);
        require!(
            self.pools_map().contains_key(&asset_to_repay),
            "asset not supported"
        );

        self.liquidity_pool_proxy(asset_address)
            .repay(initial_caller)
            .with_multi_token_transfer(transfers)
            .execute_on_dest_context_ignore_result();
    }

    #[payable("*")]
    #[endpoint(liquidate)]
    fn liquidate(&self, borrow_position_nonce: u64, caller: OptionalValue<ManagedAddress>) {
        let (amount, asset) = self.call_value().payment_token_pair();
        let initial_caller = self.caller_from_option_or_sender(caller);

        self.require_asset_supported(&asset);
        self.require_amount_greater_than_zero(&amount);
        self.require_non_zero_address(&initial_caller);

        let asset_address = self.get_pool_address(&asset);
        let liq_bonus = self.get_liquidation_bonus_non_zero(&asset);

        let lend_tokens: TokenAmountPair<Self::Api> = self
            .liquidity_pool_proxy(asset_address)
            .liquidate(initial_caller, borrow_position_nonce, liq_bonus)
            .add_token_transfer(asset, 0, amount)
            .execute_on_dest_context();

        let lend_tokens_pool = self.get_pool_address(&lend_tokens.token_id);

        self.liquidity_pool_proxy(lend_tokens_pool)
            .reduce_position_after_liquidation()
            .add_token_transfer(lend_tokens.token_id, lend_tokens.nonce, lend_tokens.amount)
            .execute_on_dest_context_ignore_result();
    }

    fn caller_from_option_or_sender(
        &self,
        caller: OptionalValue<ManagedAddress>,
    ) -> ManagedAddress {
        caller
            .into_option()
            .unwrap_or_else(|| self.blockchain().get_caller())
    }

    fn require_asset_supported(&self, asset: &TokenIdentifier) {
        require!(self.pools_map().contains_key(asset), "asset not supported");
    }
}
