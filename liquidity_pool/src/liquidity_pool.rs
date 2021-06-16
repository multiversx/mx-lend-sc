elrond_wasm::imports!();
use elrond_wasm::*;
use elrond_wasm::types::{Address, TokenIdentifier, SCResult, H256, BoxedBytes};
use crate::{InterestMetadata, DebtMetadata, DebtPosition, RepayPostion, LiquidateData};

#[elrond_wasm_derive::module]
pub trait LiquidityPoolModule: crate::storage::StorageModule + crate::tokens::TokensModule{

    fn deposit_asset(
        &self,
        initial_caller: Address,
        asset: TokenIdentifier,
        amount: Self::BigUint,
    ) -> SCResult<()> {
        require!(
            self.get_lending_pool() == self.blockchain().get_caller(),
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
            .blockchain().get_current_esdt_nft_nonce(&self.blockchain().get_sc_address(), &lend_token);

        self.send().direct_nft(
            &initial_caller,
            &lend_token,
            nonce,
            &amount,
            &[],
        );

        let mut asset_reserve = self
            .reserves()
            .get(&pool_asset)
            .unwrap_or_else(Self::BigUint::zero);
        asset_reserve += amount;
        self.reserves().insert(pool_asset, asset_reserve);

        Ok(())
    }

   
    fn borrow(
        &self,
        initial_caller: Address,
        lend_token: TokenIdentifier,
        amount: Self::BigUint,
        timestamp: u64,
    ) -> SCResult<()> {
        require!(
            self.blockchain().get_caller() == self.get_lending_pool(),
            "can only be called through lending pool"
        );
        require!(amount > 0, "lend amount must be bigger then 0");
        require!(!initial_caller.is_zero(), "invalid address provided");

        let borrows_token = self.borrow_token().get();
        let asset = self.pool_asset().get();

        let mut borrows_reserve = self
            .reserves()
            .get(&borrows_token)
            .unwrap_or_else(Self::BigUint::zero);
        let mut asset_reserve = self.reserves().get(&asset).unwrap_or_else(Self::BigUint::zero);

        require!(asset_reserve != Self::BigUint::zero(), "asset reserve is empty");

        let position_id = self.get_nft_hash();
        let debt_metadata = DebtMetadata {
            timestamp: self.blockchain().get_block_timestamp(),
            collateral_amount: amount.clone(),
            collateral_identifier: lend_token.clone(),
            collateral_timestamp: timestamp,
        };

        self.mint_debt(amount.clone(), debt_metadata.clone(), position_id.clone());

        let nonce = self
            .blockchain().get_current_esdt_nft_nonce(&self.blockchain().get_sc_address(), &borrows_token);

        // send debt position tokens

        self.send().direct_nft(
            &initial_caller,
            &borrows_token,
            nonce,
            &amount,
            &[],
        );

        // send collateral requested to the user

        self.send().direct(&initial_caller, &asset, &amount, &[]);

        borrows_reserve += amount.clone();
        asset_reserve -= amount.clone();

        let mut total_borrow = self.get_total_borrow();
        total_borrow += amount.clone();
        self.set_total_borrow(total_borrow);

        self.reserves().insert(borrows_token, borrows_reserve);
        self.reserves().insert(asset, asset_reserve);

        let current_health = self.compute_health_factor();
        let debt_position = DebtPosition::<Self::BigUint> {
            size: amount.clone(), // this will be initial L tokens amount
            health_factor: current_health,
            is_liquidated: false,
            timestamp: debt_metadata.timestamp,
            collateral_amount: amount,
            collateral_identifier: lend_token,
        };
        self.debt_positions().insert(position_id, debt_position);

        Ok(())
    }

   
    fn lock_b_tokens(
        &self,
        initial_caller: Address,
        borrow_token: TokenIdentifier,
        amount: Self::BigUint,
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

        let nft_nonce = self.call_value().esdt_token_nonce();

        let esdt_nft_data = self.blockchain().get_esdt_token_data(
            &self.blockchain().get_sc_address(),
            &borrow_token,
            nft_nonce,
        );

        let debt_position_id = esdt_nft_data.hash;
        let debt_position: DebtPosition<Self::BigUint> = self
            .debt_positions()
            .get(&debt_position_id)
            .unwrap_or_default();

        require!(
            debt_position != DebtPosition::default(),
            "invalid debt position"
        );
        require!(!debt_position.is_liquidated, "position is liquidated");

        let metadata: DebtMetadata<Self::BigUint>;
        match DebtMetadata::<Self::BigUint>::top_decode(esdt_nft_data.attributes.as_slice()) {
            Result::Ok(decoded) => {
                metadata = decoded;
            }
            Result::Err(_) => {
                return sc_error!("could not parse token metadata");
            }
        }
        let data = [
            borrow_token.as_esdt_identifier(),
            amount.to_bytes_be().as_slice(),
            &nft_nonce.to_be_bytes()[..],
        ]
        .concat();

        let unique_repay_id = self.crypto().keccak256(&data);
        let repay_position = RepayPostion {
            identifier: borrow_token,
            amount,
            nonce: nft_nonce,
            borrow_timestamp: metadata.timestamp,
            collateral_identifier: metadata.collateral_identifier,
            collateral_amount: metadata.collateral_amount,
            collateral_timestamp: metadata.collateral_timestamp,
        };
        self.repay_position()
            .insert(unique_repay_id.to_box_bytes(), repay_position);

        Ok(unique_repay_id)
    }

  
    fn repay(
        &self,
        unique_id: BoxedBytes,
        asset: TokenIdentifier,
        amount: Self::BigUint,
    ) -> SCResult<RepayPostion<Self::BigUint>> {
        require!(
            self.blockchain().get_caller() == self.get_lending_pool(),
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
        let mut repay_position:RepayPostion<Self::BigUint> = self.repay_position().get(&unique_id).unwrap_or_default();

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
        );

        if repay_position.amount.clone() + interest == amount {
            self.repay_position().remove(&unique_id);
        } else if repay_position.amount > amount {
            repay_position.amount -= amount.clone();
            self.repay_position()
                .insert(unique_id, repay_position.clone());
        }

        self.set_repay_position_amount(amount.clone());
        self.set_repay_position_id(repay_position.identifier.clone());
        self.set_repay_position_nonce(repay_position.nonce.clone());

        /*self.burn(
            amount.clone(),
            repay_position.nonce,
            repay_position.identifier.clone(),
        );*/

        repay_position.amount = amount;

        Ok(repay_position)
    }


    fn withdraw(
        &self,
        initial_caller: Address,
        lend_token: TokenIdentifier,
        amount: Self::BigUint,
    ) -> SCResult<()> {
        require!(
            self.blockchain().get_caller() == self.lending_pool().get(),
            "permission denied"
        );
        require!(
            lend_token == self.lending_pool().get(),
            "lend token not supported"
        );

        let pool_asset = self.pool_asset().get();
        let mut asset_reserve = self
            .reserves()
            .get(&pool_asset)
            .unwrap_or_else(Self::BigUint::zero);

        let nft_nonce = self.call_value().esdt_token_nonce();
        let nft_info = self.blockchain().get_esdt_token_data(
            &self.blockchain().get_sc_address(),
            &lend_token,
            nft_nonce,
        );
        let metadata: InterestMetadata;
        match InterestMetadata::top_decode(nft_info.attributes.clone().as_slice()) {
            Result::Ok(decoded) => {
                metadata = decoded;
            }
            Result::Err(_) => {
                return sc_error!("could not parse token metadata");
            }
        }

        let deposit_rate = self.get_deposit_rate();
        let time_diff = Self::BigUint::from(self.blockchain().get_block_timestamp() - metadata.timestamp);
        let withdrawal_amount = self.compute_withdrawal_amount(
            amount.clone(),
            time_diff,
            deposit_rate,
        );

        self.set_asset_reserve(asset_reserve.clone());
        self.set_withdraw_amount(withdrawal_amount.clone());
        //require!(asset_reserve > withdrawal_amount, "insufficient funds");

        self.send()
            .direct(&initial_caller, &pool_asset, &withdrawal_amount, &[]);

        self.burn(amount.clone(), nft_nonce, lend_token);

        asset_reserve -= amount;
        self.reserves().insert(pool_asset, asset_reserve);

        Ok(())
    }


    fn liquidate(
        &self,
        position_id: BoxedBytes,
        token: TokenIdentifier,
        amount: Self::BigUint,
    ) -> SCResult<LiquidateData<Self::BigUint>> {
        require!(
            self.blockchain().get_caller() == self.get_lending_pool(),
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
            debt_position.health_factor < self.get_health_factor_threshold(),
            "the health factor is not low enough"
        );

        let interest = self.get_debt_interest(debt_position.size.clone(), debt_position.timestamp);

        require!(
            debt_position.size.clone() + interest == amount,
            "position can't be liquidated, not enough or to much tokens send"
        );

        debt_position.is_liquidated = true;

        self.debt_positions()
            .insert(position_id, debt_position.clone());

        let liquidate_data = LiquidateData {
            collateral_token: debt_position.collateral_identifier,
            amount : debt_position.size
        };

        Ok(liquidate_data)
    }

}