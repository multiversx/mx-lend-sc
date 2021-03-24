#![no_std]

use elrond_wasm::{require, sc_error};

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

const ESDT_TRANSFER_STRING: &[u8] = b"ESDTTransfer";

#[elrond_wasm_derive::contract(LendingPoolImpl)]
pub trait LendingPool {
    #[init]
    fn init(&self) {}

    #[storage_set("debug")]
    fn set_debug(&self, value: u8);

    #[view]
    #[storage_get("debug")]
    fn get_debug(&self) -> u8;

    #[payable("*")]
    #[endpoint]
    fn deposit(
        &self,
        #[var_args] caller: OptionalArg<Address>,
        #[payment_token] asset: TokenIdentifier,
        #[payment] amount: BigUint,
    ) -> SCResult<()> {
        
        let initial_caller = caller.into_option().unwrap_or(self.get_caller());

        require!(amount > 0, "amount must be greater than 0");
        require!(!initial_caller.is_zero(), "invalid address provided");
        require!(self.pools_map().contains_key(&asset.clone()), "asset not supported");

        let pool_address = self
            .pools_map()
            .get(&asset.clone())
            .unwrap_or(Address::zero());

        require!(!pool_address.is_zero(), "invalid liquidity pool address");
        
        let mut args = ArgBuffer::new();
        args.push_argument_bytes(asset.as_esdt_identifier());
        args.push_argument_bytes(amount.to_bytes_be().as_slice());
        args.push_argument_bytes(b"deposit_asset");
        args.push_argument_bytes(initial_caller.as_bytes());

        self.send().execute_on_dest_context_raw(
            self.get_gas_left(), 
            &pool_address,
            &BigUint::zero(), 
            ESDT_TRANSFER_STRING, 
            &args
        );
        
        Ok(())
    }

    #[payable("*")]
    #[endpoint]
    fn withdraw(
        &self,
        asset_to_withdraw: TokenIdentifier,
        #[var_args] initial_caller: OptionalArg<Address>,
        #[paymnet_token] lend_token: TokenIdentifier,
        #[payment] amount: BigUint
    ) -> SCResult<()> {
        let caller = initial_caller.into_option().unwrap_or(self.get_caller());

        require!(amount > 0, "amount must be greater than 0");
        require!(!caller.is_zero(), "invalid address provided");
        require!(self.pools_map().contains_key(&asset_to_withdraw.clone()), "asset not supported");


        let asset_token_pool_address = self
            .pools_map()
            .get(&asset_to_withdraw.clone())
            .unwrap_or(Address::zero());

        require!(!asset_token_pool_address.is_zero(), "invalid liquidity pool address");

        let mut args = ArgBuffer::new();
        args.push_argument_bytes(lend_token.as_esdt_identifier());
        args.push_argument_bytes(amount.to_bytes_be().as_slice());
        args.push_argument_bytes(b"withdraw");
        args.push_argument_bytes(caller.as_bytes());

        self.send().execute_on_dest_context_raw(
            self.get_gas_left(), 
            &asset_token_pool_address,
            &BigUint::zero(), 
            ESDT_TRANSFER_STRING, 
            &args
        );

        Ok(())
    }

    #[payable("*")]
    #[endpoint]
    fn repay(&self) -> SCResult<()> {
        Ok(())
    }

    #[payable("*")]
    #[endpoint]
    fn borrow(
        &self,
        asset_to_put_as_collateral: TokenIdentifier,
        asset_to_borrow: TokenIdentifier,
        #[var_args] caller: OptionalArg<Address>,
        #[payment_token] asset_collateral_lend_token: TokenIdentifier,
        #[payment] amount: BigUint,
    ) -> SCResult<()> {
        
        let initial_caller = caller.into_option().unwrap_or(self.get_caller());

        require!(amount > 0, "amount must be greater than 0");
        require!(!initial_caller.is_zero(), "invalid address provided");
        require!(self.pools_map().contains_key(&asset_to_put_as_collateral.clone()), "asset not supported");
        require!(self.pools_map().contains_key(&asset_to_borrow.clone()), "asset not supported");

        let collateral_token_pool_address = self
            .pools_map()
            .get(&asset_to_put_as_collateral.clone())
            .unwrap_or(Address::zero());

        let borrow_token_pool_address = self
            .pools_map()
            .get(&asset_to_borrow.clone())
            .unwrap_or(Address::zero());


        require!(!collateral_token_pool_address.is_zero(), "invalid liquidity pool address");
        require!(!borrow_token_pool_address.is_zero(), "invalid liquidity pool address");


        let mut args_add_collateral = ArgBuffer::new();
        args_add_collateral.push_argument_bytes(asset_collateral_lend_token.as_esdt_identifier());
        args_add_collateral.push_argument_bytes(amount.to_bytes_be().as_slice());
        args_add_collateral.push_argument_bytes(b"addCollateral");
        


        self.send().execute_on_dest_context_raw(
            self.get_gas_left(), 
            &collateral_token_pool_address,
            &BigUint::zero(), 
            ESDT_TRANSFER_STRING, 
            &args_add_collateral
        );
        
        let mut args_borrow = ArgBuffer::new();
        args_borrow.push_argument_bytes(amount.to_bytes_be().as_slice());
        args_borrow.push_argument_bytes(b"borrow");
        args_borrow.push_argument_bytes(initial_caller.as_bytes());


        self.send().execute_on_dest_context_raw(
            self.get_gas_left(), 
            &collateral_token_pool_address,
            &BigUint::zero(), 
            ESDT_TRANSFER_STRING, 
            &args_borrow
        );

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
