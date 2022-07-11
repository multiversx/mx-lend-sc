elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use crate::{math, storage};
use price_aggregator_proxy::AggregatorResult;

use common_structs::*;

const TOKEN_ID_SUFFIX_LEN: usize = 7; // "dash" + 6 random bytes
const LEND_TOKEN_NAME: &[u8] = b"IntBearing";
const DEBT_TOKEN_NAME: &[u8] = b"DebtBearing";
const DOLLAR_TICKER: &[u8] = b"USD";

#[elrond_wasm::module]
pub trait UtilsModule:
    math::MathModule + storage::StorageModule + price_aggregator_proxy::PriceAggregatorModule
{
    fn prepare_issue_data(&self, prefix: u8, ticker: ManagedBuffer) -> IssueData<Self::Api> {
        let mut prefixed_ticker = ManagedBuffer::new_from_bytes(&[prefix]);
        prefixed_ticker.append(&ticker);

        let mut issue_data = IssueData {
            name: ManagedBuffer::new(),
            ticker: prefixed_ticker,
            is_empty_ticker: true,
        };

        if prefix == LEND_TOKEN_PREFIX {
            let mut name = ManagedBuffer::new_from_bytes(LEND_TOKEN_NAME);
            name.append(&ticker);

            issue_data.name = name;
            issue_data.is_empty_ticker = self.lend_token().is_empty();
        } else if prefix == BORROW_TOKEN_PREFIX {
            let mut name = ManagedBuffer::new_from_bytes(DEBT_TOKEN_NAME);
            name.append(&ticker);

            issue_data.name = name;
            issue_data.is_empty_ticker = self.borrow_token().is_empty();
        }

        issue_data
    }

    fn get_token_price_data(&self, token_id: &TokenIdentifier) -> AggregatorResult<Self::Api> {
        let from_ticker = self.get_token_ticker(token_id);
        let result = self
            .get_full_result_for_pair(from_ticker, ManagedBuffer::new_from_bytes(DOLLAR_TICKER));

        match result {
            Some(r) => r,
            None => sc_panic!("failed to get token price"),
        }
    }

    fn get_token_price_data_lending(
        &self,
        token_id: &TokenIdentifier,
    ) -> AggregatorResult<Self::Api> {
        let from_ticker = self.get_token_ticker_from_lending(token_id);
        let result = self
            .get_full_result_for_pair(from_ticker, ManagedBuffer::new_from_bytes(DOLLAR_TICKER));

        match result {
            Some(r) => r,
            None => sc_panic!("failed to get token price"),
        }
    }

    fn get_token_ticker(&self, token_id: &TokenIdentifier) -> ManagedBuffer {
        if token_id.is_egld() {
            return ManagedBuffer::new_from_bytes(b"EGLD");
        }
        let as_buffer = token_id.as_managed_buffer();
        let ticker_start_index = 0;
        let ticker_end_index = as_buffer.len() - TOKEN_ID_SUFFIX_LEN;

        as_buffer
            .copy_slice(ticker_start_index, ticker_end_index)
            .unwrap()
    }

    // Each lent/borrowed token has an L/B prefix, so we start from index 1
    fn get_token_ticker_from_lending(&self, token_id: &TokenIdentifier) -> ManagedBuffer {
        let as_buffer = token_id.as_managed_buffer();
        let ticker_start_index = 1;
        let ticker_end_index = as_buffer.len() - TOKEN_ID_SUFFIX_LEN - 1;

        as_buffer
            .copy_slice(ticker_start_index, ticker_end_index)
            .unwrap()
    }

    #[view(getCapitalUtilisation)]
    fn get_capital_utilisation(&self) -> BigUint {
        let reserve_amount = self.reserves().get();
        let borrowed_amount = self.borrowed_amount().get();
        let rewards_reserves = self.rewards_reserves().get();
        sc_print!(
            "reserve_amount = {}, borrowed_amount = {}, rewards_reserves = {}",
            reserve_amount,
            borrowed_amount,
            rewards_reserves
        );
        let total_amount = &reserve_amount + &borrowed_amount - rewards_reserves;

        self.compute_capital_utilisation(&borrowed_amount, &total_amount)
    }

    #[view(getDebtInterest)]
    fn get_debt_interest(&self, amount: &BigUint, initial_borrow_index: &BigUint) -> BigUint {
        let borrow_index_diff = self.get_borrow_index_diff(initial_borrow_index);
        sc_print!(
            "borrow_index_diff = {}, amount = {}",
            borrow_index_diff,
            amount
        );
        amount * &borrow_index_diff / BP
    }

    #[view(getDepositRate)]
    fn get_deposit_rate(&self) -> BigUint {
        let pool_params = self.pool_params().get();
        let capital_utilisation = self.get_capital_utilisation();
        let borrow_rate = self.get_borrow_rate();

        self.compute_deposit_rate(
            &capital_utilisation,
            &borrow_rate,
            &pool_params.reserve_factor,
        )
    }

    #[view(getBorrowRate)]
    fn get_borrow_rate(&self) -> BigUint {
        let pool_params = self.pool_params().get();
        let capital_utilisation = self.get_capital_utilisation();

        self.compute_borrow_rate(
            &pool_params.r_base,
            &pool_params.r_slope1,
            &pool_params.r_slope2,
            &pool_params.u_optimal,
            &capital_utilisation,
        )
    }

    fn update_borrow_index(&self, round_last_update: u64) {
        let borrow_rate = self.get_borrow_rate();
        // let capital_utilisation = self.get_capital_utilisation();
        let delta_rounds = self.get_round_diff(round_last_update);

        self.borrow_index()
            .set(self.borrow_index().get() + &borrow_rate * delta_rounds);
    }

    fn update_supply_index(&self, rewards_increase: BigUint) {
        let reserve_amount = self.reserves().get();
        let borrowed_amount = self.borrowed_amount().get();
        let rewards_reserves = self.rewards_reserves().get();
        let total_amount =
            &reserve_amount + &borrowed_amount - rewards_reserves + &rewards_increase;

        if total_amount != BigUint::zero() {
            sc_print!(
                "!!!!rewards_increase = {}, total_amount = {}",
                rewards_increase,
                total_amount
            );
            self.supply_index()
                .set(self.supply_index().get() + rewards_increase * BP / total_amount);

            sc_print!("supply_index = {}", self.supply_index().get());
        }
    }

    fn update_rewards_reserves(&self, borrow_index_last_used: u64) -> BigUint {
        let borrow_rate = self.get_borrow_rate();
        // let capital_utilisation = self.get_capital_utilisation();
        let delta_rounds = self.get_round_diff(borrow_index_last_used);
        let borrowed_amount = self.borrowed_amount().get();
        let initial_rewards_reserves = self.rewards_reserves().get();

        self.rewards_reserves().set(
            self.rewards_reserves().get() + &borrow_rate * &borrowed_amount * delta_rounds / BP,
        );
        self.rewards_reserves().get() - initial_rewards_reserves
    }

    fn update_index_last_used(&self) {
        self.borrow_index_last_used()
            .set(self.blockchain().get_block_round());
        self.supply_index_last_used()
            .set(self.blockchain().get_block_round());
    }

    fn get_timestamp_diff(&self, timestamp: u64) -> u64 {
        let current_time = self.blockchain().get_block_timestamp();
        require!(current_time >= timestamp, "Invalid timestamp");

        current_time - timestamp
    }

    fn get_round_diff(&self, initial_round: u64) -> u64 {
        let current_round = self.blockchain().get_block_round();
        require!(current_round >= initial_round, "Invalid timestamp");

        current_round - initial_round
    }

    fn get_borrow_index_diff(&self, initial_borrow_index: &BigUint) -> BigUint {
        let current_borrow_index = self.borrow_index().get();
        sc_print!(
            "current_borrow_index = {} initial_borrow_index = {}",
            current_borrow_index,
            initial_borrow_index
        );
        require!(
            &current_borrow_index >= initial_borrow_index,
            "Invalid timestamp"
        );

        current_borrow_index - initial_borrow_index
    }

    #[inline]
    fn is_full_repay(
        &self,
        borrow_position: &BorrowPosition<Self::Api>,
        borrow_token_repaid: &BigUint,
    ) -> bool {
        &borrow_position.borrowed_amount == borrow_token_repaid
    }
}
