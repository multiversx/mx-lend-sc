#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

pub mod factory;
mod proxy;
pub mod router;

pub use common_structs::*;
use liquidity_pool::{liquidity::ProxyTrait as _, math, storage, tokens, utils};

#[elrond_wasm::contract]
pub trait LendingPool:
    factory::FactoryModule
    + router::RouterModule
    + common_checks::ChecksModule
    + proxy::ProxyModule
    + tokens::TokensModule
    + storage::StorageModule
    + utils::UtilsModule
    + math::MathModule
    + price_aggregator_proxy::PriceAggregatorModule
{
    #[init]
    fn init(&self, lp_template_address: ManagedAddress) {
        self.liq_pool_template_address().set(&lp_template_address);
    }

    // #[endpoint]
    // fn enter_market(&self, caller: OptionalValue<ManagedAddress>) -> u64 {
    //     let initial_caller = self.caller_from_option_or_sender(caller);

    //     let account_token_id = TokenIdentifier::from(ACCOUNT_TOKEN);
    //     let new_account_nonce = self.mint_account_token(account_token_id);

    //     self.account_list().nft_add_quantity_and_send(
    //         &initial_caller,
    //         new_account_nonce,
    //         BigUint::from(1u64),
    //     );

    //     new_account_nonce
    // }

    #[payable("*")]
    #[endpoint]
    fn deposit(&self, caller: OptionalValue<ManagedAddress>, account_nonce: u64) {
        let (asset, amount) = self.call_value().egld_or_single_fungible_esdt();
        let initial_caller = self.caller_from_option_or_sender(caller);

        self.require_amount_greater_than_zero(&amount);
        self.require_non_zero_address(&initial_caller);

        let pool_address = self.get_pool_address(&asset);

        self.liquidity_pool_proxy(pool_address)
            .deposit_asset(account_nonce)
            .with_egld_or_single_esdt_token_transfer(asset, 0, amount)
            .execute_on_dest_context_ignore_result();
    }

    #[payable("*")]
    #[endpoint]
    fn withdraw(
        &self,
        token_id: EgldOrEsdtTokenIdentifier,
        amount: BigUint,
        account_position: u64,
        caller: OptionalValue<ManagedAddress>,
    ) {
        let initial_caller = self.caller_from_option_or_sender(caller);

        self.require_amount_greater_than_zero(&amount);
        self.require_non_zero_address(&initial_caller);

        let pool_address = self.get_pool_address(&token_id);
        self.liquidity_pool_proxy(pool_address)
            .withdraw(initial_caller, amount, account_position)
            .execute_on_dest_context_ignore_result();
    }

    #[payable("*")]
    #[endpoint]
    fn borrow(
        &self,
        asset_to_borrow: EgldOrEsdtTokenIdentifier,
        amount: BigUint,
        account_position: u64,
        caller: OptionalValue<ManagedAddress>,
    ) {
        let initial_caller = self.caller_from_option_or_sender(caller);

        self.require_amount_greater_than_zero(&amount);
        self.require_non_zero_address(&initial_caller);

        let borrow_token_pool_address = self.get_pool_address(&asset_to_borrow);
        let loan_to_value = self.get_loan_to_value_exists_and_non_zero(&asset_to_borrow);

        self.liquidity_pool_proxy(borrow_token_pool_address)
            .borrow(amount, account_position, loan_to_value)
            .execute_on_dest_context_ignore_result();
    }

    #[payable("*")]
    #[endpoint]
    fn repay(
        &self,
        asset_to_repay: EgldOrEsdtTokenIdentifier,
        account_position: u64,
        caller: OptionalValue<ManagedAddress>,
    ) {
        let (asset, amount) = self.call_value().egld_or_single_fungible_esdt();
        let initial_caller = self.caller_from_option_or_sender(caller);

        self.require_amount_greater_than_zero(&amount);
        self.require_non_zero_address(&initial_caller);

        let asset_address = self.get_pool_address(&asset);
        require!(
            self.pools_map().contains_key(&asset_to_repay),
            "asset not supported"
        );

        self.liquidity_pool_proxy(asset_address)
            .repay(initial_caller, account_position)
            .with_egld_or_single_esdt_token_transfer(asset_to_repay, 0, amount)
            .execute_on_dest_context_ignore_result();
    }

    #[payable("*")]
    #[endpoint(liquidate)]
    fn liquidate(&self, account_position: u64, caller: OptionalValue<ManagedAddress>) {
        let (asset, amount) = self.call_value().egld_or_single_fungible_esdt();
        let initial_caller = self.caller_from_option_or_sender(caller);

        self.require_asset_supported(&asset);
        self.require_amount_greater_than_zero(&amount);
        self.require_non_zero_address(&initial_caller);

        let asset_address = self.get_pool_address(&asset);
        let liq_bonus = self.get_liquidation_bonus_non_zero(&asset);

        self.liquidity_pool_proxy(asset_address)
            .liquidate(initial_caller, account_position, liq_bonus)
            .with_egld_or_single_esdt_token_transfer(asset, 0, amount)
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

    fn require_asset_supported(&self, asset: &EgldOrEsdtTokenIdentifier) {
        require!(self.pools_map().contains_key(asset), "asset not supported");
    }
}
