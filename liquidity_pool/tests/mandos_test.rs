extern crate liquidity_pool;
use liquidity_pool::*;
use elrond_wasm::*;
use elrond_wasm_debug::*;

fn contract_map() -> ContractMap<TxContext> {
	let mut contract_map = ContractMap::new();
	contract_map.register_contract(
		"file:../output/liquidity-pool.wasm",
		Box::new(|context| Box::new(LiquidityPoolImpl::new(context))),
	);
	contract_map
}

#[test]
fn test_deploy() {
	parse_execute_mandos("mandos/liquidity_pool-init.scen.json", &contract_map());
}

#[test]
fn test() {
	parse_execute_mandos("mandos/liquidity_pool-deposit.scen.json", &contract_map());
}
