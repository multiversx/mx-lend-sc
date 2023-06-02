multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::{math, storage};

use common_structs::*;
use price_aggregator_proxy::AggregatorResult;

const TOKEN_ID_SUFFIX_LEN: usize = 7; // "dash" + 6 random bytes
const DOLLAR_TICKER: &[u8] = b"USD";

#[multiversx_sc::module]
pub trait LendingUtilsModule:
    math::LendingMathModule
    + storage::LendingStorageModule
    + price_aggregator_proxy::PriceAggregatorModule
{
    fn get_token_price_data(&self, token_id: TokenIdentifier) -> AggregatorResult<Self::Api> {
        let from_ticker = self.get_token_ticker(token_id);
        let result = self
            .get_full_result_for_pair(from_ticker, ManagedBuffer::new_from_bytes(DOLLAR_TICKER));

        match result {
            Some(r) => r,
            None => sc_panic!("failed to get token price"),
        }
    }

    fn get_token_ticker(&self, token_id: TokenIdentifier) -> ManagedBuffer {
        let as_buffer = token_id.into_managed_buffer();
        let ticker_start_index = 0;
        let ticker_end_index = as_buffer.len() - TOKEN_ID_SUFFIX_LEN;

        let result = as_buffer.copy_slice(ticker_start_index, ticker_end_index);

        match result {
            Some(r) => r,
            None => sc_panic!("failed to get token ticker"),
        }
    }

    // Returns the collateral position for the user or a new DepositPosition if the user didn't add collateral previously
    fn get_existing_or_new_deposit_position_for_token(
        &self,
        account_position: u64,
        token_id: TokenIdentifier,
    ) -> DepositPosition<Self::Api> {
        match self.deposit_positions(account_position).get(&token_id) {
            Some(dp) => dp,
            None => DepositPosition::new(
                token_id,
                BigUint::zero(),
                account_position,
                self.blockchain().get_block_round(),
                BigUint::from(BP),
            ),
        }
    }

    fn get_existing_or_new_borrow_position_for_token(
        &self,
        account_position: u64,
        token_id: TokenIdentifier,
    ) -> BorrowPosition<Self::Api> {
        match self.borrow_positions(account_position).get(&token_id) {
            Some(bp) => bp,
            None => BorrowPosition::new(
                token_id,
                BigUint::zero(),
                account_position,
                self.blockchain().get_block_round(),
                BigUint::from(BP),
            ),
        }
    }

    #[inline]
    #[view(getCollateralAmountForToken)]
    fn get_collateral_amount_for_token(
        &self,
        account_position: u64,
        token_id: TokenIdentifier,
    ) -> BigUint {
        match self.deposit_positions(account_position).get(&token_id) {
            Some(dp) => dp.amount,
            None => BigUint::zero(),
        }
    }

    #[inline]
    #[view(getTotalCollateralAvailable)]
    fn get_total_collateral_in_dollars(&self, account_position: u64) -> BigUint {
        let mut deposited_amount_in_dollars = BigUint::zero();
        let deposit_positions = self.deposit_positions(account_position);

        for dp in deposit_positions.values() {
            let dp_data = self.get_token_price_data(dp.token_id);
            deposited_amount_in_dollars += dp.amount * dp_data.price;
        }

        deposited_amount_in_dollars
    }

    #[view(getTotalBorrowInDollars)]
    fn get_total_borrow_in_dollars(&self, account_position: u64) -> BigUint {
        let mut total_borrow_in_dollars = BigUint::zero();
        let borrow_positions = self.borrow_positions(account_position);

        for bp in borrow_positions.values() {
            let bp_data = self.get_token_price_data(bp.token_id);
            total_borrow_in_dollars += bp.amount * bp_data.price;
        }

        total_borrow_in_dollars
    }

    fn compute_amount_in_tokens(
        &self,
        liquidatee_account_nonce: u64,
        token_to_liquidate: TokenIdentifier,
        amount_to_return_to_liquidator_in_dollars: BigUint,
    ) -> BigUint {
        require!(
            self.deposit_positions(liquidatee_account_nonce)
                .contains_key(&token_to_liquidate),
            "Liquidatee user doesn't have this token as collateral"
        );

        let token_data = self.get_token_price_data(token_to_liquidate);
        (&amount_to_return_to_liquidator_in_dollars * BP / &token_data.price)
            * BigUint::from(10u64).pow(token_data.decimals as u32)
            / BP
    }
}
