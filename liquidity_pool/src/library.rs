elrond_wasm::imports!();

const BP: u32 = 1000000000;

const SECONDS_IN_YEAR: u32 = 31556926;

#[elrond_wasm::module]
pub trait LibraryModule {
    fn compute_borrow_rate(
        &self,
        r_base: Self::BigUint,
        r_slope1: Self::BigUint,
        r_slope2: Self::BigUint,
        u_optimal: Self::BigUint,
        u_current: Self::BigUint,
    ) -> Self::BigUint {
        let bp = Self::BigUint::from(BP);

        let borrow_rate: Self::BigUint;
        if u_current < u_optimal {
            let utilisation_ratio = (u_current * r_slope1) / u_optimal;
            borrow_rate = r_base + utilisation_ratio;
        } else {
            let denominator = bp - u_optimal.clone();
            let numerator = (u_current - u_optimal) * r_slope2;
            borrow_rate = (r_base + r_slope1) + numerator / denominator;
        }

        borrow_rate
    }

    fn compute_deposit_rate(
        &self,
        u_current: Self::BigUint,
        borrow_rate: Self::BigUint,
        reserve_factor: Self::BigUint,
    ) -> Self::BigUint {
        let bp = Self::BigUint::from(BP);
        let loan_ratio = u_current.clone() * borrow_rate;
        let deposit_rate = u_current * loan_ratio * (bp.clone() - reserve_factor);
        deposit_rate / (bp.clone() * bp.clone() * bp)
    }

    fn compute_capital_utilisation(
        &self,
        borrowed_amount: Self::BigUint,
        total_pool_reserves: Self::BigUint,
    ) -> Self::BigUint {
        let bp = Self::BigUint::from(BP);
        (borrowed_amount * bp) / total_pool_reserves
    }

    fn compute_debt(
        &self,
        amount: Self::BigUint,
        time_diff: Self::BigUint,
        borrow_rate: Self::BigUint,
    ) -> Self::BigUint {
        let bp = Self::BigUint::from(BP);
        let secs_year = Self::BigUint::from(SECONDS_IN_YEAR);
        let time_unit_percentage = (time_diff * bp.clone()) / secs_year;

        let debt_percetange = (time_unit_percentage * borrow_rate) / bp.clone();

        if debt_percetange <= bp {
            let amount_diff = ((bp.clone() - debt_percetange) * amount.clone()) / bp;
            return amount - amount_diff;
        }

        (debt_percetange * amount) / bp
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

        amount + &(&(&percentage * amount) / &bp)
    }
}
