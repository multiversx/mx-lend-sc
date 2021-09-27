elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use common_structs::*;

use super::library;
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
    + library::LibraryModule
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

        self.interest_metadata(new_nonce).set(&InterestMetadata {
            timestamp: self.blockchain().get_block_timestamp(),
        });

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
    #[endpoint(borrow)]
    fn borrow(
        &self,
        initial_caller: Address,
        collateral_token_id: TokenIdentifier,
        collateral_amount: Self::BigUint,
        deposit_timestamp: u64,
        ltv: Self::BigUint,
    ) -> SCResult<()> {
        self.require_amount_greater_than_zero(&collateral_amount)?;
        self.require_non_zero_address(&initial_caller)?;

        let borrow_token_id = self.borrow_token().get();
        let pool_token_id = self.pool_asset().get();

        let collateral_data = self.get_token_price_data(&collateral_token_id)?;
        let pool_asset_data = self.get_token_price_data(&pool_token_id)?;

        let borrowable_amount = self.compute_borrowable_amount(
            &collateral_amount,
            &collateral_data.price,
            &ltv,
            collateral_data.decimals,
        );

        let pool_asset_dec_big = Self::BigUint::from(pool_asset_data.decimals as u64);
        let total_borrowable = (&borrowable_amount / &pool_asset_data.price) / pool_asset_dec_big;
        let asset_reserve = self.reserves(&pool_token_id).get();

        require!(
            asset_reserve < total_borrowable,
            "insufficient funds to perform loan"
        );

        let new_nonce = self.mint_position_tokens(&borrow_token_id, &collateral_amount);

        self.debt_metadata(new_nonce).set(&DebtMetadata {
            timestamp: self.blockchain().get_block_timestamp(),
            collateral_amount: collateral_amount.clone(),
            collateral_identifier: collateral_token_id,
            collateral_timestamp: deposit_timestamp,
        });

        self.borrowed_amount()
            .update(|total| *total += &total_borrowable);

        self.reserves(&pool_token_id)
            .update(|total| *total -= &borrowable_amount);

        self.send().direct(
            &initial_caller,
            &borrow_token_id,
            new_nonce,
            &collateral_amount,
            &[],
        );

        self.send()
            .direct(&initial_caller, &pool_token_id, 0, &borrowable_amount, &[]);

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
        let metadata = self.interest_metadata(token_nonce).get();

        let deposit_rate = self.get_deposit_rate();
        let time_diff = self.get_timestamp_diff(metadata.timestamp)?;
        let withdrawal_amount =
            self.compute_withdrawal_amount(&amount, &time_diff.into(), &deposit_rate);

        self.reserves(&pool_asset).update(|asset_reserve| {
            require!(*asset_reserve >= withdrawal_amount, "insufficient funds");
            *asset_reserve -= &withdrawal_amount;
            Ok(())
        })?;

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
    fn repay(
        &self,
        initial_caller: Address,
    ) -> SCResult<MultiResult3<TokenIdentifier, Self::BigUint, u64>> {
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

        let esdt_nft_data = self.blockchain().get_esdt_token_data(
            &self.blockchain().get_sc_address(),
            borrow_token_id,
            borrow_token_nonce,
        );

        let debt_position_id = &esdt_nft_data.hash;
        require!(
            self.debt_positions().contains_key(debt_position_id),
            "invalid debt position"
        );

        let mut debt_position = self
            .debt_positions()
            .get(debt_position_id)
            .unwrap_or_default();

        require!(!debt_position.is_liquidated, "position is liquidated");

        let debt_metadata = esdt_nft_data.decode_attributes::<DebtMetadata<Self::BigUint>>()?;

        let accumulated_debt =
            self.get_debt_interest(borrow_token_amount, debt_metadata.timestamp)?;
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

        // TODO: Instead of borrow_token_amount (i.e. 1:1 ratio), calculate how much collateral amount was repaid
        debt_position.collateral_amount -= borrow_token_amount;
        if debt_position.collateral_amount == 0 {
            self.debt_positions().remove(debt_position_id);
        } else {
            let _ = self
                .debt_positions()
                .insert(debt_position_id.clone(), debt_position.clone());
        }

        self.send()
            .esdt_local_burn(borrow_token_id, borrow_token_nonce, borrow_token_amount);

        // Same here, use calculated amount of repaid collateral instead of borrow_token_amount
        Ok((
            debt_position.collateral_identifier,
            borrow_token_amount.clone(),
            debt_position.timestamp,
        )
            .into())
    }

    #[only_owner]
    #[payable("*")]
    #[endpoint]
    fn liquidate(
        &self,
        position_id: BoxedBytes,
        #[payment_token] token: TokenIdentifier,
        #[payment_amount] amount: Self::BigUint,
    ) -> SCResult<LiquidateData<Self::BigUint>> {
        require!(amount > 0, "amount must be bigger then 0");
        require!(
            token == self.pool_asset().get(),
            "asset is not supported by this pool"
        );

        let mut debt_position = self.debt_positions().get(&position_id).unwrap_or_default();

        require!(
            debt_position != DebtPosition::default(),
            "invalid debt position id"
        );
        require!(
            !debt_position.is_liquidated,
            "position is already liquidated"
        );
        require!(
            debt_position.health_factor < self.health_factor_threshold().get(),
            "the health factor is not low enough"
        );

        let interest = self.get_debt_interest(&debt_position.size, debt_position.timestamp)?;

        require!(
            debt_position.size.clone() + interest == amount,
            "position can't be liquidated, not enough or to much tokens send"
        );

        debt_position.is_liquidated = true;

        self.debt_positions()
            .insert(position_id, debt_position.clone());

        let liquidate_data = LiquidateData {
            collateral_token: debt_position.collateral_identifier,
            amount: debt_position.size,
        };

        Ok(liquidate_data)
    }
}
