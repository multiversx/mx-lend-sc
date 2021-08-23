elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use crate::{DebtMetadata, DebtPosition, InterestMetadata, LiquidateData, RepayPostion};

#[elrond_wasm::module]
pub trait LiquidityModule:
    crate::storage::StorageModule
    + crate::tokens::TokensModule
    + crate::utils::UtilsModule
    + crate::library::LibraryModule
{
    #[payable("*")]
    #[endpoint(depositAsset)]
    fn deposit_asset(
        &self,
        initial_caller: Address,
        #[payment_token] asset: TokenIdentifier,
        #[payment_amount] amount: Self::BigUint,
    ) -> SCResult<()> {
        require!(
            self.lending_pool().get() == self.blockchain().get_caller(),
            "permission denied"
        );

        let pool_asset = self.pool_asset().get();
        require!(
            asset == pool_asset,
            "asset not supported for this liquidity pool"
        );

        let interest_metadata = InterestMetadata {
            timestamp: self.blockchain().get_block_timestamp(),
        };
        self.mint_interest(amount.clone(), interest_metadata);

        let lend_token = self.lend_token().get();
        let nonce = self
            .blockchain()
            .get_current_esdt_nft_nonce(&self.blockchain().get_sc_address(), &lend_token);

        self.reserves(&pool_asset).update(|x| *x += &amount);

        self.send()
            .direct(&initial_caller, &lend_token, nonce, &amount, &[]);

        Ok(())
    }

    #[payable("*")]
    #[endpoint(borrow)]
    fn borrow(
        &self,
        initial_caller: Address,
        #[payment_token] lend_token: TokenIdentifier,
        #[payment_amount] amount: Self::BigUint,
        timestamp: u64,
    ) -> SCResult<()> {
        require!(
            self.blockchain().get_caller() == self.lending_pool().get(),
            "can only be called through lending pool"
        );
        require!(amount > 0, "lend amount must be bigger then 0");
        require!(!initial_caller.is_zero(), "invalid address provided");

        let borrows_token = self.borrow_token().get();
        let asset = self.pool_asset().get();

        let mut borrows_reserve = self.reserves(&borrows_token).get();
        let mut asset_reserve = self.reserves(&asset).get();

        require!(
            asset_reserve != Self::BigUint::zero(),
            "asset reserve is empty"
        );

        let position_id = self.get_nft_hash();
        let debt_metadata = DebtMetadata {
            timestamp: self.blockchain().get_block_timestamp(),
            collateral_amount: amount.clone(),
            collateral_identifier: lend_token.clone(),
            collateral_timestamp: timestamp,
        };

        self.mint_debt(amount.clone(), debt_metadata.clone(), position_id.clone());

        let nonce = self
            .blockchain()
            .get_current_esdt_nft_nonce(&self.blockchain().get_sc_address(), &borrows_token);

        // send debt position tokens

        self.send()
            .direct(&initial_caller, &borrows_token, nonce, &amount, &[]);

        // send collateral requested to the user

        // self.send().direct(&initial_caller, &asset, &amount, &[]);

        borrows_reserve += amount.clone();
        asset_reserve -= amount.clone();

        let mut total_borrow = self.total_borrow().get();
        total_borrow += amount.clone();
        self.total_borrow().set(&total_borrow);

        self.reserves(&borrows_token).set(&borrows_reserve);
        self.reserves(&asset).set(&asset_reserve);

        let current_health = self.compute_health_factor();
        let debt_position = DebtPosition::<Self::BigUint> {
            size: amount.clone(), // this will be initial L tokens amount
            health_factor: current_health,
            is_liquidated: false,
            timestamp: debt_metadata.timestamp,
            collateral_amount: amount,
            collateral_identifier: lend_token,
        };
        self.debt_positions()
            .insert(position_id.into_boxed_bytes(), debt_position);

        Ok(())
    }

    #[payable("*")]
    #[endpoint(lockBTokens)]
    fn lock_b_tokens(
        &self,
        initial_caller: Address,
        #[payment_token] borrow_token: TokenIdentifier,
        #[payment_nonce] token_nonce: u64,
        #[payment_amount] amount: Self::BigUint,
    ) -> SCResult<H256> {
        require!(
            self.blockchain().get_caller() == self.lending_pool().get(),
            "can only be called by lending pool"
        );
        require!(amount > 0, "amount must be greater then 0");
        require!(!initial_caller.is_zero(), "invalid address");

        require!(
            borrow_token == self.borrow_token().get(),
            "borrow token not supported by this pool"
        );

        let esdt_nft_data = self.blockchain().get_esdt_token_data(
            &self.blockchain().get_sc_address(),
            &borrow_token,
            token_nonce,
        );

        let debt_position_id = esdt_nft_data.hash.clone();
        let debt_position: DebtPosition<Self::BigUint> = self
            .debt_positions()
            .get(&debt_position_id)
            .unwrap_or_default();

        require!(
            debt_position != DebtPosition::default(),
            "invalid debt position"
        );
        require!(!debt_position.is_liquidated, "position is liquidated");

        let metadata = esdt_nft_data.decode_attributes::<DebtMetadata<Self::BigUint>>()?;
        let data = [
            borrow_token.as_esdt_identifier(),
            amount.to_bytes_be().as_slice(),
            &token_nonce.to_be_bytes()[..],
        ]
        .concat();

        let unique_repay_id = self.crypto().keccak256(&data);
        let repay_position = RepayPostion {
            identifier: borrow_token,
            amount,
            nonce: token_nonce,
            borrow_timestamp: metadata.timestamp,
            collateral_identifier: metadata.collateral_identifier,
            collateral_amount: metadata.collateral_amount,
            collateral_timestamp: metadata.collateral_timestamp,
        };
        self.repay_position()
            .insert(unique_repay_id.clone().into_boxed_bytes(), repay_position);

        Ok(unique_repay_id)
    }

    #[payable("*")]
    #[endpoint]
    fn repay(
        &self,
        unique_id: BoxedBytes,
        #[payment_token] asset: TokenIdentifier,
        #[payment_amount] amount: Self::BigUint,
    ) -> SCResult<RepayPostion<Self::BigUint>> {
        require!(
            self.blockchain().get_caller() == self.lending_pool().get(),
            "function can only be called by lending pool"
        );
        require!(amount > 0, "amount must be greater then 0");
        require!(
            asset == self.pool_asset().get(),
            "asset is not supported by this pool"
        );

        require!(
            self.repay_position().contains_key(&unique_id),
            "there are no locked borrowed token for this id, lock b tokens first"
        );
        let mut repay_position: RepayPostion<Self::BigUint> =
            self.repay_position().get(&unique_id).unwrap_or_default();

        require!(
            repay_position.amount >= amount,
            "b tokens amount locked must be equal with the amount of asset token send"
        );

        let esdt_nft_data = self.blockchain().get_esdt_token_data(
            &self.blockchain().get_sc_address(),
            &repay_position.identifier,
            repay_position.nonce,
        );

        let debt_position_id = esdt_nft_data.hash;

        require!(
            self.debt_positions().contains_key(&debt_position_id),
            "invalid debt position id"
        );
        let debt_position = self
            .debt_positions()
            .get(&debt_position_id)
            .unwrap_or_default();

        require!(!debt_position.is_liquidated, "position is liquidated");

        let interest = self.get_debt_interest(
            repay_position.amount.clone(),
            repay_position.borrow_timestamp,
        )?;

        if repay_position.amount.clone() + interest == amount {
            self.repay_position().remove(&unique_id);
        } else if repay_position.amount > amount {
            repay_position.amount -= amount.clone();
            self.repay_position()
                .insert(unique_id, repay_position.clone());
        }

        /*self.send().esdt_local_burn(
            amount.clone(),
            repay_position.nonce,
            repay_position.identifier.clone(),
        );*/

        repay_position.amount = amount;

        Ok(repay_position)
    }

    #[payable("*")]
    #[endpoint]
    fn withdraw(
        &self,
        initial_caller: Address,
        #[payment_token] lend_token: TokenIdentifier,
        #[payment_amount] amount: Self::BigUint,
    ) -> SCResult<()> {
        require!(
            self.blockchain().get_caller() == self.lending_pool().get(),
            "permission denied"
        );
        require!(
            lend_token == self.lend_token().get(),
            "lend token not supported"
        );

        let pool_asset = self.pool_asset().get();
        let mut asset_reserve = self.reserves(&pool_asset).get();

        let nft_nonce = self.call_value().esdt_token_nonce();
        let nft_info = self.blockchain().get_esdt_token_data(
            &self.blockchain().get_sc_address(),
            &lend_token,
            nft_nonce,
        );
        let metadata: InterestMetadata = nft_info.decode_attributes::<InterestMetadata>()?;

        let deposit_rate = self.get_deposit_rate();
        let time_diff =
            Self::BigUint::from(self.blockchain().get_block_timestamp() - metadata.timestamp);
        let withdrawal_amount =
            self.compute_withdrawal_amount(amount.clone(), time_diff, deposit_rate);
        require!(asset_reserve >= withdrawal_amount, "insufficient funds");

        asset_reserve -= &withdrawal_amount;
        self.reserves(&pool_asset).set(&asset_reserve);

        self.send()
            .direct(&initial_caller, &pool_asset, 0, &withdrawal_amount, &[]);

        self.send().esdt_local_burn(&lend_token, nft_nonce, &amount);

        Ok(())
    }

    #[payable("*")]
    #[endpoint]
    fn liquidate(
        &self,
        position_id: BoxedBytes,
        #[payment_token] token: TokenIdentifier,
        #[payment_amount] amount: Self::BigUint,
    ) -> SCResult<LiquidateData<Self::BigUint>> {
        require!(
            self.blockchain().get_caller() == self.lending_pool().get(),
            "function can only be called by lending pool"
        );
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

        let interest =
            self.get_debt_interest(debt_position.size.clone(), debt_position.timestamp)?;

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
