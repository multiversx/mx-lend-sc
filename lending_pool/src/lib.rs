#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

mod factory;
mod proxy_common;
mod router;

pub use common_structs::*;

use liquidity_pool::liquidity::ProxyTrait as _;
use liquidity_pool::tokens::ProxyTrait as _;

#[elrond_wasm::contract]
pub trait LendingPool:
    factory::FactoryModule + router::RouterModule + proxy_common::ProxyCommonModule
{
    #[init]
    fn init(&self) {}

    #[payable("*")]
    #[endpoint(deposit)]
    fn deposit_endpoint(
        &self,
        #[payment_token] asset: TokenIdentifier,
        #[payment_amount] amount: Self::BigUint,
        #[var_args] caller: OptionalArg<Address>,
    ) -> SCResult<()> {
        let initial_caller = self.caller_from_option_or_sender(caller);

        self.require_amount_greater_than_zero(&amount)?;
        self.require_valid_address_provided(&initial_caller)?;

        let pool_address = self.get_pool_address(&asset);
        require!(!pool_address.is_zero(), "invalid liquidity pool address");

        self.liquidity_pool_proxy(pool_address)
            .deposit_asset(initial_caller, asset, amount)
            .execute_on_dest_context();

        Ok(())
    }

    #[payable("*")]
    #[endpoint(withdraw)]
    fn withdraw_endpoint(
        &self,
        #[payment_token] lend_token: TokenIdentifier,
        #[payment_nonce] token_nonce: u64,
        #[payment_amount] amount: Self::BigUint,
        #[var_args] caller: OptionalArg<Address>,
    ) -> SCResult<()> {
        let initial_caller = self.caller_from_option_or_sender(caller);

        self.require_amount_greater_than_zero(&amount)?;
        self.require_valid_address_provided(&initial_caller)?;

        let pool_address = self.get_pool_address(&lend_token);
        require!(!pool_address.is_zero(), "invalid liquidity pool address");

        self.liquidity_pool_proxy(pool_address)
            .withdraw(initial_caller, lend_token, token_nonce, amount)
            .execute_on_dest_context();

        Ok(())
    }

    #[payable("*")]
    #[endpoint(borrow)]
    fn borrow_endpoint(
        &self,
        #[payment_token] lend_token_id: TokenIdentifier,
        #[payment_nonce] lend_token_nonce: u64,
        #[payment_amount] amount: Self::BigUint,
        asset_to_borrow: TokenIdentifier,
        asset_collateral: TokenIdentifier,
        #[var_args] caller: OptionalArg<Address>,
    ) -> SCResult<()> {
        let initial_caller = self.caller_from_option_or_sender(caller);

        self.require_amount_greater_than_zero(&amount)?;
        self.require_valid_address_provided(&initial_caller)?;

        require!(lend_token_nonce != 0, "lend token can not have nonce zero");

        let borrow_token_pool_address = self.get_pool_address_non_zero(&asset_to_borrow)?;
        let lend_token_pool_address = self.get_pool_address_non_zero(&lend_token_id)?;

        let metadata = self.get_interest_metadata(&lend_token_id, lend_token_nonce)?;

        self.liquidity_pool_proxy(borrow_token_pool_address)
            .borrow(
                initial_caller.clone(),
                asset_collateral,
                amount.clone(),
                metadata.timestamp,
            )
            .execute_on_dest_context_ignore_result();

        self.liquidity_pool_proxy(lend_token_pool_address)
            .burn_l_tokens(lend_token_id, lend_token_nonce, amount, initial_caller)
            .execute_on_dest_context_ignore_result();

        Ok(())
    }

    #[payable("*")]
    #[endpoint(lockBTokens)]
    fn lock_b_tokens_endpoint(
        &self,
        asset_to_repay: TokenIdentifier,
        #[var_args] caller: OptionalArg<Address>,
        #[payment_token] borrow_token: TokenIdentifier,
        #[payment_nonce] token_nonce: u64,
        #[payment_amount] amount: Self::BigUint,
    ) -> SCResult<()> {
        let initial_caller = self.caller_from_option_or_sender(caller);

        self.require_asset_supported(&asset_to_repay)?;
        self.require_amount_greater_than_zero(&amount)?;
        self.require_valid_address_provided(&initial_caller)?;

        let asset_address = self.get_pool_address(&asset_to_repay);

        self.liquidity_pool_proxy(asset_address)
            .lock_b_tokens(initial_caller, borrow_token, token_nonce, amount)
            .execute_on_dest_context_ignore_result();

        Ok(())
    }

    #[payable("*")]
    #[endpoint(repay)]
    fn repay_endpoint(
        &self,
        repay_unique_id: BoxedBytes,
        #[var_args] caller: OptionalArg<Address>,
        #[payment_token] asset: TokenIdentifier,
        #[payment_amount] amount: Self::BigUint,
    ) -> SCResult<()> {
        let initial_caller = self.caller_from_option_or_sender(caller);

        self.require_asset_supported(&asset)?;
        self.require_amount_greater_than_zero(&amount)?;
        self.require_valid_address_provided(&initial_caller)?;

        let asset_address = self.get_pool_address(&asset);

        let results = self
            .liquidity_pool_proxy(asset_address)
            .repay(repay_unique_id, asset, amount)
            .execute_on_dest_context();

        let collateral_token_address = self.get_pool_address(&results.collateral_identifier);

        require!(
            collateral_token_address != Address::zero(),
            "asset is not supported"
        );

        self.liquidity_pool_proxy(collateral_token_address)
            .mint_l_tokens(
                initial_caller,
                results.collateral_identifier,
                results.amount,
                results.collateral_timestamp,
            )
            .execute_on_dest_context_ignore_result();

        Ok(())
    }

    #[payable("*")]
    #[endpoint(liquidate)]
    fn liquidate_endpoint(
        &self,
        liquidate_unique_id: BoxedBytes,
        #[var_args] caller: OptionalArg<Address>,
        #[payment_token] asset: TokenIdentifier,
        #[payment_amount] amount: Self::BigUint,
    ) -> SCResult<()> {
        let initial_caller = self.caller_from_option_or_sender(caller);

        self.require_asset_supported(&asset)?;
        self.require_amount_greater_than_zero(&amount)?;
        self.require_valid_address_provided(&initial_caller)?;

        let asset_address = self.get_pool_address(&asset);

        let results = self
            .liquidity_pool_proxy(asset_address)
            .liquidate(liquidate_unique_id, asset, amount)
            .execute_on_dest_context();

        let collateral_token_address = self.get_pool_address(&results.collateral_token);

        require!(
            collateral_token_address != Address::zero(),
            "asset is not supported"
        );

        self.liquidity_pool_proxy(collateral_token_address)
            .mint_l_tokens(
                initial_caller,
                results.collateral_token,
                results.amount,
                self.blockchain().get_block_timestamp(),
            )
            .execute_on_dest_context_ignore_result();

        Ok(())
    }

    fn get_interest_metadata(
        &self,
        token_id: &TokenIdentifier,
        nonce: u64,
    ) -> SCResult<InterestMetadata> {
        let esdt_nft_data = self.blockchain().get_esdt_token_data(
            &self.blockchain().get_sc_address(),
            token_id,
            nonce,
        );
        esdt_nft_data.decode_attributes().into()
    }

    fn caller_from_option_or_sender(&self, caller: OptionalArg<Address>) -> Address {
        caller
            .into_option()
            .unwrap_or_else(|| self.blockchain().get_caller())
    }

    fn require_amount_greater_than_zero(&self, amount: &Self::BigUint) -> SCResult<()> {
        require!(amount > &0, "amount must be greater than 0");

        Ok(())
    }

    fn require_valid_address_provided(&self, caller: &Address) -> SCResult<()> {
        require!(!caller.is_zero(), "invalid address provided");

        Ok(())
    }

    fn require_asset_supported(&self, asset: &TokenIdentifier) -> SCResult<()> {
        require!(self.pools_map().contains_key(asset), "asset not supported");

        Ok(())
    }
}
