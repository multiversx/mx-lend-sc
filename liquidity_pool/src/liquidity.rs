elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use core::cmp;

use common_structs::*;

use super::math;
use super::storage;
use super::tokens;
use super::utils;

#[elrond_wasm::module]
pub trait LiquidityModule:
    storage::StorageModule
    + tokens::TokensModule
    + utils::UtilsModule
    + math::MathModule
    + price_aggregator_proxy::PriceAggregatorModule
    + common_checks::ChecksModule
{
    #[only_owner]
    #[payable("*")]
    #[endpoint(depositAsset)]
    fn deposit_asset(&self, account_nonce: u64) {
        let (asset, amount) = self.call_value().egld_or_single_fungible_esdt();
        let pool_asset = self.pool_asset().get();
        let round = self.blockchain().get_block_round();
        let supply_index = self.supply_index().get();

        require!(
            asset == pool_asset,
            "asset not supported for this liquidity pool"
        );

        self.update_interest_indexes();

        let deposit_position = DepositPosition::new(
            pool_asset,
            amount.clone(),
            account_nonce,
            round,
            supply_index,
        );
        self.deposit_position().insert(deposit_position);

        self.reserves().update(|x| *x += amount);
    }

    #[only_owner]
    #[payable("*")]
    #[endpoint]
    fn borrow(&self, amount: BigUint, account_position: u64, loan_to_value: BigUint) {
        let pool_token_id = self.pool_asset().get();
        let pool_asset_data = self.get_token_price_data(pool_token_id.clone());
        let collateral_amount = self.get_collateral_available(account_position);
        let borrowable_amount_in_dollars = self.compute_borrowable_amount(
            &collateral_amount,
            &loan_to_value,
            pool_asset_data.decimals,
        );
        let borrowable_amount_in_tokens = (&borrowable_amount_in_dollars / &pool_asset_data.price)
            * BigUint::from(10u64).pow(pool_asset_data.decimals as u32);
        let borrow_amount_in_tokens = cmp::min(borrowable_amount_in_tokens, amount);
        let asset_reserve = self.reserves().get();

        require!(
            asset_reserve >= borrow_amount_in_tokens,
            "insufficient funds to perform loan"
        );

        self.update_interest_indexes();

        let round = self.blockchain().get_block_round();
        let borrow_index = self.borrow_index().get();
        let borrow_position = BorrowPosition::new(
            pool_token_id,
            borrow_amount_in_tokens.clone(),
            account_position,
            round,
            borrow_index,
        );

        self.borrow_position().insert(borrow_position);

        self.borrowed_amount()
            .update(|total| *total += &borrow_amount_in_tokens);

        self.reserves()
            .update(|total| *total -= &borrow_amount_in_tokens);

    }

    #[only_owner]
    #[payable("*")]
    #[endpoint]
    fn withdraw(&self, initial_caller: ManagedAddress, amount: BigUint, account_position: u64) {
        let pool_asset = self.pool_asset().get();
        let mut deposit = self.merge_deposit_positions(account_position);

        self.update_interest_indexes();

        let withdrawal_amount = self.compute_withdrawal_amount(
            &amount,
            &self.supply_index().get(),
            &deposit.initial_supply_index,
        );

        self.reserves().update(|asset_reserve| {
            require!(*asset_reserve >= withdrawal_amount, "insufficient funds");
            *asset_reserve -= &withdrawal_amount;
        });

        deposit.amount -= &amount;
        self.deposit_position().swap_remove(&deposit);
        if deposit.amount != 0 {
            self.deposit_position().insert(deposit);
        }

        self.send()
            .direct(&initial_caller, &pool_asset, 0, &withdrawal_amount);
    }

    #[only_owner]
    #[payable("*")]
    #[endpoint]
    fn repay(&self, initial_caller: ManagedAddress, account_position: u64) {
        let (asset_token_id, amount_to_be_repaid) = self.call_value().egld_or_single_fungible_esdt();
        let pool_asset = self.pool_asset().get();
        let mut borrow_position = self.merge_borrow_positions(account_position);

        require!(
            asset_token_id == pool_asset,
            "asset not supported for this liquidity pool"
        );
        self.require_non_zero_address(&initial_caller);

        self.update_interest_indexes();

        let accumulated_debt = self.get_debt_interest(
            &borrow_position.amount,
            &borrow_position.initial_borrow_index,
        );
        let total_owed = &borrow_position.amount + &accumulated_debt;

        if amount_to_be_repaid > total_owed {
            let extra_asset_paid = &amount_to_be_repaid - &total_owed;
            self.send()
                .direct(&initial_caller, &asset_token_id, 0, &extra_asset_paid);
        }

        self.borrow_position().swap_remove(&borrow_position);
        if !self.is_full_repay(&borrow_position, &amount_to_be_repaid) {
            borrow_position.amount -= &amount_to_be_repaid;
            self.borrow_position().insert(borrow_position.clone());
        }

        self.borrowed_amount()
            .update(|total| *total -= amount_to_be_repaid);

        self.reserves().update(|total| *total += &total_owed);
    }

    #[only_owner]
    #[payable("*")]
    #[endpoint]
    fn liquidate(
        &self,
        initial_caller: ManagedAddress,
        account_position: u64,
        liquidation_bonus: BigUint,
    ) {
        let (asset_token_id, asset_amount) = self.call_value().egld_or_single_fungible_esdt();

        self.require_non_zero_address(&initial_caller);
        self.require_amount_greater_than_zero(&asset_amount);

        require!(
            asset_token_id == self.pool_asset().get(),
            "asset is not supported by this pool"
        );

        let collateral_value_in_dollars = self.get_collateral_available(account_position);
        let collateral_amount_in_dollars = self.get_collateral_available(account_position);
        let borrowed_value_in_dollars = self.get_total_borrowed_amount(account_position);

        let liquidation_threshold = self.liquidation_threshold().get();
        let health_factor = self.compute_health_factor(
            &collateral_amount_in_dollars,
            &borrowed_value_in_dollars,
            &liquidation_threshold,
        );

        let bp = self.get_base_precision();

        require!(health_factor < 1, "health not low enough for liquidation");
        require!(
            asset_amount >= collateral_value_in_dollars * liquidation_threshold / &bp,
            "insufficient funds for liquidation"
        );

        self.update_interest_indexes();

        // Total borrowed amount is covered/paid by the liquidator with asset_amount
        self.borrowed_amount()
            .update(|total| *total -= &asset_amount);

        self.reserves().update(|total| *total += &asset_amount);

        let amount_to_return_in_dollars = (asset_amount * (&bp + &liquidation_bonus)) / bp;

        // Go through all DepositPositions and send amount_to_return_in_dollars to Liquidator
        self.send_amount_in_dollars_to_liquidator(
            initial_caller,
            account_position,
            amount_to_return_in_dollars,
        );
    }
}
