elrond_wasm::imports!();

const BP: u32 = 1_000_000_000;

const SECONDS_IN_YEAR: u32 = 31556926;

#[elrond_wasm::module]
pub trait MathModule {
    fn compute_borrow_rate(
        &self,
        r_base: &Self::BigUint,
        r_slope1: &Self::BigUint,
        r_slope2: &Self::BigUint,
        u_optimal: &Self::BigUint,
        u_current: &Self::BigUint,
    ) -> Self::BigUint {
        let bp = Self::BigUint::from(BP);

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
        u_current: &Self::BigUint,
        borrow_rate: &Self::BigUint,
        reserve_factor: &Self::BigUint,
    ) -> Self::BigUint {
        let bp = Self::BigUint::from(BP);
        let loan_ratio = u_current * borrow_rate;
        let deposit_rate = &(u_current * &loan_ratio) * &(&bp - reserve_factor);
        deposit_rate / (&bp * &bp * bp)
    }

    fn compute_capital_utilisation(
        &self,
        borrowed_amount: &Self::BigUint,
        total_reserves: &Self::BigUint,
    ) -> Self::BigUint {
        let bp = Self::BigUint::from(BP);
        &(borrowed_amount * &bp) / total_reserves
    }

    fn compute_debt(
        &self,
        amount: &Self::BigUint,
        time_diff: &Self::BigUint,
        borrow_rate: &Self::BigUint,
    ) -> Self::BigUint {
        let bp = Self::BigUint::from(BP);
        let secs_year = Self::BigUint::from(SECONDS_IN_YEAR);
        let time_unit_percentage = (time_diff * &bp) / secs_year;
        let debt_percetange = &(&time_unit_percentage * borrow_rate) / &bp;

        (&debt_percetange * amount) / bp
    }

    fn compute_withdrawal_amount(
        &self,
        amount: &Self::BigUint,
        time_diff: &Self::BigUint,
        deposit_rate: &Self::BigUint,
    ) -> Self::BigUint {
        let bp = Self::BigUint::from(BP);
        let secs_year = Self::BigUint::from(SECONDS_IN_YEAR);
        let percentage = &(time_diff * deposit_rate) / &secs_year;
        let interest = &percentage * amount / bp;

        amount + &interest
    }

    fn compute_borrowable_amount(
        &self,
        amount: &Self::BigUint,
        price: &Self::BigUint,
        loan_to_value: &Self::BigUint,
        decimals: u8,
    ) -> Self::BigUint {
        let bp = Self::BigUint::from(BP);
        let total_collateral = amount * price;

        ((&total_collateral * loan_to_value) / bp) / Self::BigUint::from(10u64).pow(decimals as u32)
    }

    fn compute_health_factor(
        &self,
        collateral_value_in_dollars: &Self::BigUint,
        borrowed_value_in_dollars: &Self::BigUint,
        liquidation_threshold: &Self::BigUint,
    ) -> Self::BigUint {
        let bp = self.get_base_precision();

        let allowed_collateral_in_dollars = collateral_value_in_dollars * liquidation_threshold;

        let health_factor = &allowed_collateral_in_dollars / borrowed_value_in_dollars;

        health_factor / bp
    }

    fn get_base_precision(&self) -> Self::BigUint {
        Self::BigUint::from(BP)
    }

    fn rule_of_three(
        &self,
        value: &Self::BigUint,
        part: &Self::BigUint,
        total: &Self::BigUint,
    ) -> Self::BigUint {
        &(value * part) / total
    }
}
