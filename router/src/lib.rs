#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use elrond_wasm::{only_owner, require, sc_error};

pub mod pool_factory;
pub use pool_factory::*;

const EMPTY_TOKEN_ID: &[u8] = b"EGLD";
const LEND_TOKEN_PREFIX: &[u8] = b"L";
const BORROW_TOKEN_PREFIX: &[u8] = b"B";

const ISSUE_ENDPOINT: &[u8] = b"issue";

#[elrond_wasm_derive::contract(RouterImpl)]
pub trait Router {
    #[module(PoolFactoryModuleImpl)]
    fn pool_factory(&self) -> PoolFactoryModuleImpl<T, BigInt, BigUint>;

    #[init]
    fn init(&self) {
        self.pool_factory().init();
    }

    /// ENDPOINTS

    #[endpoint(createLiquidityPool)]
    fn create_liquidity_pool(
        &self,
        base_asset: TokenIdentifier,
        lending_pool_address: Address,
        r_base: BigUint,
        r_slope1: BigUint,       
        r_slope2: BigUint,       
        u_optimal: BigUint,      
        reserve_factor: BigUint,
        pool_bytecode: BoxedBytes,
    ) -> SCResult<Address> {
        only_owner!(self, "only owner can create new pools");

        require!(
            !self.pools_map().contains_key(&base_asset.clone()),
            "asset already supported"
        );
        require!(base_asset.is_esdt(), "non-ESDT asset provided");

        let address = self.pool_factory().create_pool(
            &base_asset,
            &lending_pool_address,
            r_base,
            r_slope1,
            r_slope2,
            u_optimal,
            reserve_factor,
             &pool_bytecode
        );

        if !address.is_zero() {
            self.pools_map().insert(base_asset, address.clone());
            self.pools_allowed().insert(address.clone(), true);
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

    #[payable("EGLD")]
    #[endpoint(issueLendToken)]
    fn issue_lend_token(
        &self,
        token_ticker: TokenIdentifier,
        token_supply: BigUint,
        num_decimals: u8,
        #[payment] amount: BigUint,
    ) -> SCResult<()> {
        only_owner!(self, "only owner may call this function");

        let pool_address = self.pools_map().get(&token_ticker.clone()).unwrap();

        let args = self.prepare_issue_args(
            token_ticker, 
            token_supply, 
            num_decimals, 
            LEND_TOKEN_PREFIX
        );

        self.send().execute_on_dest_context_raw(
            self.get_gas_left(),
            &pool_address,
            &amount,
            ISSUE_ENDPOINT,
            &args,
        );

        Ok(())
    }

    #[payable("EGLD")]
    #[endpoint(issueBorrowToken)]
    fn issue_borrow_token(
        &self,
        token_ticker: TokenIdentifier,
        token_supply: BigUint,
        num_decimals: u8,
        #[payment] amount: BigUint,
    ) -> SCResult<()> {
        only_owner!(self, "only owner may call this function");

        let pool_address = self.pools_map().get(&token_ticker.clone()).unwrap();

        let args = self.prepare_issue_args(
            token_ticker,
            token_supply,
            num_decimals,
            BORROW_TOKEN_PREFIX,
        );

        self.send().execute_on_dest_context_raw(
            self.get_gas_left(),
            &pool_address,
            &amount,
            ISSUE_ENDPOINT,
            &args,
        );

        Ok(())
    }

    #[endpoint(setLendTokenAddress)]
    fn set_lend_asset_address(&self, lend_ticker: TokenIdentifier) -> SCResult<()> {
        let caller = self.get_caller();
        let is_pool_allowed = self.pools_allowed().get(&caller).unwrap_or_default();
        require!(is_pool_allowed, "access restricted: unknown caller address");

        require!(lend_ticker != EMPTY_TOKEN_ID, "invalid ticker provided");

        self.pools_map().insert(lend_ticker, caller);

        Ok(())
    }

    #[endpoint(setBorrowTokenAddress)]
    fn set_borrow_asset_address(&self, borrow_ticker: TokenIdentifier) -> SCResult<()> {
        let caller = self.get_caller();
        let is_pool_allowed = self.pools_allowed().get(&caller).unwrap_or_default();
        require!(is_pool_allowed, "access restricted: unknown caller address");

        require!(borrow_ticker != EMPTY_TOKEN_ID, "invalid ticker provided");

        self.pools_map().insert(borrow_ticker, caller);

        Ok(())
    }

    /// VIEWS

    #[view(getPoolAddress)]
    fn get_pool_address(&self, asset: TokenIdentifier) -> Address {
        self.pools_map().get(&asset).unwrap_or(Address::zero())
    }

    /// UTILS

    fn prepare_issue_args(
        &self,
        token_ticker: TokenIdentifier,
        token_supply: BigUint,
        num_decimals: u8,
        prefix: &[u8],
    ) -> ArgBuffer {
        let mut args = ArgBuffer::new();
        args.push_argument_bytes(token_ticker.as_esdt_identifier());
        args.push_argument_bytes(token_ticker.as_name());
        args.push_argument_bytes(prefix);
        args.push_argument_bytes(token_supply.to_bytes_be().as_slice());
        args.push_argument_bytes(&[num_decimals]);

        return args;
    }

    //
    /// STORAGE

    #[storage_mapper("pools_map")]
    fn pools_map(&self) -> MapMapper<Self::Storage, TokenIdentifier, Address>;

    #[storage_mapper("pool_allowed")]
    fn pools_allowed(&self) -> MapMapper<Self::Storage, Address, bool>;
}
