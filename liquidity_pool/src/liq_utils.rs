elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use crate::{liq_math, liq_storage};

use common_structs::*;

#[elrond_wasm::module]
pub trait UtilsModule:
    liq_math::MathModule + liq_storage::StorageModule + price_aggregator_proxy::PriceAggregatorModule
{
    #[view(getCapitalUtilisation)]
    fn get_capital_utilisation(&self) -> BigUint {
        let borrowed_amount = self.borrowed_amount().get();
        let total_amount = self.supplied_amount().get();

        sc_print!(
            "borrowed_amount = {}, total_amount = {}",
            borrowed_amount,
            total_amount
        );

        self.compute_capital_utilisation(&borrowed_amount, &total_amount)
    }

    #[view(getTotalCapital)]
    fn get_total_capital(&self) -> BigUint {
        let reserve_amount = self.reserves().get();
        let borrowed_amount = self.borrowed_amount().get();

        &reserve_amount + &borrowed_amount
    }

    #[view(getDebtInterest)]
    fn get_debt_interest(&self, amount: &BigUint, initial_borrow_index: &BigUint) -> BigUint {
        let borrow_index_diff = self.get_borrow_index_diff(initial_borrow_index);

        sc_print!(
            "amount = {}, borrow_index_diff = {}",
            &amount,
            &borrow_index_diff
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

        sc_print!("Capital Utilisation = {}", capital_utilisation);
        self.compute_borrow_rate(
            &pool_params.r_base,
            &pool_params.r_slope1,
            &pool_params.r_slope2,
            &pool_params.u_optimal,
            &capital_utilisation,
        )
    }

    fn update_borrow_index(&self, borrow_rate: &BigUint, delta_rounds: u64) {
        sc_print!(
            "borrow_rate = {}, delta_rounds = {}, self.borrow_index= {}",
            borrow_rate,
            delta_rounds,
            self.borrow_index().get()
        );
        self.borrow_index()
            .update(|new_index| *new_index += borrow_rate * delta_rounds);
        sc_print!(
            "a devenit self.borrow_index = {}",
            self.borrow_index().get()
        );
    }

    fn update_supply_index(&self, rewards_increase: BigUint) {
        let total_supplied_amount = self.supplied_amount().get();

        sc_print!(
            "supply_index = {} + rewards_increase = {} / total_supplied_amount = {}",
            self.supply_index().get(),
            rewards_increase,
            &total_supplied_amount
        );

        if total_supplied_amount != BigUint::zero() {
            self.supply_index()
                .update(|new_index| *new_index += rewards_increase * BP / total_supplied_amount);
        }
    }

    fn update_rewards_reserves(&self, borrow_rate: &BigUint, delta_rounds: u64) -> BigUint {
        let borrowed_amount = self.borrowed_amount().get();
        let rewards_increase = borrow_rate * &borrowed_amount * delta_rounds / BP;

        sc_print!("rewards_increase = {}", rewards_increase);
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
        sc_print!(
            "initial_borrow_index = {} -> current_borrow_index = {}",
            initial_borrow_index,
            current_borrow_index
        );
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
}
