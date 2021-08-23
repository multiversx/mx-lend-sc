#![allow(clippy::too_many_arguments)]

elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait FactoryModule {
    fn create_pool(
        &self,
        base_asset: &TokenIdentifier,
        lending_pool_address: &Address,
        r_base: Self::BigUint,
        r_slope1: Self::BigUint,
        r_slope2: Self::BigUint,
        u_optimal: Self::BigUint,
        reserve_factor: Self::BigUint,
        bytecode: &BoxedBytes,
    ) -> Address {
        let code_metadata = CodeMetadata::UPGRADEABLE;
        let amount = Self::BigUint::from(0u32);

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
            .unwrap_or_else(Address::zero)
    }

    // can be implemented when upgrade is available in elrond-wasm
    fn upgrade_pool(&self, _pool_address: &Address, _new_bytecode: &BoxedBytes) -> bool {
        true
    }
}
