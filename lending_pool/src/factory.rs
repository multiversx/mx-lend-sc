#![allow(clippy::too_many_arguments)]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

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
    ) -> SCResult<ManagedAddress> {
        require!(
            !self.liq_pool_template_address().is_empty(),
            "liquidity pool contract template is empty"
        );

        let mut arg_buffer = ManagedArgBuffer::new_empty(self.type_manager());
        arg_buffer.push_arg(base_asset);
        arg_buffer.push_arg(lending_pool_address);
        arg_buffer.push_arg(r_base);
        arg_buffer.push_arg(r_slope1);
        arg_buffer.push_arg(r_slope2);
        arg_buffer.push_arg(u_optimal);
        arg_buffer.push_arg(reserve_factor);

        let (new_address, _) = self.raw_vm_api().deploy_from_source_contract(
            self.blockchain().get_gas_left(),
            &BigUint::zero(),
            &self.liq_pool_template_address().get(),
            CodeMetadata::UPGRADEABLE,
            &arg_buffer,
        );

        Ok(new_address)
    }

    fn upgrade_pool(
        &self,
        lp_address: ManagedAddress,
        base_asset: &TokenIdentifier,
        lending_pool_address: &ManagedAddress,
        r_base: BigUint,
        r_slope1: BigUint,
        r_slope2: BigUint,
        u_optimal: BigUint,
        reserve_factor: BigUint,
    ) -> SCResult<()> {
        require!(
            !self.liq_pool_template_address().is_empty(),
            "liquidity pool contract template is empty"
        );

        let mut arg_buffer = ManagedArgBuffer::new_empty(self.type_manager());
        arg_buffer.push_arg(base_asset);
        arg_buffer.push_arg(lending_pool_address);
        arg_buffer.push_arg(r_base);
        arg_buffer.push_arg(r_slope1);
        arg_buffer.push_arg(r_slope2);
        arg_buffer.push_arg(u_optimal);
        arg_buffer.push_arg(reserve_factor);

        self.raw_vm_api().upgrade_from_source_contract(
            &lp_address,
            self.blockchain().get_gas_left(),
            &BigUint::zero(),
            &self.liq_pool_template_address().get(),
            CodeMetadata::UPGRADEABLE,
            &arg_buffer,
        );

        Ok(())
    }

    #[view(getLiqPoolTemplateAddress)]
    #[storage_mapper("liq_pool_template_address")]
    fn liq_pool_template_address(&self) -> SingleValueMapper<ManagedAddress>;
}
