extern crate lending_pool;
use lending_pool::*;
use elrond_wasm::*;
use elrond_wasm_debug::*;

fn contract_map() -> ContractMap<TxContext> {
	let mut contract_map = ContractMap::new();
	contract_map.register_contract(
		"file:../output/lending_pool.wasm",
		Box::new(|context| Box::new(LendingPoolImpl::new(context))),
	);
	contract_map
}

#[test]
fn test_deploy() {
	parse_execute_mandos("mandos/lending_pool-init.scen.json", &contract_map());
}
