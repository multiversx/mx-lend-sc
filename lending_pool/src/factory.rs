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

        let mut arg_buffer = ArgBuffer::new();
        arg_buffer.push_argument_bytes(base_asset.as_esdt_identifier());
        arg_buffer.push_argument_bytes(lending_pool_address.as_bytes());
        arg_buffer.push_argument_bytes(r_base.to_bytes_be().as_slice());
        arg_buffer.push_argument_bytes(r_slope1.to_bytes_be().as_slice());
        arg_buffer.push_argument_bytes(r_slope2.to_bytes_be().as_slice());
        arg_buffer.push_argument_bytes(u_optimal.to_bytes_be().as_slice());
        arg_buffer.push_argument_bytes(reserve_factor.to_bytes_be().as_slice());

        self.send()
            .deploy_contract(
                self.blockchain().get_gas_left(),
                &amount,
                bytecode,
                code_metadata,
                &arg_buffer,
            )
            .unwrap_or_else(self.types().managed_address_zero())
    }

    // can be implemented when upgrade is available in elrond-wasm
    fn upgrade_pool(&self, _pool_address: &Address, _new_bytecode: &ManagedBuffer) -> bool {
        true
    }
}
