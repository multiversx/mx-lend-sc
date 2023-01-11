elrond_wasm::imports!();

use common_structs::BP;

#[elrond_wasm::module]
pub trait LendingMathModule {
    fn compute_health_factor(
        &self,
        collateral_value_in_dollars: &BigUint,
        borrowed_value_in_dollars: &BigUint,
        liquidation_threshold: &BigUint,
    ) -> BigUint {
        let allowed_collateral_in_dollars = collateral_value_in_dollars * liquidation_threshold;
        let health_factor = &allowed_collateral_in_dollars / borrowed_value_in_dollars;

        health_factor / BP
    }
}
