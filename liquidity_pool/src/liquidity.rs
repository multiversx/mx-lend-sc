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
    + token_send::TokenSendModule
{
    #[only_owner]
    #[payable("*")]
    #[endpoint(depositAsset)]
    fn deposit_asset(
        &self,
        #[payment_token] asset: TokenIdentifier,
        #[payment_amount] amount: BigUint,
        initial_caller: ManagedAddress,
        #[var_args] accept_funds_func: OptionalArg<ManagedBuffer>,
    ) {
        let pool_asset = self.pool_asset().get();
        require!(
            asset == pool_asset,
            "asset not supported for this liquidity pool"
        );

        let lend_token_id = self.lend_token().get();
        let new_nonce = self.mint_position_tokens(&lend_token_id, &amount);

        let timestamp = self.blockchain().get_block_timestamp();
        let deposit_position = DepositPosition::new(timestamp, amount.clone());
        self.deposit_position(new_nonce).set(&deposit_position);

        self.reserves().update(|x| *x += &amount);

        self.send_nft_tokens(
            &initial_caller,
            &self.lend_token().get(),
            new_nonce,
            &amount,
            &accept_funds_func,
        );
    }

    #[only_owner]
    #[payable("*")]
    #[endpoint(reducePositionAfterLiquidation)]
    fn reduce_position_after_liquidation(
        &self,
        #[payment_token] payment_token_id: TokenIdentifier,
        #[payment_nonce] payment_token_nonce: u64,
        #[payment_amount] payment_amount: BigUint,
    ) {
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
        #[payment_token] payment_lend_token_id: TokenIdentifier,
        #[payment_nonce] payment_lend_token_nonce: u64,
        #[payment_amount] payment_lend_amount: BigUint,
        initial_caller: ManagedAddress,
        collateral_tokens: TokenAmountPair<Self::Api>,
        loan_to_value: BigUint,
        #[var_args] accept_funds_func: OptionalArg<ManagedBuffer>,
    ) {
        self.require_amount_greater_than_zero(&collateral_tokens.amount);
        self.require_non_zero_address(&initial_caller);
        let lend_tokens = TokenAmountPair::new(
            payment_lend_token_id,
            payment_lend_token_nonce,
            payment_lend_amount,
        );

        let borrow_token_id = self.borrow_token().get();
        let pool_token_id = self.pool_asset().get();

        let collateral_data = self.get_token_price_data(&collateral_tokens.token_id);
        let pool_asset_data = self.get_token_price_data(&pool_token_id);

        let borrow_amount_in_dollars = self.compute_borrowable_amount(
            &collateral_tokens.amount,
            &collateral_data.price,
            &loan_to_value,
            collateral_data.decimals,
        );

        let borrow_amount_in_tokens = (&borrow_amount_in_dollars / &pool_asset_data.price)
            * BigUint::from(10u64).pow(pool_asset_data.decimals as u32);

        let asset_reserve = self.reserves().get();

        require!(
            asset_reserve > borrow_amount_in_tokens,
            "insufficient funds to perform loan"
        );

        let new_nonce = self.mint_position_tokens(&borrow_token_id, &borrow_amount_in_tokens);
        let timestamp = self.blockchain().get_block_timestamp();
        let borrow_position = BorrowPosition::new(
            timestamp,
            lend_tokens,
            borrow_amount_in_tokens.clone(),
            collateral_tokens.token_id,
        );

        self.borrow_position(new_nonce).set(&borrow_position);

        self.borrowed_amount()
            .update(|total| *total += &borrow_amount_in_tokens);

        self.reserves()
            .update(|total| *total -= &borrow_amount_in_tokens);

        self.send_nft_tokens(
            &initial_caller,
            &borrow_token_id,
            new_nonce,
            &borrow_amount_in_tokens,
            &accept_funds_func,
        );
        self.send_fft_tokens(
            &initial_caller,
            &pool_token_id,
            &borrow_amount_in_tokens,
            &accept_funds_func,
        );
    }

    #[only_owner]
    #[payable("*")]
    #[endpoint]
    fn withdraw(
        &self,
        initial_caller: ManagedAddress,
        #[payment_token] lend_token: TokenIdentifier,
        #[payment_nonce] token_nonce: u64,
        #[payment_amount] amount: BigUint,
        #[var_args] accept_funds_func: OptionalArg<ManagedBuffer>,
    ) {
        require!(
            lend_token == self.lend_token().get(),
            "lend token not supported"
        );

        let pool_asset = self.pool_asset().get();
        let mut deposit = self.deposit_position(token_nonce).get();

        let deposit_rate = self.get_deposit_rate();
        let time_diff = self.get_timestamp_diff(deposit.timestamp);
        let withdrawal_amount =
            self.compute_withdrawal_amount(&amount, &BigUint::from(time_diff), &deposit_rate);

        self.reserves().update(|asset_reserve| {
            require!(*asset_reserve >= withdrawal_amount, "insufficient funds");
            *asset_reserve -= &withdrawal_amount;
        });

        deposit.amount -= &amount;
        if deposit.amount == 0 {
            self.deposit_position(token_nonce).clear();
        } else {
            self.deposit_position(token_nonce).set(&deposit);
        }

        self.send()
            .esdt_local_burn(&lend_token, token_nonce, &amount);

        self.send_fft_tokens(
            &initial_caller,
            &pool_asset,
            &withdrawal_amount,
            &accept_funds_func,
        );
    }

    #[only_owner]
    #[payable("*")]
    #[endpoint]
    fn repay(
        &self,
        initial_caller: ManagedAddress,
        #[var_args] accept_funds_func: OptionalArg<ManagedBuffer>,
    ) {
        self.require_non_zero_address(&initial_caller);

        let transfers = self
            .call_value()
            .all_esdt_transfers()
            .into_iter()
            .collect::<Vec<EsdtTokenPayment<Self::Api>>>();

        require!(
            transfers.len() == REPAY_PAYMENTS_LEN,
            "Invalid number of payments"
        );
        require!(
            transfers[0].token_identifier == self.borrow_token().get(),
            "First payment should be the borrow SFTs"
        );
        require!(
            transfers[1].token_identifier == self.pool_asset().get(),
            "Second payment should be this pool's asset"
        );

        let borrow_token_id = &transfers[0].token_identifier;
        let borrow_token_nonce = transfers[0].token_nonce;
        let borrow_token_amount = &transfers[0].amount;

        let asset_token_id = &transfers[1].token_identifier;
        let asset_amount = &transfers[1].amount;

        require!(
            !self.borrow_position(borrow_token_nonce).is_empty(),
            "liquidated position"
        );
        let mut borrow_position = self.borrow_position(borrow_token_nonce).get();

        let accumulated_debt =
            self.get_debt_interest(borrow_token_amount, borrow_position.timestamp);
        let total_owed = borrow_token_amount + &accumulated_debt;

        require!(
            asset_amount >= &total_owed,
            "Not enough asset tokens deposited"
        );

        if asset_amount > &total_owed {
            let extra_asset_paid = asset_amount - &total_owed;
            self.send_fft_tokens(
                &initial_caller,
                asset_token_id,
                &extra_asset_paid,
                &accept_funds_func,
            );
        }

        let lend_token_amount_to_send_back: BigUint;
        if self.is_full_repay(&borrow_position, borrow_token_amount) {
            lend_token_amount_to_send_back = borrow_position.lend_tokens.amount;
            self.borrow_position(borrow_token_nonce).clear();
        } else {
            lend_token_amount_to_send_back = self.rule_of_three(
                &borrow_position.lend_tokens.amount,
                borrow_token_amount,
                &borrow_position.borrowed_amount,
            );

            require!(
                lend_token_amount_to_send_back > 0,
                "repay too little. lend tokens amount is zero"
            );

            borrow_position.borrowed_amount -= borrow_token_amount;
            borrow_position.lend_tokens.amount -= &lend_token_amount_to_send_back;
            self.borrow_position(borrow_token_nonce)
                .set(&borrow_position);
        }

        self.borrowed_amount()
            .update(|total| *total -= borrow_token_amount);

        self.reserves().update(|total| *total += &total_owed);

        self.send()
            .esdt_local_burn(borrow_token_id, borrow_token_nonce, borrow_token_amount);

        self.send_nft_tokens(
            &initial_caller,
            &borrow_position.lend_tokens.token_id,
            borrow_position.lend_tokens.nonce,
            &lend_token_amount_to_send_back,
            &accept_funds_func,
        );
    }

    #[only_owner]
    #[payable("*")]
    #[endpoint]
    fn liquidate(
        &self,
        #[payment_token] asset_token_id: TokenIdentifier,
        #[payment_amount] asset_amount: BigUint,
        initial_caller: ManagedAddress,
        borrow_position_nonce: u64,
        liquidation_bonus: BigUint,
        #[var_args] accept_funds_func: OptionalArg<ManagedBuffer>,
    ) -> TokenAmountPair<Self::Api> {
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

        let collateral_price_data = self.get_token_price_data(&collateral_token_id);
        let collateral_price_decs = base_big.pow(collateral_price_data.decimals as u32);

        let collateral_amount = borrow_position.lend_tokens.amount.clone();
        let collateral_value_in_dollars =
            (collateral_amount * collateral_price_data.price) / collateral_price_decs;

        let borrowed_value_in_dollars =
            (&asset_amount * &asset_price_data.price) / asset_price_decs;

        let liquidation_threshold = self.liquidation_threshold().get();
        let health_factor = self.compute_health_factor(
            &collateral_value_in_dollars,
            &borrowed_value_in_dollars,
            &liquidation_threshold,
        );

        require!(health_factor < 1, "health not low enough for liquidation");
        require!(
            asset_amount >= borrow_position.borrowed_amount,
            "insufficient funds for liquidation"
        );

        self.borrowed_amount()
            .update(|total| *total -= &borrow_position.borrowed_amount);

        self.reserves().update(|total| *total += &asset_amount);

        self.borrow_position(borrow_position_nonce).clear();

        let bp = self.get_base_precision();
        let lend_amount_to_return = (asset_amount * (&bp + &liquidation_bonus)) / bp;
        let lend_tokens = borrow_position.lend_tokens.clone();
        require!(
            lend_tokens.amount > lend_amount_to_return,
            "total amount to return bigger than position"
        );

        self.send_nft_tokens(
            &initial_caller,
            &borrow_position.lend_tokens.token_id,
            borrow_position.lend_tokens.nonce,
            &lend_amount_to_return,
            &accept_funds_func,
        );

        let remaining_amount = lend_tokens.amount - lend_amount_to_return;
        let lend_token_pair =
            TokenAmountPair::new(lend_tokens.token_id, lend_tokens.nonce, remaining_amount);

        lend_token_pair
    }
}
