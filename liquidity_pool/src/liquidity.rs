elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use common_structs::*;

use super::math;
use super::storage;
use super::tokens;
use super::utils;

const REPAY_PAYMENTS_LEN: usize = 2;

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
    fn deposit_asset(
        &self,
        initial_caller: ManagedAddress,
        account_nonce: u64,
    ) -> DepositPosition<Self::Api> {
        let (amount, asset) = self.call_value().payment_token_pair();
        let pool_asset = self.pool_asset().get();
        require!(
            asset == pool_asset,
            "asset not supported for this liquidity pool"
        );

        // let lend_token_id = self.lend_token().get();
        // let new_token_nonce = self.mint_position_tokens(&lend_token_id, &amount);

        let round = self.blockchain().get_block_round();

        self.update_interest_indexes();
        let supply_index = self.supply_index().get();

        // let lend_tokens =
        //     TokenAmountPair::new(lend_token_id.clone(), new_token_nonce, amount.clone());

        let deposit_position =
            DepositPosition::new(pool_asset, amount, account_nonce, round, supply_index);
        self.deposit_position().insert(deposit_position);

        self.reserves().update(|x| *x += &amount);

        // self.send().direct(
        //     &initial_caller,
        //     &self.lend_token().get(),
        //     new_nonce,
        //     &amount,
        //     &[],
        // );
        deposit_position
    }

    #[only_owner]
    #[payable("*")]
    #[endpoint(reducePositionAfterLiquidation)]
    fn reduce_position_after_liquidation(&self) {
        let (payment_amount, payment_token_id) = self.call_value().payment_token_pair();
        let payment_token_nonce = self.call_value().esdt_token_nonce();
        let lend_token_id = self.lend_token().get();

        require!(
            payment_token_id == lend_token_id,
            "lend tokens not supported by this pool"
        );

        let mut deposit = self.deposit_position(payment_token_nonce).get();
        require!(
            deposit.amount >= payment_amount,
            "payment tokens greater than position size"
        );

        deposit.amount -= &payment_amount;
        if deposit.amount == 0 {
            self.deposit_position(payment_token_nonce).clear();
        } else {
            self.deposit_position(payment_token_nonce).set(&deposit);
        }
    }

    #[only_owner]
    #[payable("*")]
    #[endpoint]
    fn borrow(
        &self,
        initial_caller: ManagedAddress,
        amount: BigUint,
        account_position: u64,
        loan_to_value: BigUint,
    ) -> BorrowPosition<Self::Api> {
        // let (payment_lend_amount, payment_lend_token_id) = self.call_value().payment_token_pair();
        // let payment_lend_token_nonce = self.call_value().esdt_token_nonce();

        self.require_amount_greater_than_zero(&amount);
        self.require_non_zero_address(&initial_caller);
        // let lend_tokens = TokenAmountPair::new(
        //     payment_lend_token_id.clone(),
        //     payment_lend_token_nonce,
        //     payment_lend_amount.clone(),
        // );

        // let borrow_token_id = self.borrow_token().get();
        let pool_token_id = self.pool_asset().get();

        // let collateral_data = self.get_token_price_data_lending(&payment_lend_token_id);
        let pool_asset_data = self.get_token_price_data(&pool_token_id);

        let borrow_amount_in_dollars =
            self.get_collateral_available(account_position, loan_to_value);

        let borrow_amount_in_tokens = (&borrow_amount_in_dollars / &pool_asset_data.price)
            * BigUint::from(10u64).pow(pool_asset_data.decimals as u32);

        let asset_reserve = self.reserves().get();

        require!(
            asset_reserve >= borrow_amount_in_tokens,
            "insufficient funds to perform loan"
        );

        self.update_interest_indexes();

        // let new_nonce = self.mint_position_tokens(&borrow_token_id, &borrow_amount_in_tokens);
        let round = self.blockchain().get_block_round();
        let borrow_index = self.borrow_index().get();
        let borrow_position =
            BorrowPosition::new(pool_token_id, amount, account_position, round, borrow_index);

        self.borrow_position().insert(borrow_position);

        self.borrowed_amount()
            .update(|total| *total += &borrow_amount_in_tokens);

        self.reserves()
            .update(|total| *total -= &borrow_amount_in_tokens);

        // self.send().direct(
        //     &initial_caller,
        //     &borrow_token_id,
        //     new_nonce,
        //     &borrow_amount_in_tokens,
        //     &[],
        // );
        // self.send().direct(
        //     &initial_caller,
        //     &pool_token_id,
        //     0,
        //     &borrow_amount_in_tokens,
        //     &[],
        // );
        borrow_position
    }

    #[only_owner]
    #[payable("*")]
    #[endpoint]
    fn withdraw(&self, initial_caller: ManagedAddress, amount: BigUint, account_position: u64) {
        // let (amount, lend_token) = self.call_value().payment_token_pair();
        // let token_nonce = self.call_value().esdt_token_nonce();

        let pool_asset = self.pool_asset().get();

        let deposit = self.merge_deposit_positions(account_position);

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

        let interest = &withdrawal_amount - &amount;

        self.rewards_reserves_accumulated_not_distributed()
            .update(|rewards| {
                require!(*rewards >= interest, "rewards accumulated not sufficient");

                *rewards -= interest;
            });

        deposit.amount -= &amount;
        self.deposit_position().swap_remove(&deposit);
        if deposit.amount != 0 {
            self.deposit_position().insert(deposit);
        }

        // self.send()
        //     .esdt_local_burn(&lend_token, token_nonce, &amount);

        self.send()
            .direct(&initial_caller, &pool_asset, 0, &withdrawal_amount, &[]);

        // self.deposit_position(token_nonce)
    }

    #[only_owner]
    #[payable("*")]
    #[endpoint]
    fn repay(
        &self,
        initial_caller: ManagedAddress,
        account_position: u64,
    ) -> BorrowPosition<Self::Api> {
        let (amount_to_be_repaid, asset_token_id) = self.call_value().payment_token_pair();
        let pool_asset = self.pool_asset().get();
        require!(
            asset_token_id == pool_asset,
            "asset not supported for this liquidity pool"
        );
        self.require_non_zero_address(&initial_caller);

        // let borrow_token_id = &first_payment.token_identifier;
        // let borrow_token_nonce = first_payment.token_nonce;
        // let borrow_token_amount = &first_payment.amount;

        // let asset_token_id = &second_payment.token_identifier;
        // let asset_amount = &second_payment.amount;

        let mut borrow_position = self.merge_borrow_positions(account_position);

        // require!(
        //     borrow_position.is_empty(),
        //     "liquidated position"
        // );
        // let mut borrow_position = self.borrow_position(borrow_token_nonce).get();

        self.update_interest_indexes();

        let accumulated_debt = self.get_debt_interest(
            &borrow_position.amount,
            &borrow_position.initial_borrow_index,
        );
        let total_owed = borrow_position.amount + &accumulated_debt;

        if amount_to_be_repaid > total_owed {
            let extra_asset_paid = amount_to_be_repaid - &total_owed;
            self.send()
                .direct(&initial_caller, &asset_token_id, 0, &extra_asset_paid, &[]);
        }

        // let lend_token_amount_to_send_back: BigUint;
        self.borrow_position().swap_remove(&borrow_position);
        if !self.is_full_repay(&borrow_position, &amount_to_be_repaid) {
            // // Issue here:
            // lend_token_amount_to_send_back = self.rule_of_three(
            //     &borrow_position.lend_tokens.amount,
            //     amount_to_be_repaid,
            //     &borrow_position.amount,
            // );

            // require!(
            //     lend_token_amount_to_send_back > 0,
            //     "repay too little. lend tokens amount is zero"
            // );

            borrow_position.amount -= amount_to_be_repaid;
            // borrow_position.lend_tokens.amount -= &lend_token_amount_to_send_back;
            self.borrow_position().insert(borrow_position);
        }

        self.borrowed_amount()
            .update(|total| *total -= amount_to_be_repaid);

        self.reserves().update(|total| *total += &total_owed);

        // self.send()
        //     .esdt_local_burn(borrow_token_id, borrow_token_nonce, borrow_token_amount);

        // self.send().direct(
        //     &initial_caller,
        //     &borrow_position.lend_tokens.token_id,
        //     borrow_position.lend_tokens.nonce,
        //     &lend_token_amount_to_send_back,
        //     &[],
        // );

        borrow_position
    }

    #[only_owner]
    #[payable("*")]
    #[endpoint]
    fn liquidate(
        &self,
        initial_caller: ManagedAddress,
        borrow_position_nonce: u64,
        liquidation_bonus: BigUint,
    ) -> TokenAmountPair<Self::Api> {
        let (asset_amount, asset_token_id) = self.call_value().payment_token_pair();

        self.require_non_zero_address(&initial_caller);
        self.require_amount_greater_than_zero(&asset_amount);

        require!(
            asset_token_id == self.pool_asset().get(),
            "asset is not supported by this pool"
        );

        require!(
            !self.borrow_position(borrow_position_nonce).is_empty(),
            "position was repaid or already liquidated"
        );

        let borrow_position = self.borrow_position(borrow_position_nonce).get();
        let collateral_token_id = borrow_position.collateral_token_id.clone();

        let base_big = BigUint::from(10u64);

        let asset_price_data = self.get_token_price_data(&asset_token_id);
        let asset_price_decs = base_big.pow(asset_price_data.decimals as u32);

        let collateral_price_data = self.get_token_price_data_lending(&collateral_token_id);
        let collateral_price_decs = base_big.pow(collateral_price_data.decimals as u32);

        let collateral_amount = borrow_position.lend_tokens.amount.clone();
        let collateral_value_in_dollars =
            (collateral_amount * collateral_price_data.price.clone()) / collateral_price_decs;

        let borrowed_value_in_dollars =
            (&asset_amount * &asset_price_data.price) / asset_price_decs;

        let liquidation_threshold = self.liquidation_threshold().get();
        let health_factor = self.compute_health_factor(
            &collateral_value_in_dollars,
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

        self.borrowed_amount()
            .update(|total| *total -= &borrow_position.borrowed_amount);

        self.reserves().update(|total| *total += &asset_amount);

        self.borrow_position(borrow_position_nonce).clear();

        let lend_amount_to_return_in_dollars = (asset_amount * (&bp + &liquidation_bonus)) / bp;
        let lend_amount_to_return = (&lend_amount_to_return_in_dollars * &asset_price_data.price)
            / &collateral_price_data.price;
        let lend_tokens = borrow_position.lend_tokens.clone();

        require!(
            lend_tokens.amount >= lend_amount_to_return,
            "total amount to return bigger than position"
        );

        self.send().direct(
            &initial_caller,
            &borrow_position.lend_tokens.token_id,
            borrow_position.lend_tokens.nonce,
            &lend_amount_to_return,
            &[],
        );

        TokenAmountPair::new(
            lend_tokens.token_id,
            lend_tokens.nonce,
            lend_amount_to_return,
        )
    }
}
