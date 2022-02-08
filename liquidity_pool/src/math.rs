elrond_wasm::imports!();

const BP: u32 = 1_000_000_000;

const SECONDS_IN_YEAR: u32 = 31_556_926;

#[elrond_wasm::module]
pub trait MathModule {
    fn compute_borrow_rate(
        &self,
        r_base: &BigUint,
        r_slope1: &BigUint,
        r_slope2: &BigUint,
        u_optimal: &BigUint,
        u_current: &BigUint,
    ) -> BigUint {
        let bp = BigUint::from(BP);

        if u_current < u_optimal {
            let utilisation_ratio = &(u_current * r_slope1) / u_optimal;
            r_base + &utilisation_ratio
        } else {
            let denominator = &bp - u_optimal;
            let numerator = &(u_current - u_optimal) * r_slope2;
            (r_base + r_slope1) + numerator / denominator
        }
    }

    fn compute_deposit_rate(
        &self,
        u_current: &BigUint,
        borrow_rate: &BigUint,
        reserve_factor: &BigUint,
    ) -> BigUint {
        let bp = BigUint::from(BP);
        let loan_ratio = u_current * borrow_rate;
        let deposit_rate = &(u_current * &loan_ratio) * &(&bp - reserve_factor);
        deposit_rate / (&bp * &bp * bp)
    }

    fn compute_capital_utilisation(
        &self,
        borrowed_amount: &BigUint,
        total_reserves: &BigUint,
    ) -> BigUint {
        let bp = BigUint::from(BP);
        &(borrowed_amount * &bp) / total_reserves
    }

    fn compute_debt(
        &self,
        amount: &BigUint,
        time_diff: &BigUint,
        borrow_rate: &BigUint,
    ) -> BigUint {
        let bp = BigUint::from(BP);
        let secs_year = BigUint::from(SECONDS_IN_YEAR);
        let time_unit_percentage = (time_diff * &bp) / secs_year;
        let debt_percetange = &(&time_unit_percentage * borrow_rate) / &bp;

        (&debt_percetange * amount) / bp
    }

    fn compute_withdrawal_amount(
        &self,
        amount: &BigUint,
        time_diff: &BigUint,
        deposit_rate: &BigUint,
    ) -> BigUint {
        let bp = BigUint::from(BP);
        let secs_year = BigUint::from(SECONDS_IN_YEAR);
        let percentage = &(time_diff * deposit_rate) / &secs_year;
        let interest = &percentage * amount / bp;

        amount + &interest
    }

    fn compute_borrowable_amount(
        &self,
        amount: &BigUint,
        price: &BigUint,
        loan_to_value: &BigUint,
        decimals: u8,
    ) -> BigUint {
        let bp = BigUint::from(BP);
        let total_collateral = amount * price;

        ((&total_collateral * loan_to_value) / bp) / BigUint::from(10u64).pow(decimals as u32)
    }

    fn compute_health_factor(
        &self,
        collateral_value_in_dollars: &BigUint,
        borrowed_value_in_dollars: &BigUint,
        liquidation_threshold: &BigUint,
    ) -> BigUint {
        let bp = self.get_base_precision();

        let allowed_collateral_in_dollars = collateral_value_in_dollars * liquidation_threshold;

        let health_factor = &allowed_collateral_in_dollars / borrowed_value_in_dollars;

        health_factor / bp
    }

    fn get_base_precision(&self) -> BigUint {
        BigUint::from(BP)
    }

    fn rule_of_three(&self, value: &BigUint, part: &BigUint, total: &BigUint) -> BigUint {
        &(value * part) / total
    }
}
