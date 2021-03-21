extern crate router;
use router::*;
use elrond_wasm::*;
use elrond_wasm_debug::*;

fn contract_map() -> ContractMap<TxContext> {
	let mut contract_map = ContractMap::new();
	contract_map.register_contract(
		"file:../output/router.wasm",
		Box::new(|context| Box::new(RouterImpl::new(context))),
	);
	contract_map
}

#[test]
fn test_deploy() {
	parse_execute_mandos("mandos/router-init.scen.json", &contract_map());
}

#[test]
fn test_create_pool() {
	parse_execute_mandos("mandos/router-create-pool.scen.json", &contract_map())
}
