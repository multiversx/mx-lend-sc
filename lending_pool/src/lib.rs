#![no_std]

use elrond_wasm::{require, sc_error};

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[elrond_wasm_derive::callable(LiquidityPoolProxy)]
pub trait LiquidityPool {
    fn deposit_asset(
        &self,
        initial_caller: Address,
        #[payment_token] asset: TokenIdentifier,
        #[payment] amount: BigUint,
    ) -> ContractCall<BigUint>;
}

#[elrond_wasm_derive::contract(LendingPoolImpl)]
pub trait LendingPool {
    #[init]
    fn init(&self) {}

    #[payable("*")]
    #[endpoint]
    fn deposit(
        &self,
        initial_caller: Address,
        #[payment_token] asset: TokenIdentifier,
        #[payment] amount: BigUint,
    ) -> SCResult<AsyncCall<BigUint>> {

        require!(amount > 0, "amount must be greater than 0");
        require!(!initial_caller.is_zero(), "invalid address provided");
        require!(self.pools_map().contains_key(&asset.clone()), "asset not supported");

        let pool_address = self
            .pools_map()
            .get(&asset.clone())
            .unwrap_or(Address::zero());

        require!(!pool_address.is_zero(), "invalid liquidity pool address");

        let mut contract_call = ContractCall::new(
            pool_address,
            asset,
            amount,
            BoxedBytes::from(&b"deposit_asset"[..])
        );
        contract_call.push_argument_raw_bytes(initial_caller.as_bytes());

        Ok(contract_call.async_call()
            .with_callback(
                self.callbacks().deposit_callback()
            )
        )
    }

    #[payable("*")]
    #[endpoint]
    fn withdraw(&self) -> SCResult<()> {
        Ok(())
    }

    #[payable("*")]
    #[endpoint]
    fn repay(&self) -> SCResult<()> {
        Ok(())
    }

    #[payable("*")]
    #[endpoint]
    fn borrow(&self) -> SCResult<()> {
        Ok(())
    }

    #[callback]
    fn deposit_callback(&self, #[call_result] result: AsyncCallResult<()>) {
        match result {
            AsyncCallResult::Ok(_) => {}
            AsyncCallResult::Err(_) => {}
        }
    }

    #[endpoint(setPoolAddress)]
    fn set_pool_address(
        &self, 
        base_asset: TokenIdentifier, 
        pool_address: Address
    ) -> SCResult<()> {

        require!(
            !self.pools_map().contains_key(&base_asset.clone()),
            "asset already supported"
        );

        self.pools_map().insert(base_asset, pool_address);

        Ok(())
    }

    #[view(getPoolAddress)]
    fn get_pool_address(&self, base_asset: TokenIdentifier) -> SCResult<Address> {
        Ok(self.pools_map().get(&base_asset).unwrap_or(Address::zero()))
    }

    #[storage_mapper("pools_map")]
    fn pools_map(&self) -> MapMapper<Self::Storage, TokenIdentifier, Address>;
}
