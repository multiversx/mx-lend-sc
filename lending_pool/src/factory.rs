#![allow(clippy::too_many_arguments)]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub mod liq_pool_proxy {
    multiversx_sc::imports!();

    #[multiversx_sc::proxy]
    pub trait LiqPoolProxy {
        #[init]
        fn init(
            &self,
            asset: TokenIdentifier,
            r_base: BigUint,
            r_slope1: BigUint,
            r_slope2: BigUint,
            u_optimal: BigUint,
            reserve_factor: BigUint,
            liquidation_threshold: BigUint,
        );
    }
}

#[multiversx_sc::module]
pub trait FactoryModule {
    fn create_pool(
        &self,
        base_asset: TokenIdentifier,
        r_base: BigUint,
        r_slope1: BigUint,
        r_slope2: BigUint,
        u_optimal: BigUint,
        reserve_factor: BigUint,
        liquidation_threshold: BigUint,
    ) -> ManagedAddress {
        require!(
            !self.liq_pool_template_address().is_empty(),
            "liquidity pool contract template is empty"
        );

        let (new_address, _) = self
            .liq_pool_proxy_obj(ManagedAddress::zero())
            .init(
                base_asset,
                r_base,
                r_slope1,
                r_slope2,
                u_optimal,
                reserve_factor,
                liquidation_threshold,
            )
            .deploy_from_source::<()>(
                &self.liq_pool_template_address().get(),
                CodeMetadata::UPGRADEABLE,
            );

        new_address
    }

    fn upgrade_pool(
        &self,
        lp_address: ManagedAddress,
        base_asset: TokenIdentifier,
        r_base: BigUint,
        r_slope1: BigUint,
        r_slope2: BigUint,
        u_optimal: BigUint,
        reserve_factor: BigUint,
        liquidation_threshold: BigUint,
    ) {
        require!(
            !self.liq_pool_template_address().is_empty(),
            "liquidity pool contract template is empty"
        );

        self.liq_pool_proxy_obj(lp_address)
            .init(
                base_asset,
                r_base,
                r_slope1,
                r_slope2,
                u_optimal,
                reserve_factor,
                liquidation_threshold,
            )
            .upgrade_from_source(
                &self.liq_pool_template_address().get(),
                CodeMetadata::UPGRADEABLE,
            );
    }

    #[view(getLiqPoolTemplateAddress)]
    #[storage_mapper("liq_pool_template_address")]
    fn liq_pool_template_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[proxy]
    fn liq_pool_proxy_obj(&self, sc_address: ManagedAddress) -> liq_pool_proxy::Proxy<Self::Api>;
}
