#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

mod factory;
mod proxy;
mod router;

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
    fn deposit(
        &self,
        #[payment_token] asset: TokenIdentifier,
        #[payment_amount] amount: BigUint,
        #[var_args] caller: OptionalArg<ManagedAddress>,
        #[var_args] accept_funds_func: OptionalArg<ManagedBuffer>,
    ) -> SCResult<()> {
        let initial_caller = self.caller_from_option_or_sender(caller);

        self.require_amount_greater_than_zero(&amount)?;
        self.require_non_zero_address(&initial_caller)?;

        let pool_address = self.get_pool_address_non_zero(&asset)?;
        self.require_non_zero_address(&pool_address)?;

        self.liquidity_pool_proxy(pool_address)
            .deposit_asset(initial_caller, asset, amount, accept_funds_func)
            .execute_on_dest_context();

        Ok(())
    }

    #[payable("*")]
    #[endpoint]
    fn withdraw(
        &self,
        #[payment_token] lend_token: TokenIdentifier,
        #[payment_nonce] token_nonce: u64,
        #[payment_amount] amount: BigUint,
        #[var_args] caller: OptionalArg<ManagedAddress>,
        #[var_args] accept_funds_func: OptionalArg<ManagedBuffer>,
    ) -> SCResult<()> {
        let initial_caller = self.caller_from_option_or_sender(caller);

        self.require_amount_greater_than_zero(&amount)?;
        self.require_non_zero_address(&initial_caller)?;

        let pool_address = self.get_pool_address(&lend_token);
        self.require_non_zero_address(&pool_address)?;

        self.liquidity_pool_proxy(pool_address)
            .withdraw(
                initial_caller,
                lend_token,
                token_nonce,
                amount,
                accept_funds_func,
            )
            .execute_on_dest_context();

        Ok(())
    }

    #[payable("*")]
    #[endpoint]
    fn borrow(
        &self,
        #[payment_token] payment_lend_id: TokenIdentifier,
        #[payment_nonce] payment_nonce: u64,
        #[payment_amount] payment_amount: BigUint,
        collateral_token_id: TokenIdentifier,
        asset_to_borrow: TokenIdentifier,
        #[var_args] caller: OptionalArg<ManagedAddress>,
        #[var_args] accept_funds_func: OptionalArg<ManagedBuffer>,
    ) -> SCResult<()> {
        let initial_caller = self.caller_from_option_or_sender(caller);

        self.require_amount_greater_than_zero(&payment_amount)?;
        self.require_non_zero_address(&initial_caller)?;

        let borrow_token_pool_address = self.get_pool_address_non_zero(&asset_to_borrow)?;
        let loan_to_value = self.get_loan_to_value_exists_and_non_zero(&collateral_token_id)?;

        //L tokens for a specific token X are 1:1 with deposited X tokens
        let collateral_amount = payment_amount.clone();
        let collateral_tokens = TokenAmountPair::new(collateral_token_id, 0, collateral_amount);
        self.liquidity_pool_proxy(borrow_token_pool_address)
            .borrow(
                payment_lend_id,
                payment_nonce,
                payment_amount,
                initial_caller,
                collateral_tokens,
                loan_to_value,
                accept_funds_func,
            )
            .execute_on_dest_context_ignore_result();

        Ok(())
    }

    #[payable("*")]
    #[endpoint]
    fn repay(
        &self,
        asset_to_repay: TokenIdentifier,
        #[var_args] caller: OptionalArg<ManagedAddress>,
        #[var_args] accept_funds_func: OptionalArg<ManagedBuffer>,
    ) -> SCResult<()> {
        let initial_caller = self.caller_from_option_or_sender(caller);

        let asset_address = self.get_pool_address(&asset_to_repay);
        require!(
            self.pools_map().contains_key(&asset_to_repay),
            "asset not supported"
        );

        let transfers = self.raw_vm_api().get_all_esdt_transfers();
        self.liquidity_pool_proxy(asset_address)
            .repay(initial_caller, accept_funds_func)
            .with_multi_token_transfer(transfers)
            .execute_on_dest_context();

        Ok(())
    }

    #[payable("*")]
    #[endpoint(liquidate)]
    fn liquidate(
        &self,
        #[payment_token] asset: TokenIdentifier,
        #[payment_amount] amount: BigUint,
        borrow_position_nonce: u64,
        #[var_args] caller: OptionalArg<ManagedAddress>,
        #[var_args] accept_funds_func: OptionalArg<ManagedBuffer>,
    ) -> SCResult<()> {
        let initial_caller = self.caller_from_option_or_sender(caller);

        self.require_asset_supported(&asset)?;
        self.require_amount_greater_than_zero(&amount)?;
        self.require_non_zero_address(&initial_caller)?;

        let asset_address = self.get_pool_address(&asset);
        let liq_bonus = self.get_liquidation_bonus_non_zero(&asset)?;

        let lend_tokens = self
            .liquidity_pool_proxy(asset_address)
            .liquidate(
                asset,
                amount,
                initial_caller,
                borrow_position_nonce,
                liq_bonus,
                accept_funds_func,
            )
            .execute_on_dest_context();

        let lend_tokens_pool = self.get_pool_address(&lend_tokens.token_id);

        self.liquidity_pool_proxy(lend_tokens_pool)
            .reduce_position_after_liquidation(
                lend_tokens.token_id,
                lend_tokens.nonce,
                lend_tokens.amount,
            )
            .execute_on_dest_context_ignore_result();

        Ok(())
    }

    fn caller_from_option_or_sender(&self, caller: OptionalArg<ManagedAddress>) -> ManagedAddress {
        caller
            .into_option()
            .unwrap_or_else(|| self.blockchain().get_caller())
    }

    fn require_asset_supported(&self, asset: &TokenIdentifier) -> SCResult<()> {
        require!(self.pools_map().contains_key(asset), "asset not supported");

        Ok(())
    }
}
