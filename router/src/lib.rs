#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use elrond_wasm::{only_owner, require, sc_error};

pub mod pool_factory;
pub use pool_factory::*;

#[elrond_wasm_derive::contract(RouterImpl)]
pub trait Router {
    #[module(PoolFactoryModuleImpl)]
    fn pool_factory(&self) -> PoolFactoryModuleImpl<T, BigInt, BigUint>;

    #[init]
    fn init(&self) {
        self.pool_factory().init();
    }

    #[endpoint(createLiquidityPool)]
    fn create_liquidity_pool(
        &self,
        base_asset: TokenIdentifier,
        pool_bytecode: BoxedBytes,
    ) -> SCResult<Address> {
        only_owner!(self, "only owner can create new pools");

        require!(
            !self.pools_map().contains_key(&base_asset.clone()),
            "Asset already supported"
        );
        require!(base_asset.is_esdt(), "Non-ESDT asset provided");

        let address = self.pool_factory().create_pool(&base_asset, &pool_bytecode);

        if !address.is_zero() {
            self.pools_map().insert(base_asset, address.clone());
        }

        Ok(address)
    }

    #[endpoint(upgradeLiquidityPool)]
    fn upgrade_liquidity_pool(
        &self,
        base_asset: TokenIdentifier,
        new_bytecode: BoxedBytes,
    ) -> SCResult<()> {
        only_owner!(self, "only owner can upgrade existing pools");

        require!(
            self.pools_map().contains_key(&base_asset.clone()),
            "no pool found for this asset"
        );

        let pool_address = self.pools_map().get(&base_asset).unwrap();

        let success = self
            .pool_factory()
            .upgrade_pool(&pool_address, &new_bytecode);

        if !success {
            return sc_error!("pair upgrade failed");
        }

        Ok(())
    }

    #[view(getPoolAddress)]
    fn get_pool_address(&self, base_asset: TokenIdentifier) -> SCResult<Address> {
        Ok(self.pools_map().get(&base_asset).unwrap_or(Address::zero()))
    }

    #[storage_mapper("pools_map")]
    fn pools_map(&self) -> MapMapper<Self::Storage, TokenIdentifier, Address>;
}
