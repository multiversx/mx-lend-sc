elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use common_structs::*;

use super::liq_math;
use super::liq_storage;
use super::liq_utils;
use super::tokens;

#[elrond_wasm::module]
pub trait LiquidityModule:
    liq_storage::StorageModule
    + tokens::TokensModule
    + common_tokens::AccountTokenModule
    + liq_utils::UtilsModule
    + liq_math::MathModule
    + price_aggregator_proxy::PriceAggregatorModule
    + common_checks::ChecksModule
{
    #[only_owner]
    #[payable("*")]
    #[endpoint(updateCollateralWithInterest)]
    fn update_collateral_with_interest(
        &self,
        mut deposit_position: DepositPosition<Self::Api>,
    ) -> DepositPosition<Self::Api> {
        let round = self.blockchain().get_block_round();
        let supply_index = self.supply_index().get();

        self.update_interest_indexes();

        let accrued_interest = self.compute_interest(
            &deposit_position.amount,
            &supply_index,
            &deposit_position.initial_supply_index,
        );

        deposit_position.amount += accrued_interest;
        deposit_position.round = round;
        deposit_position.initial_supply_index = supply_index;

        deposit_position
    }

    #[only_owner]
    #[payable("*")]
    #[endpoint(updateBorrowsWithDebt)]
    fn update_borrows_with_debt(
        &self,
        mut borrow_position: BorrowPosition<Self::Api>,
    ) -> BorrowPosition<Self::Api> {
        let round = self.blockchain().get_block_round();
        let borrow_index = self.borrow_index().get();

        self.update_interest_indexes();

        let accumulated_debt = self.get_debt_interest(
            &borrow_position.amount,
            &borrow_position.initial_borrow_index,
        );

        borrow_position.amount += accumulated_debt;
        borrow_position.round = round;
        borrow_position.initial_borrow_index = borrow_index;

        borrow_position
    }

    #[only_owner]
    #[payable("*")]
    #[endpoint(addCollateral)]
    fn add_collateral(
        &self,
        deposit_position: DepositPosition<Self::Api>,
    ) -> DepositPosition<Self::Api> {
        let (deposit_asset, deposit_amount) = self.call_value().single_fungible_esdt();
        let pool_asset = self.pool_asset().get();
        let round = self.blockchain().get_block_round();
        let supply_index = self.supply_index().get();
        let mut ret_deposit_position = deposit_position.clone();

        require!(
            deposit_asset == pool_asset,
            "asset not supported for this liquidity pool"
        );

        self.update_interest_indexes();

        // Update DepositPosition
        if deposit_position.amount != 0 {
            ret_deposit_position = self.update_collateral_with_interest(deposit_position);
        }
        ret_deposit_position.amount += &deposit_amount;
        ret_deposit_position.round = round;
        ret_deposit_position.initial_supply_index = supply_index;

        // let deposit_position = DepositPosition::new(
        //     pool_asset,
        //     deposit_amount.clone(),
        //     account_nonce,
        //     round,
        //     supply_index,
        // );
        // self.deposit_position().insert(deposit_position);

        self.reserves().update(|x| *x += deposit_amount);
        ret_deposit_position
    }

    #[only_owner]
    #[payable("*")]
    #[endpoint]
    fn borrow(
        &self,
        initial_caller: ManagedAddress,
        borrow_amount: BigUint,
        existing_borrow_position: BorrowPosition<Self::Api>,
        // loan_to_value: BigUint,
    ) -> BorrowPosition<Self::Api> {
        let pool_token_id = self.pool_asset().get();
        // let collateral_amount = self.get_collateral_available(account_position);
        // let borrowable_amount_in_dollars = self.compute_borrowable_amount(
        //     &collateral_amount,
        //     &loan_to_value,
        //     pool_asset_data.decimals,
        // );
        // let borrowable_amount_in_tokens = (&borrowable_amount_in_dollars / &pool_asset_data.price)
        //     * BigUint::from(10u64).pow(pool_asset_data.decimals as u32);
        // let borrow_amount_in_tokens = cmp::min(borrowable_amount_in_tokens, amount);
        let asset_reserve = self.reserves().get();
        let mut ret_borrow_position = existing_borrow_position.clone();
        self.require_non_zero_address(&initial_caller);
        require!(
            asset_reserve >= borrow_amount,
            "insufficient funds to perform loan"
        );

        self.update_interest_indexes();
        if ret_borrow_position.amount != 0 {
            ret_borrow_position = self.update_borrows_with_debt(existing_borrow_position);
        }

        let round = self.blockchain().get_block_round();
        let borrow_index = self.borrow_index().get();
        ret_borrow_position.amount += &borrow_amount;
        ret_borrow_position.round = round;
        ret_borrow_position.initial_borrow_index = borrow_index;

        // self.borrow_position().insert(borrow_position);

        self.borrowed_amount()
            .update(|total| *total += &borrow_amount);

        self.reserves().update(|total| *total -= &borrow_amount);

        self.send()
            .direct_esdt(&initial_caller, &pool_token_id, 0, &borrow_amount);

        ret_borrow_position
    }

    #[only_owner]
    #[payable("*")]
    #[endpoint]
    fn remove_collateral(
        &self,
        initial_caller: ManagedAddress,
        amount: BigUint,
        mut deposit_position: DepositPosition<Self::Api>,
    ) -> DepositPosition<Self::Api> {
        let pool_asset = self.pool_asset().get();

        self.require_non_zero_address(&initial_caller);
        self.require_amount_greater_than_zero(&amount);

        self.update_interest_indexes();

        // Withdrawal amount = initial_deposit + Interest
        let withdrawal_amount = self.compute_withdrawal_amount(
            &amount,
            &self.supply_index().get(),
            &deposit_position.initial_supply_index,
        );

        self.reserves().update(|asset_reserve| {
            require!(*asset_reserve >= withdrawal_amount, "insufficient funds");
            *asset_reserve -= &withdrawal_amount;
        });

        deposit_position.amount -= &amount;

        self.send()
            .direct_esdt(&initial_caller, &pool_asset, 0, &withdrawal_amount);

        deposit_position
    }

    #[only_owner]
    #[payable("*")]
    #[endpoint]
    fn repay(
        &self,
        initial_caller: ManagedAddress,
        borrow_position: BorrowPosition<Self::Api>,
    ) -> BorrowPosition<Self::Api> {
        let (repay_asset, mut repay_amount) = self.call_value().single_fungible_esdt();
        let pool_asset = self.pool_asset().get();
        let initial_borrowed_amount = borrow_position.amount.clone();

        self.require_non_zero_address(&initial_caller);
        self.require_amount_greater_than_zero(&repay_amount);
        require!(
            repay_asset == pool_asset,
            "asset not supported for this liquidity pool"
        );

        // self.update_interest_indexes();
        let mut ret_borrow_position = self.update_borrows_with_debt(borrow_position);

        let total_owed_with_interest = ret_borrow_position.amount.clone();

        if repay_amount >= total_owed_with_interest {
            let extra_amount = &repay_amount - &total_owed_with_interest;
            self.send()
                .direct_esdt(&initial_caller, &repay_asset, 0, &extra_amount);
            ret_borrow_position.amount = BigUint::zero();
            repay_amount = total_owed_with_interest.clone();
        } else {
            ret_borrow_position.amount -= &repay_amount;
        }

        self.borrowed_amount()
            .update(|total| *total -= initial_borrowed_amount);

        self.reserves()
            .update(|total| *total += &total_owed_with_interest);

        ret_borrow_position
    }

    #[only_owner]
    #[endpoint(sendTokens)]
    fn send_tokens(&self, initial_caller: ManagedAddress, payment_amount: BigUint) {
        let pool_asset = self.pool_asset().get();

        self.send()
            .direct_esdt(&initial_caller, &pool_asset, 0, &payment_amount);
    }
    /*

    #[only_owner]
    #[payable("*")]
    #[endpoint]
    fn liquidate(
        &self,
        initial_caller: ManagedAddress,
        liquidatee_account_nonce: u64,
        liquidation_bonus: BigUint,
    ) {
        let (liquidator_asset, liquidator_asset_amount) = self.call_value().single_fungible_esdt();

        self.require_non_zero_address(&initial_caller);
        self.require_amount_greater_than_zero(&liquidator_asset_amount);

        require!(
            liquidator_asset == self.pool_asset().get(),
            "asset is not supported by this pool"
        );

        // let collateral_value_in_dollars = self.get_collateral_available(account_position);
        // let collateral_amount_in_dollars = self.get_collateral_available(account_position);
        // let borrowed_value_in_dollars = self.get_total_borrowed_amount(account_position);

        // let liquidation_threshold = self.liquidation_threshold().get();
        // let health_factor = self.compute_health_factor(
        //     &collateral_amount_in_dollars,
        //     &borrowed_value_in_dollars,
        //     &liquidation_threshold,
        // );

        // let bp = self.get_base_precision();

        // require!(health_factor < 1, "health not low enough for liquidation");
        // require!(
        //     asset_amount >= collateral_value_in_dollars * liquidation_threshold / &bp,
        //     "insufficient funds for liquidation"
        // );

        self.update_interest_indexes();

        // Total borrowed amount is covered/paid by the liquidator with asset_amount
        self.borrowed_amount()
            .update(|total| *total -= &asset_amount);

        self.reserves().update(|total| *total += &liquidator_asset_amount);

        let amount_to_return_in_dollars = (asset_amount * (&bp + &liquidation_bonus)) / bp;

        // Go through all DepositPositions and send amount_to_return_in_dollars to Liquidator
        self.send_amount_in_dollars_to_liquidator(
            initial_caller,
            account_position,
            amount_to_return_in_dollars,
        );
    }

    */
}
