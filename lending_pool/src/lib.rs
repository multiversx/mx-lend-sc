#![no_std]
#![allow(non_snake_case)]

use elrond_wasm::{contract_call, only_owner, require, sc_error};

mod proxies;
use proxies::*;

pub mod models;
pub use models::*;

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

const ESDT_TRANSFER_STRING: &[u8] = b"ESDTNFTTransfer";

#[elrond_wasm_derive::contract(LendingPoolImpl)]
pub trait LendingPool {
    #[init]
    fn init(&self) {}

    #[payable("*")]
    #[endpoint]
    fn deposit(
        &self,
        #[var_args] caller: OptionalArg<Address>,
        #[payment_token] asset: TokenIdentifier,
        #[payment] amount: BigUint,
    ) -> SCResult<()> {
        let initial_caller = caller.into_option().unwrap_or_else(|| self.get_caller());

        require!(amount > 0, "amount must be greater than 0");
        require!(!initial_caller.is_zero(), "invalid address provided");
        
        let pool_address = self.get_pool_address(asset.clone());
        require!(!pool_address.is_zero(), "invalid liquidity pool address");

        Ok(contract_call!(self, pool_address, LiquidtyPoolProxy)
            .with_token_transfer(asset, amount)
            .deposit_asset(initial_caller)
            .execute_on_dest_context(self.get_gas_left() / 2, self.send()))
    }

    #[payable("*")]
    #[endpoint]
    fn withdraw(
        &self,
        #[var_args] caller: OptionalArg<Address>,
        #[paymnet_token] lend_token: TokenIdentifier,
        #[payment] amount: BigUint,
    ) -> SCResult<()> {
        let initial_caller = caller.into_option().unwrap_or_else(|| self.get_caller());

        require!(amount > 0, "amount must be greater than 0");
        require!(!initial_caller.is_zero(), "invalid address provided");

        let pool_address = self.get_pool_address(lend_token.clone());
        require!(!pool_address.is_zero(), "invalid liquidity pool address");

        Ok(contract_call!(self, pool_address, LiquidtyPoolProxy)
            .with_token_transfer(lend_token, amount)
            .withdraw(initial_caller)
            .execute_on_dest_context(self.get_gas_left(), self.send()))
    }

    #[payable("*")]
    #[endpoint(lockBTokens)]
    fn lock_b_tokens(
        &self,
        asset_to_repay: TokenIdentifier,
        #[var_args] caller: OptionalArg<Address>,
        #[payment_token] borrow_token: TokenIdentifier,
        #[payment] amount: BigUint,
    ) -> SCResult<()> {
        let initial_caller = caller.into_option().unwrap_or_else(|| self.get_caller());

        require!(amount > 0, "amount must be greater than 0");
        require!(!initial_caller.is_zero(), "invalid address provided");

        let asset_address = self.get_pool_address(asset_to_repay.clone());

        require!(
            self.pools_map().contains_key(&asset_to_repay),
            "asset not supported"
        );

        let mut args = ArgBuffer::new();
        args.push_argument_bytes(borrow_token.as_esdt_identifier());
        args.push_argument_bytes(amount.to_bytes_be().as_slice());
        args.push_argument_bytes(b"lockBTokens");
        args.push_argument_bytes(initial_caller.as_bytes());

        self.send().execute_on_dest_context_raw(
            self.get_gas_left(),
            &asset_address,
            &BigUint::zero(),
            ESDT_TRANSFER_STRING,
            &args,
        );

        Ok(())
    }

    #[payable("*")]
    #[endpoint]
    fn repay(
        &self,
        repay_unique_id: H256,
        #[var_args] initial_caller: OptionalArg<Address>,
        #[payment_token] asset: TokenIdentifier,
        #[payment] amount: BigUint,
    ) -> SCResult<()> {
        let caller = initial_caller
            .into_option()
            .unwrap_or_else(|| self.get_caller());

        require!(amount > 0, "amount must be greater than 0");
        require!(!caller.is_zero(), "invalid address provided");
        require!(self.pools_map().contains_key(&asset), "asset not supported");

        let asset_address = self.get_pool_address(asset.clone());

        let results = contract_call!(&self, asset_address, LiquidtyPoolProxy)
            .repay(caller.clone(), repay_unique_id, asset, amount)
            .execute_on_dest_context(self.get_gas_left(), self.send());

        let collateral_token_address = self
            .pools_map()
            .get(&results.collateral_identifier)
            .unwrap_or_else(Address::zero);

        require!(
            collateral_token_address != Address::zero(),
            "asset is not supported"
        );

        let mut args_mint_lend = ArgBuffer::new();
        args_mint_lend.push_argument_bytes(results.collateral_identifier.as_esdt_identifier());
        args_mint_lend.push_argument_bytes(results.collateral_amount.to_bytes_be().as_slice());
        args_mint_lend.push_argument_bytes(b"mintLTokens");
        args_mint_lend.push_argument_bytes(caller.as_bytes());
        args_mint_lend.push_argument_bytes(&results.collateral_timestamp.to_be_bytes()[..]);

        self.send().execute_on_dest_context_raw(
            self.get_gas_left(),
            &collateral_token_address,
            &BigUint::zero(),
            ESDT_TRANSFER_STRING,
            &args_mint_lend,
        );

        Ok(())
    }

    #[payable("*")]
    #[endpoint]
    fn liquidate(
        &self,
        liquidate_unique_id: H256,
        #[var_args] initial_caller: OptionalArg<Address>,
        #[payment_token] asset: TokenIdentifier,
        #[payment] amount: BigUint,
    ) -> SCResult<()> {
        let caller = initial_caller
            .into_option()
            .unwrap_or_else(|| self.get_caller());

        require!(amount > 0, "amount must be greater than 0");
        require!(!caller.is_zero(), "invalid address provided");
        require!(self.pools_map().contains_key(&asset), "asset not supported");

        let asset_address = self.get_pool_address(asset.clone());

        let results = contract_call!(&self, asset_address, LiquidtyPoolProxy)
            .liquidate(liquidate_unique_id, asset, amount)
            .execute_on_dest_context(self.get_gas_left(), self.send());

        let collateral_token_address = self
            .pools_map()
            .get(&results.collateral_token)
            .unwrap_or_else(Address::zero);

        require!(
            collateral_token_address != Address::zero(),
            "asset is not supported"
        );

        let mut args_mint_lend = ArgBuffer::new();
        args_mint_lend.push_argument_bytes(results.collateral_token.as_esdt_identifier());
        args_mint_lend.push_argument_bytes(results.amount.to_bytes_be().as_slice());
        args_mint_lend.push_argument_bytes(b"mintLTokens");
        args_mint_lend.push_argument_bytes(caller.as_bytes());
        args_mint_lend.push_argument_bytes(&self.get_block_timestamp().to_be_bytes()[..]);

        self.send().execute_on_dest_context_raw(
            self.get_gas_left(),
            &collateral_token_address,
            &BigUint::zero(),
            ESDT_TRANSFER_STRING,
            &args_mint_lend,
        );

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
        let initial_caller = caller.into_option().unwrap_or_else(|| self.get_caller());

        require!(amount > 0, "amount must be greater than 0");
        require!(!initial_caller.is_zero(), "invalid address provided");
        require!(
            self.pools_map().contains_key(&asset_to_put_as_collateral),
            "asset not supported"
        );
        require!(
            self.pools_map().contains_key(&asset_to_borrow),
            "asset not supported"
        );

        let collateral_token_pool_address = self
            .pools_map()
            .get(&asset_to_put_as_collateral)
            .unwrap_or_else(Address::zero);

        let borrow_token_pool_address = self
            .pools_map()
            .get(&asset_to_borrow)
            .unwrap_or_else(Address::zero);

        require!(
            !collateral_token_pool_address.is_zero(),
            "invalid liquidity pool address"
        );
        require!(
            !borrow_token_pool_address.is_zero(),
            "invalid liquidity pool address"
        );

        let mut args_add_collateral = ArgBuffer::new();
        args_add_collateral.push_argument_bytes(asset_collateral_lend_token.as_esdt_identifier());
        args_add_collateral.push_argument_bytes(amount.to_bytes_be().as_slice());
        args_add_collateral.push_argument_bytes(b"addCollateral");

        self.send().execute_on_dest_context_raw(
            self.get_gas_left(),
            &collateral_token_pool_address,
            &BigUint::zero(),
            ESDT_TRANSFER_STRING,
            &args_add_collateral,
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
            &args_borrow,
        );

        Ok(())
    }

    #[endpoint(setRouterAddress)]
    fn set_router_address(&self, address: Address) -> SCResult<()> {
        only_owner!(self, "permission denied");
        self.router().set(&address);
        Ok(())
    }

    /// UTILS

    fn get_pool_address(&self, asset: TokenIdentifier) -> Address {
        if !self.pools_map().contains_key(&asset) {
            let router_address = self.router().get();
            let pool_address = contract_call!(self, router_address, RouterProxy)
                .getPoolAddress(asset.clone())
                .execute_on_dest_context(self.get_gas_left(), self.send());

            self.pools_map().insert(asset, pool_address.clone());
            return pool_address;
        }

        self.pools_map().get(&asset).unwrap_or_else(Address::zero)
    }

    /// STORAGE

    /// router address
    #[storage_mapper("router")]
    fn router(&self) -> SingleValueMapper<Self::Storage, Address>;

    //
    /// mapping for tokens to their liquidity pools addresses
    #[storage_mapper("pools_map")]
    fn pools_map(&self) -> MapMapper<Self::Storage, TokenIdentifier, Address>;
}
