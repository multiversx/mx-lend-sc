elrond_wasm::imports!();

const GAS: u64 = 60000000;

#[elrond_wasm_derive::module(PoolFactoryModuleImpl)]
pub trait PoolFactoryModule {
    fn init(&self) {}

    fn create_pool(
        &self, 
        base_asset: &TokenIdentifier,
        bytecode: &BoxedBytes
    ) -> Address {
        let code_metadata = CodeMetadata::UPGRADEABLE;
        let amount = BigUint::from(0u32);

        let mut arg_buffer = ArgBuffer::new();
        arg_buffer.push_argument_bytes(base_asset.as_slice());

        let pool_address = self.send().deploy_contract(
            GAS, 
            &amount, 
            bytecode, 
            code_metadata, 
            &arg_buffer
        );

        return pool_address;
    }

    // can be implemented when upgrade is available in elrond-wasm
    fn upgrade_pool(
        &self,
        _pool_address: &Address,
        _new_bytecode: &BoxedBytes
    ) -> bool {
        return true;
    }
}
