elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use crate::{math, storage};

use common_structs::*;
use price_aggregator_proxy::AggregatorResult;

const TOKEN_ID_SUFFIX_LEN: usize = 7; // "dash" + 6 random bytes
const DOLLAR_TICKER: &[u8] = b"USD";

#[elrond_wasm::module]
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
    fn get_collateral_position_for_token(
        &self,
        account_position: u64,
        token_id: TokenIdentifier,
    ) -> DepositPosition<Self::Api> {
        let deposit_positions = self.deposit_position();
        let deposit_position_iter = deposit_positions
            .iter()
            .filter(|dp| dp.owner_nonce == account_position);

        for dp in deposit_position_iter {
            if dp.token_id == token_id {
                self.deposit_position().swap_remove(&dp);
                return dp;
            }
        }
        DepositPosition::new(
            token_id,
            BigUint::zero(),
            account_position,
            self.blockchain().get_block_round(),
            BigUint::from(BP),
        )
    }

    fn get_borrow_position_for_token(
        &self,
        account_position: u64,
        token_id: TokenIdentifier,
    ) -> BorrowPosition<Self::Api> {
        let borrow_positions = self.borrow_position();
        let borrow_position_iter = borrow_positions
            .iter()
            .filter(|bp| bp.owner_nonce == account_position);

        for bp in borrow_position_iter {
            if bp.token_id == token_id {
                self.borrow_position().swap_remove(&bp);
                return bp;
            }
        }
        BorrowPosition::new(
            token_id,
            BigUint::zero(),
            account_position,
            self.blockchain().get_block_round(),
            BigUint::from(BP),
        )
    }

    #[inline]
    #[view(getTotalCollateralAvailableForToken)]
    fn get_collateral_available_for_token(
        &self,
        account_position: u64,
        token_id: TokenIdentifier,
    ) -> BigUint {
        let mut deposited_amount_in_dollars = BigUint::zero();
        let deposit_positions = self.deposit_position();
        let deposit_position_iter = deposit_positions
            .iter()
            .filter(|dp| dp.owner_nonce == account_position && dp.token_id == token_id);

        for dp in deposit_position_iter {
            let dp_data = self.get_token_price_data(dp.token_id);
            deposited_amount_in_dollars += dp.amount * dp_data.price;
        }

        deposited_amount_in_dollars
    }

    #[inline]
    #[view(getTotalCollateralAvailable)]
    fn get_total_collateral_available(&self, account_position: u64) -> BigUint {
        let mut deposited_amount_in_dollars = BigUint::zero();
        let deposit_positions = self.deposit_position();
        let deposit_position_iter = deposit_positions
            .iter()
            .filter(|dp| dp.owner_nonce == account_position);

        for dp in deposit_position_iter {
            let dp_data = self.get_token_price_data(dp.token_id);
            deposited_amount_in_dollars += dp.amount * dp_data.price;
        }

        deposited_amount_in_dollars
    }

    #[view(getTotalBorrowedAmount)]
    fn get_total_borrowed_amount(&self, account_position: u64) -> BigUint {
        let mut total_borrowed_amount = BigUint::zero();
        let borrow_positions = self.borrow_position();
        let borrow_position_iter = borrow_positions
            .iter()
            .filter(|bp| bp.owner_nonce == account_position);

        for bp in borrow_position_iter {
            let bp_data = self.get_token_price_data(bp.token_id);
            total_borrowed_amount += bp.amount * bp_data.price;
        }

        total_borrowed_amount
    }

    fn send_amount_in_dollars_to_liquidator(
        &self,
        initial_caller: ManagedAddress,
        liquidatee_account_nonce: u64,
        amount_to_return_to_liquidator_in_dollars: BigUint,
    ) {
        let mut amount_to_send = amount_to_return_to_liquidator_in_dollars;
        let deposit_positions = self.deposit_position();
        let deposit_position_iter = deposit_positions
            .iter()
            .filter(|dp| dp.owner_nonce == liquidatee_account_nonce);

        // Send amount_to_return_in_dollars to initial_caller
        for mut dp in deposit_position_iter {
            let dp_data = self.get_token_price_data(dp.token_id.clone());
            let amount_in_dollars_available_for_this_bp = &dp.amount * &dp_data.price;

            if amount_in_dollars_available_for_this_bp <= amount_to_send {
                // Send all tokens and remove DepositPosition
                       self.send()
                    .direct_esdt(&initial_caller, &dp.token_id, 0, &dp.amount);
                amount_to_send -= amount_in_dollars_available_for_this_bp;
                self.deposit_position().swap_remove(&dp);

                if amount_to_send == 0 {
                    break;
                }
            } else {
                // Send part of the tokens and update DepositPosition
                let partial_amount_to_send = (&amount_to_send * BP / &dp_data.price)
                    * BigUint::from(10u64).pow(dp_data.decimals as u32)
                    / BP;

                self.send()
                    .direct_esdt(&initial_caller, &dp.token_id, 0, &partial_amount_to_send);

                // Update DepositPosition
                self.deposit_position().swap_remove(&dp);
                dp.amount -= partial_amount_to_send;
                self.deposit_position().insert(dp);
                break;
            }
        }
    }
}
