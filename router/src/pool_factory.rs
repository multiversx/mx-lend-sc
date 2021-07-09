elrond_wasm::imports!();

#[elrond_wasm_derive::module(PoolFactoryModuleImpl)]
pub trait PoolFactoryModule {
    fn init(&self) {}

    fn create_pool(
        &self,
        base_asset: &TokenIdentifier,
        lending_pool_address: &Address,
        r_base: BigUint,
        r_slope1: BigUint,
        r_slope2: BigUint,
        u_optimal: BigUint,
        reserve_factor: BigUint,
        bytecode: &BoxedBytes,
    ) -> Address {
        let code_metadata = CodeMetadata::UPGRADEABLE;
        let amount = BigUint::from(0u32);

        let mut arg_buffer = ArgBuffer::new();
        arg_buffer.push_argument_bytes(base_asset.as_esdt_identifier());
        arg_buffer.push_argument_bytes(lending_pool_address.as_bytes());
        arg_buffer.push_argument_bytes(r_base.to_bytes_be().as_slice());
        arg_buffer.push_argument_bytes(r_slope1.to_bytes_be().as_slice());
        arg_buffer.push_argument_bytes(r_slope2.to_bytes_be().as_slice());
        arg_buffer.push_argument_bytes(u_optimal.to_bytes_be().as_slice());
        arg_buffer.push_argument_bytes(reserve_factor.to_bytes_be().as_slice());

        self.send().deploy_contract(
            self.get_gas_left(),
            &amount,
            bytecode,
            code_metadata,
            &arg_buffer,
        )
    }

    // can be implemented when upgrade is available in elrond-wasm
    fn upgrade_pool(&self, _pool_address: &Address, _new_bytecode: &BoxedBytes) -> bool {
        true
    }
}
