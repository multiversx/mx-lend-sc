elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use crate::{math, storage};
use price_aggregator_proxy::AggregatorResult;

use common_structs::*;

const TICKER_SEPARATOR: u8 = b'-';
const LEND_TOKEN_NAME: &[u8] = b"IntBearing";
const DEBT_TOKEN_NAME: &[u8] = b"DebtBearing";
const DOLLAR_TICKER: &[u8] = b"USD";

#[elrond_wasm::module]
pub trait UtilsModule:
    math::MathModule + storage::StorageModule + price_aggregator_proxy::PriceAggregatorModule
{
    fn prepare_issue_data(
        &self,
        prefix: ManagedBuffer,
        ticker: ManagedBuffer,
    ) -> IssueData<Self::Api> {
        let mut prefixed_ticker = prefix.clone();
        prefixed_ticker.append(&ticker);

        let mut issue_data = IssueData {
            name: ManagedBuffer::new(),
            ticker: prefixed_ticker,
            is_empty_ticker: true,
        };

        if prefix == ManagedBuffer::from(LEND_TOKEN_PREFIX) {
            let mut name = ManagedBuffer::new_from_bytes(LEND_TOKEN_NAME);
            name.append(&ticker);

            issue_data.name = name;
            issue_data.is_empty_ticker = self.lend_token().is_empty();
        } else if prefix == ManagedBuffer::from(BORROW_TOKEN_PREFIX) {
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

    // TODO: Get rid of this functon and store the ticker in storage
    // maybe along with a token whitelist
    fn get_token_ticker(&self, token_id: &TokenIdentifier) -> ManagedBuffer {
        for (i, char) in token_id.to_esdt_identifier().as_slice().iter().enumerate() {
            if *char == TICKER_SEPARATOR {
                let result = token_id.to_esdt_identifier();
                return ManagedBuffer::new_from_bytes(&result.as_slice()[..i]);
            }
        }

        token_id.as_managed_buffer().clone()
    }

    #[view(getCapitalUtilisation)]
    fn get_capital_utilisation(&self) -> BigUint {
        let reserve_amount = self.reserves().get();
        let borrowed_amount = self.borrowed_amount().get();

        self.compute_capital_utilisation(&borrowed_amount, &reserve_amount)
    }

    #[view(getDebtInterest)]
    fn get_debt_interest(&self, amount: &BigUint, timestamp: u64) -> BigUint {
        let time_diff = self.get_timestamp_diff(timestamp);
        let borrow_rate = self.get_borrow_rate();

        self.compute_debt(amount, &BigUint::from(time_diff as u64), &borrow_rate)
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

    fn get_timestamp_diff(&self, timestamp: u64) -> u64 {
        let current_time = self.blockchain().get_block_timestamp();
        require!(current_time >= timestamp, "Invalid timestamp");

        current_time - timestamp
    }

    fn is_full_repay(
        &self,
        borrow_position: &BorrowPosition<Self::Api>,
        borrow_token_repaid: &BigUint,
    ) -> bool {
        &borrow_position.borrowed_amount == borrow_token_repaid
    }
}
