#[test]
fn test_deploy() {
    elrond_wasm_debug::mandos_go("mandos/liquidity_pool-init.scen.json");
}

#[test]
fn test() {
    elrond_wasm_debug::mandos_go("mandos/liquidity_pool-deposit.scen.json");
}

#[test]
fn test_get_interest() {
    elrond_wasm_debug::mandos_go("mandos/liquidity_pool-get-interest.scen.json");
}
