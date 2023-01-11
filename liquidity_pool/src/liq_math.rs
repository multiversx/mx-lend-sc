use common_structs::BP;

elrond_wasm::imports!();

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

        if u_current <= u_optimal {
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
        if *total_reserves == BigUint::zero() {
            total_reserves.clone()
        } else {
            &(borrowed_amount * &bp) / total_reserves
        }
    }

    fn compute_withdrawal_amount(
        &self,
        amount: &BigUint,
        current_supply_index: &BigUint,
        initial_supply_index: &BigUint,
    ) -> BigUint {
        let bp = BigUint::from(BP);
        let interest = (current_supply_index - initial_supply_index) * amount / bp;

        amount + &interest
    }

    fn compute_interest(
        &self,
        amount: &BigUint,
        current_supply_index: &BigUint,
        initial_supply_index: &BigUint,
    ) -> BigUint {
        let bp = BigUint::from(BP);

        (current_supply_index - initial_supply_index) * amount / bp
    }

    fn compute_borrowable_amount(
        &self,
        total_collateral: &BigUint,
        loan_to_value: &BigUint,
        decimals: u8,
    ) -> BigUint {
        let bp = BigUint::from(BP);

        ((total_collateral * loan_to_value) / bp) / BigUint::from(10u64).pow(decimals as u32)
    }
}
