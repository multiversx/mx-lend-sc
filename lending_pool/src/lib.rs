#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

pub mod factory;
mod proxy;
pub mod router;
mod storage;
mod utils;
mod math;

pub use common_structs::*;
pub use common_tokens::*;
use elrond_wasm::elrond_codec::Empty;
use liquidity_pool::{liquidity::ProxyTrait as _};

#[elrond_wasm::contract]
pub trait LendingPool:
    factory::FactoryModule
    + router::RouterModule
    + common_checks::ChecksModule
    + common_tokens::AccountTokenModule
    + proxy::ProxyModule
    // + tokens::TokensModule
    // + liq_storage::StorageModule
    + storage::LendingStorageModule
    // + liq_utils::UtilsModule
    + utils::LendingUtilsModule
    // + liq_math::MathModule
    + math::LendingMathModule
    + price_aggregator_proxy::PriceAggregatorModule
{
    #[init]
    fn init(&self, lp_template_address: ManagedAddress) {
        self.liq_pool_template_address().set(&lp_template_address);
        // self.register_account_token(ManagedBuffer::new_from_bytes(ACCOUNT_TOKEN), ManagedBuffer::new_from_bytes(ACCOUNT_TICKER));
    }

    #[endpoint]
    fn enter_market(&self, caller: ManagedAddress) -> u64 {
        // let initial_caller = self.caller_from_option_or_sender(caller);
        let nft_account_amount = BigUint::from(1u64);

        let nft_account_token_id = self.account_token().get_token_id();
        let nft_account_nonce = self.mint_account_token(&Empty);

        // Send NFT to owner
        self.send()
            .direct_esdt(&caller, &nft_account_token_id, nft_account_nonce, &nft_account_amount);

            nft_account_nonce
    }

    #[payable("*")]
    #[endpoint(addCollateral)]
    fn add_collateral(&self, caller: OptionalValue<ManagedAddress>) {
        let [nft_account_token, collateral_payment] = self.call_value().multi_esdt();
        let (nft_account_token_id, nft_account_nonce, nft_account_amount) = nft_account_token.into_tuple();
        let (collateral_token_id, collateral_nonce, collateral_amount) = collateral_payment.into_tuple();
        let initial_caller = self.caller_from_option_or_sender(caller);
        let pool_address = self.get_pool_address(&collateral_token_id);

        self.lending_account_in_the_market(nft_account_nonce);
        self.lending_account_token_valid(nft_account_token_id.clone());
        self.require_amount_greater_than_zero(&collateral_amount);
        self.require_non_zero_address(&initial_caller);

        let initial_deposit_position = self.get_collateral_position_for_token(nft_account_nonce, collateral_token_id.clone());

        let deposit_position = self
            .liquidity_pool_proxy(pool_address)
            .add_collateral(initial_deposit_position)
            .add_esdt_token_transfer(collateral_token_id, collateral_nonce, collateral_amount)
            .execute_on_dest_context();

        self.deposit_position().insert(deposit_position);

        // Return NFT to owner
        self.send().direct_esdt(
            &initial_caller,
            &nft_account_token_id,
            nft_account_nonce,
            &nft_account_amount,
        );
    }

    #[payable("*")]
    #[endpoint(removeCollateral)]
    fn remove_collateral(
        &self,
        withdraw_token_id: TokenIdentifier,
        amount: BigUint,
        caller: OptionalValue<ManagedAddress>,
    ) {
        let (nft_account_token_id, nft_account_nonce, nft_account_amount) =
            self.call_value().single_esdt().into_tuple();
        let initial_caller = self.caller_from_option_or_sender(caller);
        let pool_address = self.get_pool_address(&withdraw_token_id);
        // let mut merged_deposits;

        self.lending_account_in_the_market(nft_account_nonce);
        self.lending_account_token_valid(nft_account_token_id.clone());
        self.require_amount_greater_than_zero(&amount);
        self.require_non_zero_address(&initial_caller);
        require!(
            amount
                > self.get_collateral_available_for_token(nft_account_nonce, nft_account_token_id.clone()),
            "Not enough funds deposited for this account!"
        );

        // Alternative, go through all deposits and withdraw amount
        // merged_deposits = self.merge_deposit_positions(nft_account_nonce, withdraw_token_id);
        let initial_deposit_position = self.get_collateral_position_for_token(nft_account_nonce, withdraw_token_id);
        let deposit_position: DepositPosition<<Self as ContractBase>::Api> = self
            .liquidity_pool_proxy(pool_address)
            .withdraw(&initial_caller, amount, initial_deposit_position)
            .execute_on_dest_context();

        if deposit_position.amount != 0 {
            self.deposit_position().insert(deposit_position);
        }

        // Return NFT to owner
        self.send().direct_esdt(
            &initial_caller,
            &nft_account_token_id,
            nft_account_nonce,
            &nft_account_amount,
        );
    }

    #[payable("*")]
    #[endpoint]
    fn borrow(
        &self,
        asset_to_borrow: TokenIdentifier,
        amount: BigUint,
        caller: OptionalValue<ManagedAddress>,
    ) {
        let (nft_account_token_id, nft_account_nonce, nft_account_amount) =
            self.call_value().single_esdt().into_tuple();
        let initial_caller = self.caller_from_option_or_sender(caller);
        let borrow_token_pool_address = self.get_pool_address(&asset_to_borrow);
        let loan_to_value = self.get_loan_to_value_exists_and_non_zero(&asset_to_borrow);

        self.lending_account_in_the_market(nft_account_nonce);
        self.lending_account_token_valid(nft_account_token_id.clone());
        require!(asset_to_borrow.is_valid_esdt_identifier(), "invalid ticker provided");
        self.require_amount_greater_than_zero(&amount);
        self.require_non_zero_address(&initial_caller);

        self.update_collateral_with_interest(nft_account_nonce);
        self.update_borrows_with_debt(nft_account_nonce);

        let collateral_available =
            self.get_total_borrowed_amount(nft_account_nonce);
        let borrowed_amount_in_dollars =
            self.get_total_borrowed_amount(nft_account_nonce);

        require!(
            collateral_available * loan_to_value > borrowed_amount_in_dollars,
            "Not enough collateral supplied to perform loan!"
        );

        let initial_borrow_position = self.get_borrow_position_for_token(nft_account_nonce, asset_to_borrow);
        let borrow_position = self
            .liquidity_pool_proxy(borrow_token_pool_address)
            .borrow(&initial_caller, amount, initial_borrow_position)
            .execute_on_dest_context();

        self.borrow_position().insert(borrow_position);

                // Return NFT to owner
                self.send().direct_esdt(
                    &initial_caller,
                    &nft_account_token_id,
                    nft_account_nonce,
                    &nft_account_amount,
                );
    }

    #[payable("*")]
    #[endpoint]
    fn repay(
        &self,
        caller: OptionalValue<ManagedAddress>,
    ) {
        let [nft_account_token, payment_repay] = self.call_value().multi_esdt();
        let (nft_account_token_id, nft_account_nonce, nft_account_amount) = nft_account_token.into_tuple();
        let (repay_token_id, repay_nonce, repay_amount) = payment_repay.into_tuple();
        let initial_caller = self.caller_from_option_or_sender(caller);
        let asset_address = self.get_pool_address(&repay_token_id);


        self.lending_account_in_the_market(nft_account_nonce);
        self.lending_account_token_valid(nft_account_token_id.clone());
        self.lending_account_token_valid(repay_token_id.clone());
        self.require_amount_greater_than_zero(&repay_amount);
        self.require_non_zero_address(&initial_caller);

        require!(
            self.pools_map().contains_key(&repay_token_id),
            "asset not supported"
        );

        let initial_borrow_position = self.get_borrow_position_for_token(nft_account_nonce, repay_token_id.clone());

        let borrow_position: BorrowPosition<Self::Api> = self.liquidity_pool_proxy(asset_address)
            .repay(&initial_caller, initial_borrow_position)
            .add_esdt_token_transfer(repay_token_id, repay_nonce, repay_amount)
            .execute_on_dest_context();


            if borrow_position.amount != 0 {
                self.borrow_position().insert(borrow_position);
            }
    
                    // Return NFT to owner
        self.send().direct_esdt(
            &initial_caller,
            &nft_account_token_id,
            nft_account_nonce,
            &nft_account_amount,
        );
    }

    #[payable("*")]
    #[endpoint(liquidate)]
    fn liquidate(&self, liquidatee_account_nonce: u64, liquidation_threshold : BigUint, caller: OptionalValue<ManagedAddress>) {
        let (liquidator_asset_token_id, liquidator_asset_amount) = self.call_value().single_fungible_esdt();
        let initial_caller = self.caller_from_option_or_sender(caller);
        let liq_bonus = self.get_liquidation_bonus_non_zero(&liquidator_asset_token_id);
        let total_collateral_in_dollars = self.get_total_collateral_available(liquidatee_account_nonce);
        let borrowed_value_in_dollars = self.get_total_borrowed_amount(liquidatee_account_nonce);
        let liquidator_asset_data = self.get_token_price_data(liquidator_asset_token_id.clone());
        let liquidator_asset_value_in_dollars = liquidator_asset_amount.clone() * liquidator_asset_data.price; 
        let bp = BigUint::from(BP);

        // Liquidatee is in the market; Liquidator doesn't have to be in the Lending Protocol
        self.lending_account_in_the_market(liquidatee_account_nonce);
        require!(liquidator_asset_token_id.is_valid_esdt_identifier(), "invalid ticker provided");
        self.require_asset_supported(&liquidator_asset_token_id);
        self.require_amount_greater_than_zero(&liquidator_asset_amount);
        self.require_non_zero_address(&initial_caller);
        require!(liquidation_threshold < MAX_THRESHOLD, "Cannot liquidate more than 50% of Liquidatee's position!");

        let health_factor = self.compute_health_factor(
            &total_collateral_in_dollars,
            &borrowed_value_in_dollars,
            &liquidation_threshold,
        );

        require!(health_factor < 1, "health not low enough for liquidation");
        require!(
            liquidator_asset_value_in_dollars >= total_collateral_in_dollars * liquidation_threshold / &bp,
            "insufficient funds for liquidation"
        );

        // amount_liquidated (1 + liq_bonus)
        let amount_to_return_to_liquidator_in_dollars = (liquidator_asset_amount * (&bp + &liq_bonus)) / bp;

        // Go through all DepositPositions and send amount_to_return_in_dollars to Liquidator
        self.send_amount_in_dollars_to_liquidator(
            initial_caller,
            liquidatee_account_nonce,
            amount_to_return_to_liquidator_in_dollars,
        );


        // self.liquidity_pool_proxy(asset_address)
        //     .liquidate(initial_caller, liquidatee_account_nonce, liq_bonus)
        //     .with_egld_or_single_esdt_token_transfer(liquidator_asset, 0, liquidator_asset_amount)
        //     .execute_on_dest_context();
    }


    #[endpoint(updateCollateralWithInterest)]
    fn update_collateral_with_interest(&self, account_position: u64) {
        let deposit_positions = self.deposit_position();
        let deposit_position_iter = deposit_positions
            .iter()
            .filter(|dp| dp.owner_nonce == account_position);

        for dp in deposit_position_iter {
            let asset_address = self.get_pool_address(&dp.token_id);
            self.liquidity_pool_proxy(asset_address)
                .update_collateral_with_interest(dp)
                .execute_on_dest_context_ignore_result();
        }
    }

    #[endpoint(updateBorrowsWithDebt)]
    fn update_borrows_with_debt(&self, account_position: u64) {
        let borrow_positions = self.borrow_position();
        let borrow_position_iter = borrow_positions
            .iter()
            .filter(|bp| bp.owner_nonce == account_position);

        for bp in borrow_position_iter {
            let asset_address = self.get_pool_address(&bp.token_id);
            self.liquidity_pool_proxy(asset_address)
                .update_borrows_with_debt(bp)
                .execute_on_dest_context_ignore_result();
        }
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

    fn lending_account_in_the_market(&self, nonce: u64) {
        require!(
            self.account_positions().contains(&nonce),
            "Account not in Lending Protocol!"
        );
    }
    fn lending_account_token_valid(&self, account_token_id: TokenIdentifier) {
        require!(
            account_token_id == self.account_token().get_token_id(),
            "Account token not valid!"
        );
    }
}
