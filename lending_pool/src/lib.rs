#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

mod factory;
mod proxy;
mod router;

pub use common_structs::*;

use liquidity_pool::multi_transfer;

use liquidity_pool::liquidity::ProxyTrait as _;
use liquidity_pool::tokens::ProxyTrait as _;

#[elrond_wasm::contract]
pub trait LendingPool:
    factory::FactoryModule
    + router::RouterModule
    + multi_transfer::MultiTransferModule
    + common_checks::ChecksModule
    + proxy::ProxyModule
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
        self.require_non_zero_address(&initial_caller)?;

        let pool_address = self.get_pool_address_non_zero(&asset)?;
        self.require_non_zero_address(&pool_address)?;

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
        self.require_non_zero_address(&initial_caller)?;

        let pool_address = self.get_pool_address(&lend_token);
        self.require_non_zero_address(&pool_address)?;

        self.liquidity_pool_proxy(pool_address)
            .withdraw(initial_caller, lend_token, token_nonce, amount)
            .execute_on_dest_context();

        Ok(())
    }

    #[payable("*")]
    #[endpoint(borrow)]
    fn borrow_endpoint(
        &self,
        #[payment_token] payment_lend_id: TokenIdentifier,
        #[payment_nonce] payment_nonce: u64,
        #[payment_amount] payment_amount: Self::BigUint,
        collateral_token_id: TokenIdentifier,
        asset_to_borrow: TokenIdentifier,
        #[var_args] caller: OptionalArg<Address>,
    ) -> SCResult<()> {
        let initial_caller = self.caller_from_option_or_sender(caller);

        self.require_amount_greater_than_zero(&payment_amount)?;
        self.require_non_zero_address(&initial_caller)?;

        let borrow_token_pool_address = self.get_pool_address_non_zero(&asset_to_borrow)?;
        let lend_token_pool_address = self.get_pool_address_non_zero(&payment_lend_id)?;

        let metadata = self
            .liquidity_pool_proxy(lend_token_pool_address.clone())
            .get_interest_metadata(payment_nonce)
            .execute_on_dest_context();

        let ltv = self.get_ltv_exists_and_non_zero(&collateral_token_id)?;

        self.liquidity_pool_proxy(borrow_token_pool_address)
            .borrow(
                initial_caller.clone(),
                collateral_token_id,
                payment_amount.clone(),
                metadata.timestamp,
                ltv,
            )
            .execute_on_dest_context_ignore_result();

        self.liquidity_pool_proxy(lend_token_pool_address)
            .burn_l_tokens(
                payment_lend_id,
                payment_nonce,
                payment_amount,
                initial_caller,
            )
            .execute_on_dest_context_ignore_result();

        Ok(())
    }

    #[payable("*")]
    #[endpoint(repay)]
    fn repay_endpoint(
        &self,
        asset_to_repay: TokenIdentifier,
        #[var_args] caller: OptionalArg<Address>,
    ) -> SCResult<()> {
        let initial_caller = self.caller_from_option_or_sender(caller);

        let asset_address = self.get_pool_address(&asset_to_repay);
        require!(
            self.pools_map().contains_key(&asset_to_repay),
            "asset not supported"
        );

        // TODO: Use SC Proxy instead of manual call in 0.19.0

        let transfers = self.get_all_esdt_transfers();
        let raw_results = self.multi_transfer_via_execute_on_dest_context(
            &asset_address,
            &transfers,
            &b"repay"[..].into(),
            &[initial_caller.as_bytes().into()],
        );

        let collateral_id = TokenIdentifier::top_decode(raw_results[0].as_slice())?;
        let collateral_amount_repaid = Self::BigUint::top_decode(raw_results[1].as_slice())?;
        let borrow_timestamp = u64::top_decode(raw_results[2].as_slice())?;

        let collateral_token_address = self.get_pool_address(&collateral_id);
        require!(
            !collateral_token_address.is_zero(),
            "collateral not supported"
        );

        self.liquidity_pool_proxy(collateral_token_address)
            .mint_l_tokens(
                initial_caller,
                collateral_id,
                collateral_amount_repaid,
                borrow_timestamp,
            )
            .execute_on_dest_context_ignore_result();

        Ok(())
    }

    #[payable("*")]
    #[endpoint(liquidate)]
    fn liquidate_endpoint(
        &self,
        #[payment_token] asset: TokenIdentifier,
        #[payment_amount] amount: Self::BigUint,
        liquidate_unique_id: BoxedBytes,
        #[var_args] caller: OptionalArg<Address>,
    ) -> SCResult<()> {
        let initial_caller = self.caller_from_option_or_sender(caller);

        self.require_asset_supported(&asset)?;
        self.require_amount_greater_than_zero(&amount)?;
        self.require_non_zero_address(&initial_caller)?;

        let asset_address = self.get_pool_address(&asset);

        let results = self
            .liquidity_pool_proxy(asset_address)
            .liquidate(liquidate_unique_id, asset, amount)
            .execute_on_dest_context();

        let collateral_token_address = self.get_pool_address(&results.collateral_token);
        self.require_non_zero_address(&collateral_token_address)?;

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

    fn caller_from_option_or_sender(&self, caller: OptionalArg<Address>) -> Address {
        caller
            .into_option()
            .unwrap_or_else(|| self.blockchain().get_caller())
    }

    fn require_asset_supported(&self, asset: &TokenIdentifier) -> SCResult<()> {
        require!(self.pools_map().contains_key(asset), "asset not supported");

        Ok(())
    }
}
