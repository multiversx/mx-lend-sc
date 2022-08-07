#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

pub mod factory;
mod proxy;
pub mod router;

pub use common_structs::*;
use liquidity_pool::{endpoints::borrowToken, liquidity::ProxyTrait as _};

#[elrond_wasm::contract]
pub trait LendingPool:
    factory::FactoryModule + router::RouterModule + common_checks::ChecksModule + proxy::ProxyModule
{
    #[init]
    fn init(&self, lp_template_address: ManagedAddress) {
        self.liq_pool_template_address().set(&lp_template_address);
    }

    #[endpoint]
    fn enter_market(&self, caller: OptionalValue<ManagedAddress>) -> AccountPosition<Self::Api> {
        let initial_caller = self.caller_from_option_or_sender(caller);

        let account_token_id = TokenIdentifier::from("LAccount"); //change this
        let new_account_nonce = self.mint_account_token(account_token_id);
        // let deposit_positions_array = ManagedVec::from_raw_handle(empty_buffer.get_raw_handle());
        // let borrow_positions_array = ManagedVec::from_raw_handle(empty_buffer.get_raw_handle());

        // let account_position = AccountPosition::new(
        //     // new_account_nonce,
        //     deposit_positions_array,
        //     borrow_positions_array,
        // );

        self.account_list().insert(new_account_nonce);

        self.send().direct(
            &initial_caller,
            &account_token_id,
            new_account_nonce,
            1,
            &[],
        );
    }

    #[payable("*")]
    #[endpoint]
    fn deposit(
        &self,
        caller: OptionalValue<ManagedAddress>,
        account_nonce: u64,
    ) {
        let (amount, asset) = self.call_value().payment_token_pair();
        let initial_caller = self.caller_from_option_or_sender(caller);

        self.require_amount_greater_than_zero(&amount);
        self.require_non_zero_address(&initial_caller);

        let pool_address = self.get_pool_address(&asset);

        self.liquidity_pool_proxy(pool_address, account_nonce)
            .deposit_asset(initial_caller, account_nonce)
            .add_token_transfer(asset, 0, amount)
            .execute_on_dest_context();
    }

    #[payable("*")]
    #[endpoint]
    fn withdraw(
        &self,
        token_id: TokenIdentifier,
        amount: BigUint,
        caller: OptionalValue<ManagedAddress>,
        account_position: AccountPosition<Self::Api>,
    ) {
        // let (amount, lend_token) = self.call_value().payment_token_pair();
        // let token_nonce = self.call_value().esdt_token_nonce();
        let initial_caller = self.caller_from_option_or_sender(caller);

        self.require_amount_greater_than_zero(&amount);
        self.require_non_zero_address(&initial_caller);

        self.supplied_positions(account_position.nonce, token_nonce)
            .get();

        let pool_address = self.get_pool_address(&token_id);
        let returned_deposit_position = self
            .liquidity_pool_proxy(pool_address)
            .withdraw(initial_caller, amount, account_position)
            // .add_token_transfer(lend_token, token_nonce, amount)
            .execute_on_dest_context_ignore_result();

        // if DP withdrawn, delete from array
        // Maybe better do this in Liquidity Pool
        // if !returned_deposit_position && deposit_position {
        //     account_position.supplied_positions.remove(i);
        // }
    }

    #[payable("*")]
    #[endpoint]
    fn borrow(
        &self,
        asset_to_borrow: TokenIdentifier,
        caller: OptionalValue<ManagedAddress>,
        account_position: AccountPosition<Self::Api>,
    ) {
        let (payment_amount, payment_lend_id) = self.call_value().payment_token_pair();
        let payment_nonce = self.call_value().esdt_token_nonce();
        let initial_caller = self.caller_from_option_or_sender(caller);

        self.require_amount_greater_than_zero(&payment_amount);
        self.require_non_zero_address(&initial_caller);

        let borrow_token_pool_address = self.get_pool_address(&asset_to_borrow);
        let loan_to_value = self.get_loan_to_value_exists_and_non_zero(&asset_to_borrow);

        //L tokens for a specific token X are 1:1 with deposited X tokens
        let (borrowTokenPayment, borrowPosition) = self
            .liquidity_pool_proxy(borrow_token_pool_address)
            .borrow(initial_caller, amount, account_position, loan_to_value)
            // .add_token_transfer(payment_lend_id, payment_nonce, payment_amount)
            .execute_on_dest_context();

        // account_position.borrowed_positions.try_push(borrowPosition);

        // borrowTokenPayment
    }

    #[payable("*")]
    #[endpoint]
    fn repay(
        &self,
        asset_to_repay: TokenIdentifier,
        caller: OptionalValue<ManagedAddress>,
        account_position: AccountPosition<Self::Api>,
    ) {
        let (amount, asset) = self.call_value().payment_token_pair();
        let initial_caller = self.caller_from_option_or_sender(caller);

        self.require_amount_greater_than_zero(&amount);
        self.require_non_zero_address(&initial_caller);

        // let asset_address = self.get_pool_address(&asset_to_repay);
        require!(
            self.pools_map().contains_key(&asset_to_repay),
            "asset not supported"
        );

        // for i in 0..account_position.borrowed_positions.len() {
        //     if account_position.borrowed_positions[i]. == token_nonce {
        //         // deposit_position = account_position.supplied_positions[i]
        //         break;
        //     }
        // }

        let borrow_position = self
            .liquidity_pool_proxy(asset_address)
            .repay(initial_caller, account_position)
            .add_token_transfer(asset_to_repay, 0, amount)
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
