elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use common_structs::*;

use super::math;
use super::multi_transfer;
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
    + multi_transfer::MultiTransferModule
    + common_checks::ChecksModule
{
    #[only_owner]
    #[payable("*")]
    #[endpoint(depositAsset)]
    fn deposit_asset(
        &self,
        initial_caller: Address,
        #[payment_token] asset: TokenIdentifier,
        #[payment_amount] amount: Self::BigUint,
    ) -> SCResult<()> {
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

        self.reserves(&pool_asset).update(|x| *x += &amount);

        self.send().direct(
            &initial_caller,
            &self.lend_token().get(),
            new_nonce,
            &amount,
            &[],
        );

        Ok(())
    }

    #[only_owner]
    #[endpoint]
    fn borrow(
        &self,
        initial_caller: Address,
        lend_tokens: TokenAmountPair<Self::BigUint>,
        collateral_tokens: TokenAmountPair<Self::BigUint>,
        ltv: Self::BigUint,
    ) -> SCResult<()> {
        self.require_amount_greater_than_zero(collateral_tokens.get_amount_as_ref())?;
        self.require_non_zero_address(&initial_caller)?;

        let borrow_token_id = self.borrow_token().get();
        let pool_token_id = self.pool_asset().get();

        let collateral_data = self.get_token_price_data(collateral_tokens.get_token_id_as_ref())?;
        let pool_asset_data = self.get_token_price_data(&pool_token_id)?;

        let borrow_amount_in_dollars = self.compute_borrowable_amount(
            collateral_tokens.get_amount_as_ref(),
            &collateral_data.price,
            &ltv,
            collateral_data.decimals,
        );

        let pool_asset_dec_big = Self::BigUint::from(pool_asset_data.decimals as u64);

        let borrow_amount_in_tokens =
            (&borrow_amount_in_dollars / &pool_asset_data.price) / pool_asset_dec_big;

        let asset_reserve = self.reserves(&pool_token_id).get();

        require!(
            asset_reserve > borrow_amount_in_tokens,
            "insufficient funds to perform loan"
        );

        let new_nonce =
            self.mint_position_tokens(&borrow_token_id, lend_tokens.get_amount_as_ref());

        let lend_tokens_amount = lend_tokens.get_amount();
        let timestamp = self.blockchain().get_block_timestamp();
        let borrow_position = BorrowPosition::new(timestamp, lend_tokens);
        self.borrow_position(new_nonce).set(&borrow_position);

        self.borrowed_amount()
            .update(|total| *total += &borrow_amount_in_tokens);

        self.reserves(&pool_token_id)
            .update(|total| *total -= &borrow_amount_in_tokens);

        self.send().direct(
            &initial_caller,
            &borrow_token_id,
            new_nonce,
            &lend_tokens_amount,
            &[],
        );

        self.send().direct(
            &initial_caller,
            &pool_token_id,
            0,
            &borrow_amount_in_tokens,
            &[],
        );

        Ok(())
    }

    #[only_owner]
    #[payable("*")]
    #[endpoint]
    fn withdraw(
        &self,
        initial_caller: Address,
        #[payment_token] lend_token: TokenIdentifier,
        #[payment_nonce] token_nonce: u64,
        #[payment_amount] amount: Self::BigUint,
    ) -> SCResult<()> {
        require!(
            lend_token == self.lend_token().get(),
            "lend token not supported"
        );

        let pool_asset = self.pool_asset().get();
        let mut deposit = self.deposit_position(token_nonce).get();

        let deposit_rate = self.get_deposit_rate();
        let time_diff = self.get_timestamp_diff(deposit.timestamp)?;
        let withdrawal_amount =
            self.compute_withdrawal_amount(&amount, &time_diff.into(), &deposit_rate);

        self.reserves(&pool_asset).update(|asset_reserve| {
            require!(*asset_reserve >= withdrawal_amount, "insufficient funds");
            *asset_reserve -= &withdrawal_amount;
            Ok(())
        })?;

        deposit.amount -= &amount;
        if deposit.amount == 0 {
            self.deposit_position(token_nonce).clear();
        } else {
            self.deposit_position(token_nonce).set(&deposit);
        }

        self.send()
            .esdt_local_burn(&lend_token, token_nonce, &amount);

        self.send()
            .direct(&initial_caller, &pool_asset, 0, &withdrawal_amount, &[]);

        Ok(())
    }

    // Returns the asset token ID, the amount of debt paid
    #[only_owner]
    #[payable("*")]
    #[endpoint]
    fn repay(&self, initial_caller: Address) -> SCResult<()> {
        self.require_non_zero_address(&initial_caller)?;

        let transfers = self.get_all_esdt_transfers();

        require!(
            transfers.len() == REPAY_PAYMENTS_LEN,
            "Invalid number of payments"
        );
        require!(
            transfers[0].token_name == self.borrow_token().get(),
            "First payment should be the borrow SFTs"
        );
        require!(
            transfers[1].token_name == self.pool_asset().get(),
            "Second payment should be this pool's asset"
        );

        let borrow_token_id = &transfers[0].token_name;
        let borrow_token_nonce = transfers[0].token_nonce;
        let borrow_token_amount = &transfers[0].amount;

        let asset_token_id = &transfers[1].token_name;
        let asset_amount = &transfers[1].amount;

        require!(
            !self.borrow_position(borrow_token_nonce).is_empty(),
            "liquidated position"
        );
        let mut borrow_position = self.borrow_position(borrow_token_nonce).get();

        let accumulated_debt =
            self.get_debt_interest(borrow_token_amount, borrow_position.timestamp)?;
        let total_owed = borrow_token_amount + &accumulated_debt;

        require!(
            asset_amount >= &total_owed,
            "Not enough asset tokens deposited"
        );

        let extra_asset_paid = asset_amount - &total_owed;
        if extra_asset_paid > 0 {
            self.send()
                .direct(&initial_caller, asset_token_id, 0, &extra_asset_paid, &[]);
        }

        borrow_position.lend_tokens.amount -= borrow_token_amount;
        if borrow_position.lend_tokens.amount == 0 {
            self.borrow_position(borrow_token_nonce).clear();
        } else {
            self.borrow_position(borrow_token_nonce)
                .set(&borrow_position);
        }

        self.send()
            .esdt_local_burn(borrow_token_id, borrow_token_nonce, borrow_token_amount);

        self.send().direct(
            &initial_caller,
            borrow_position.lend_tokens.get_token_id_as_ref(),
            borrow_position.lend_tokens.get_token_nonce(),
            &borrow_token_amount,
            &[],
        );

        Ok(())
    }

    #[only_owner]
    #[payable("*")]
    #[endpoint]
    fn liquidate(
        &self,
        borrow_position_nonce: u64,
        initial_caller: Address,
        #[payment_token] token: TokenIdentifier,
        #[payment_amount] amount: Self::BigUint,
    ) -> SCResult<()> {
        self.require_non_zero_address(&initial_caller)?;
        require!(amount > 0, "amount must be bigger then 0");
        require!(
            token == self.pool_asset().get(),
            "asset is not supported by this pool"
        );

        require!(
            !self.borrow_position(borrow_position_nonce).is_empty(),
            "Position is empty"
        );
        let borrow_position = self.borrow_position(borrow_position_nonce).get();

        // TODO: do the actual computation here.
        // require!(
        //     debt_position.health_factor < self.health_factor_threshold().get(),
        //     "the health factor is not low enough"
        // );

        //TODO: do the checks against Liquidation Threshold.
        self.send().direct(
            &initial_caller,
            borrow_position.lend_tokens.get_token_id_as_ref(),
            borrow_position.lend_tokens.get_token_nonce(),
            borrow_position.lend_tokens.get_amount_as_ref(),
            &[],
        );

        Ok(())
    }
}
