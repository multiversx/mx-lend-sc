#![allow(clippy::too_many_arguments)]

elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait FactoryModule {
    fn create_pool(
        &self,
        base_asset: &TokenIdentifier,
        lending_pool_address: &ManagedAddress,
        r_base: BigUint,
        r_slope1: BigUint,
        r_slope2: BigUint,
        u_optimal: BigUint,
        reserve_factor: BigUint,
        bytecode: &ManagedBuffer,
    ) -> ManagedAddress {
        let code_metadata = CodeMetadata::UPGRADEABLE;
        let amount = self.types().big_uint_zero();

        let mut arg_buffer = ManagedArgBuffer::new_empty(self.type_manager());
        arg_buffer.push_arg(base_asset);
        arg_buffer.push_arg(lending_pool_address);
        arg_buffer.push_arg(r_base);
        arg_buffer.push_arg(r_slope1);
        arg_buffer.push_arg(r_slope2);
        arg_buffer.push_arg(u_optimal);
        arg_buffer.push_arg(reserve_factor);

        let (address, _) = self.raw_vm_api().deploy_contract(
            self.blockchain().get_gas_left(),
            &amount,
            bytecode,
            code_metadata,
            &arg_buffer,
        );

        address
    }

    // can be implemented when upgrade is available in elrond-wasm
    fn upgrade_pool(&self, _pool_address: &Address, _new_bytecode: &ManagedBuffer) -> bool {
        true
    }
}
