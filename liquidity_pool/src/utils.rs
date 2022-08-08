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
        let borrowed_amount = self.borrowed_amount().get();
        let total_amount = self.get_total_supplied_capital();

        self.compute_capital_utilisation(&borrowed_amount, &total_amount)
    }

    #[view(getTotalSuppliedCapital)]
    fn get_total_supplied_capital(&self) -> BigUint {
        let reserve_amount = self.reserves().get();
        let borrowed_amount = self.borrowed_amount().get();

        &reserve_amount + &borrowed_amount
    }

    #[view(getDebtInterest)]
    fn get_debt_interest(&self, amount: &BigUint, initial_borrow_index: &BigUint) -> BigUint {
        let borrow_index_diff = self.get_borrow_index_diff(initial_borrow_index);

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

    fn update_borrow_index(&self, borrow_rate: &BigUint, delta_rounds: u64) {
        self.borrow_index()
            .update(|new_index| *new_index += borrow_rate * delta_rounds);
    }

    fn update_supply_index(&self, rewards_increase: BigUint) {
        let total_amount = self.get_total_supplied_capital();

        if total_amount != BigUint::zero() {
            self.supply_index()
                .update(|new_index| *new_index += rewards_increase * BP / total_amount);
        }
    }

    fn update_rewards_reserves(&self, borrow_rate: &BigUint, delta_rounds: u64) -> BigUint {
        let borrowed_amount = self.borrowed_amount().get();
        let rewards_increase = borrow_rate * &borrowed_amount * delta_rounds / BP;
        self.rewards_reserves().update(|rewards_reserves| {
            *rewards_reserves += &rewards_increase;
        });
        rewards_increase
    }

    fn update_index_last_used(&self) {
        let current_block_round = self.blockchain().get_block_round();
        self.borrow_index_last_update_round()
            .set(current_block_round);
    }

    fn get_round_diff(&self, initial_round: u64) -> u64 {
        let current_round = self.blockchain().get_block_round();
        require!(current_round >= initial_round, "Invalid round");

        current_round - initial_round
    }

    fn get_borrow_index_diff(&self, initial_borrow_index: &BigUint) -> BigUint {
        let current_borrow_index = self.borrow_index().get();
        require!(
            &current_borrow_index >= initial_borrow_index,
            "Invalid borrow index"
        );

        current_borrow_index - initial_borrow_index
    }

    fn update_interest_indexes(&self) {
        let borrow_index_last_update_round = self.borrow_index_last_update_round().get();
        let delta_rounds = self.get_round_diff(borrow_index_last_update_round);

        if delta_rounds > 0 {
            let borrow_rate = self.get_borrow_rate();

            self.update_borrow_index(&borrow_rate, delta_rounds);
            let rewards_increase = self.update_rewards_reserves(&borrow_rate, delta_rounds);
            self.update_supply_index(rewards_increase);
            self.update_index_last_used();
        }
    }

    #[inline]
    fn is_full_repay(
        &self,
        borrow_position: &BorrowPosition<Self::Api>,
        borrow_token_repaid: &BigUint,
    ) -> bool {
        &borrow_position.amount == borrow_token_repaid
    }

    fn get_collateral_available(&self, account_position: u64, loan_to_value: BigUint) -> BigUint {
        let collateral_available = BigUint::zero();
        let deposit_position_iter = self
            .deposit_position()
            .iter()
            .filter(|dp| dp.owner_nonce == account_position);

        for dp in deposit_position_iter {
            let dp_data = self.get_token_price_data_lending(&dp.token_id);
            let collateral_amount_in_dollars = self.compute_borrowable_amount(
                &dp.amount,
                &dp_data.price,
                &loan_to_value,
                dp_data.decimals,
            );

            collateral_available += collateral_amount_in_dollars
        }

        collateral_available
    }

    fn merge_deposit_positions(&self, account_position: u64) -> DepositPosition<Self::Api> {
        let pool_asset = self.pool_asset().get();
        let round = self.blockchain().get_block_round();
        let supply_index = self.supply_index().get();
        let zero_amount = BigUint::zero();
        let deposit_position_iter = self
            .deposit_position()
            .iter()
            .filter(|dp| dp.owner_nonce == account_position);

        // Create new DP
        let merged_deposit_position = DepositPosition::new(
            pool_asset,
            zero_amount,
            account_position,
            round,
            supply_index,
        );

        for dp in deposit_position_iter {
            // Add old DP to new DP
            let interest_accrued =
                self.compute_interest(&dp.amount, &supply_index, &dp.initial_supply_index);

            merged_deposit_position.amount += dp.amount + interest_accrued;
            self.deposit_position().swap_remove(&dp);
        }
        // Add DP to self.deposit_position()
        self.deposit_position().insert(merged_deposit_position);

        merged_deposit_position
    }

    fn merge_borrow_positions(&self, account_position: u64) -> BorrowPosition<Self::Api> {
        let pool_asset = self.pool_asset().get();
        let round = self.blockchain().get_block_round();
        let borrow_index = self.borrow_index().get();
        let zero_amount = BigUint::zero();
        let borrow_position_iter = self
            .borrow_position()
            .iter()
            .filter(|bp| bp.owner_nonce == account_position);

        // Create new DP
        let merged_borrow_position = BorrowPosition::new(
            pool_asset,
            zero_amount,
            account_position,
            round,
            borrow_index,
        );

        for bp in borrow_position_iter {
            // Add old DP to new DP
            let accumulated_debt = self.get_debt_interest(&bp.amount, &bp.initial_borrow_index);

            merged_borrow_position.amount += bp.amount + accumulated_debt;
            self.borrow_position().swap_remove(&bp);
        }
        // Add DP to self.borrow_position()
        self.borrow_position().insert(merged_borrow_position);

        merged_borrow_position
    }
}
