#![no_std]
#![allow(non_snake_case)]

pub mod models;
pub use models::*;

use elrond_wasm::*;

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

const ESDT_TRANSFER_STRING: &[u8] = b"ESDTNFTTransfer";

#[elrond_wasm_derive::contract]
pub trait LendingPool{
    #[init]
    fn init(&self) {}

    #[payable("*")]
    #[endpoint]
    fn deposit(
        &self,
        #[var_args] caller: OptionalArg<Address>,
        #[payment_token] asset: TokenIdentifier,
        #[payment] amount: Self::BigUint,
    ) -> SCResult<()> {
        let initial_caller = caller.into_option().unwrap_or_else(|| self.blockchain().get_caller());

        require!(amount > 0, "amount must be greater than 0");
        require!(!initial_caller.is_zero(), "invalid address provided");

        let pool_address = self.get_pool_address(asset.clone());
        require!(!pool_address.is_zero(), "invalid liquidity pool address");

        Ok(self.liquidity_pool_proxy(pool_address).deposit_asset_endpoint(initial_caller, asset, amount)
            .with_gas_limit(self.blockchain().get_gas_left()/2)
            .execute_on_dest_context())
    }

    #[payable("*")]
    #[endpoint]
    fn withdraw(
        &self,
        #[payment] amount: Self::BigUint,
        #[var_args] caller: OptionalArg<Address>
    ) -> SCResult<()> {

        let lend_token = self.call_value().token();
        let nft_nonce = self.call_value().esdt_token_nonce();

        let initial_caller = caller.into_option().unwrap_or_else(|| self.blockchain().get_caller());

        require!(amount > 0, "amount must be greater than 0");
        require!(!initial_caller.is_zero(), "invalid address provided");

        let pool_address = self.get_pool_address(lend_token.clone());
        require!(!pool_address.is_zero(), "invalid liquidity pool address");

        Ok(self.liquidity_pool_proxy(pool_address).withdraw_endpoint(initial_caller, lend_token, amount)
            .with_gas_limit(self.blockchain().get_gas_left()/2)
            .execute_on_dest_context())
    }

    #[payable("*")]
    #[endpoint(lockBTokens)]
    fn lock_b_tokens(
        &self,
        asset_to_repay: TokenIdentifier,
        #[var_args] caller: OptionalArg<Address>,
        #[payment_token] borrow_token: TokenIdentifier,
        #[payment] amount: Self::BigUint,
    ) -> SCResult<()> {
        let initial_caller = caller.into_option().unwrap_or_else(|| self.blockchain().get_caller());

        require!(amount > 0, "amount must be greater than 0");
        require!(!initial_caller.is_zero(), "invalid address provided");

        let asset_address = self.get_pool_address(asset_to_repay.clone());

        require!(
            self.pools_map().contains_key(&asset_to_repay),
            "asset not supported"
        );

        let nft_nonce = self.call_value().esdt_token_nonce();

        let mut args = ArgBuffer::new();
        args.push_argument_bytes(initial_caller.as_bytes());
         
        self.send().direct_esdt_nft_execute(
            &asset_address,
            &borrow_token,
            nft_nonce,
            &amount,
            self.blockchain().get_gas_left()/2,
            b"lockBTokens",
            &args
        );

        Ok(())
    }

    #[payable("*")]
    #[endpoint]
    fn repay(
        &self,
        repay_unique_id: BoxedBytes,
        #[var_args] initial_caller: OptionalArg<Address>,
        #[payment_token] asset: TokenIdentifier,
        #[payment] amount: Self::BigUint,
    ) -> SCResult<()> {
        let caller = initial_caller
            .into_option()
            .unwrap_or_else(|| self.blockchain().get_caller());

        require!(amount > 0, "amount must be greater than 0");
        require!(!caller.is_zero(), "invalid address provided");
        require!(self.pools_map().contains_key(&asset), "asset not supported");

        let asset_address = self.get_pool_address(asset.clone());

        let results = self.liquidity_pool_proxy(asset_address).repay_endpoint(repay_unique_id, asset, amount)
            .with_gas_limit(self.blockchain().get_gas_left()/2)
            .execute_on_dest_context();

        let collateral_token_address = self.get_pool_address(results.collateral_identifier.clone());

        require!(
            collateral_token_address != Address::zero(),
            "asset is not supported"
        );

       Ok(self.liquidity_pool_proxy(collateral_token_address)
            .mint_l_tokens_endpoint(caller,results.collateral_identifier, results.amount, results.collateral_timestamp)
            .with_gas_limit(self.blockchain().get_gas_left()/2)
            .execute_on_dest_context())
    }

    #[payable("*")]
    #[endpoint]
    fn liquidate(
        &self,
        liquidate_unique_id: BoxedBytes,
        #[var_args] initial_caller: OptionalArg<Address>,
        #[payment_token] asset: TokenIdentifier,
        #[payment] amount: Self::BigUint,
    ) -> SCResult<()> {
        let caller = initial_caller
            .into_option()
            .unwrap_or_else(|| self.blockchain().get_caller());

        require!(amount > 0, "amount must be greater than 0");
        require!(!caller.is_zero(), "invalid address provided");
        require!(self.pools_map().contains_key(&asset), "asset not supported");

        let asset_address = self.get_pool_address(asset.clone());

        let results = self.liquidity_pool_proxy(asset_address)
            .liquidate_endpoint(liquidate_unique_id, asset, amount)
            .with_gas_limit(self.blockchain().get_gas_left()/2)
            .execute_on_dest_context();


        let collateral_token_address = self.get_pool_address(results.collateral_token.clone());

        self.set_token_identifier_liq(results.collateral_token.clone());
        self.set_token_amount_liq(results.amount.clone());
       
        require!(
            collateral_token_address != Address::zero(),
            "asset is not supported"
        );

        self.liquidity_pool_proxy(collateral_token_address)
            .mint_l_tokens_endpoint(caller, results.collateral_token, results.amount, self.blockchain().get_block_timestamp())
            .with_gas_limit(self.blockchain().get_gas_left()/2)
            .execute_on_dest_context();
        
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
        #[payment] amount: Self::BigUint,
    ) -> SCResult<()> {
        let initial_caller = caller.into_option().unwrap_or_else(|| self.blockchain().get_caller());

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

        let nft_nonce = self.call_value().esdt_token_nonce();

        let esdt_nft_data = self.blockchain().get_esdt_token_data(
            &self.blockchain().get_sc_address(),
            &asset_collateral_lend_token,
            nft_nonce,
        );

        let metadata: InterestMetadata;
        match InterestMetadata::top_decode(esdt_nft_data.attributes.as_slice()) {
            Result::Ok(decoded) => {
                metadata = decoded;
            }
            Result::Err(_) => {
                return sc_error!("could not parse token metadata");
            }
        }

       self.liquidity_pool_proxy(borrow_token_pool_address)
            .borrow_endpoint(initial_caller.clone(), asset_collateral_lend_token.clone(), amount.clone(), metadata.timestamp)
            .with_gas_limit(self.blockchain().get_gas_left()/2)
            .execute_on_dest_context();

        let mut args_burn_lend = ArgBuffer::new();
        args_burn_lend.push_argument_bytes(initial_caller.as_bytes());

        self.send().direct_esdt_nft_execute(
            &collateral_token_pool_address,
            &asset_collateral_lend_token,
            nft_nonce,
            &amount,
            self.blockchain().get_gas_left()/2,
            b"burnLendTokens",
            &args_burn_lend
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
            let pool_address = self.router_proxy(router_address)
                .get_pool_address(asset.clone())
                .with_gas_limit(self.blockchain().get_gas_left()/2)
                .execute_on_dest_context();

            self.pools_map().insert(asset, pool_address.clone());
            return pool_address;
        }
        return self.pools_map().get(&asset).unwrap_or_else(Address::zero);
    }

    #[endpoint(setTickerAfterIssue)]
    fn set_ticker_after_issue(&self, token_ticker: TokenIdentifier) -> SCResult<()> {
        let caller = self.blockchain().get_caller();
       // let is_pool_allowed = self.pools_allowed().get(&caller).unwrap_or_default();
       // require!(is_pool_allowed, "access restricted: unknown caller address");
        require!(!token_ticker.is_egld(), "invalid ticker provided");
        self.pools_map().insert(token_ticker, caller);
        Ok(())
    }

    #[endpoint(setTicker)]
    fn set_ticker(&self, token_ticker: TokenIdentifier, pool_address: Address) -> SCResult<()> {
        require!(!token_ticker.is_egld(), "invalid ticker provided");
        self.pools_map().insert(token_ticker, pool_address);
        Ok(())
    }

    /// STORAGE

    //delete after liquidations debugging
    #[storage_set("tokenIdentifierLiq")]
    fn set_token_identifier_liq(&self, token: TokenIdentifier);

    #[view(tokenIdentifierLiq)]
    #[storage_get("tokenIdentifierLiq")]
    fn get_token_identifier_liq(&self) -> TokenIdentifier;

    #[storage_set("tokenAmountLiq")]
    fn set_token_amount_liq(&self, amount: Self::BigUint);

    #[view(tokenAmountLiq)]
    #[storage_get("tokenAmountLiq")]
    fn get_token_amount_liq(&self) -> Self::BigUint;

    /// router address
    #[storage_mapper("router")]
    fn router(&self) -> SingleValueMapper<Self::Storage, Address>;

    //
    /// mapping for tokens to their liquidity pools addresses
    #[storage_mapper("pools_map")]
    fn pools_map(&self) -> MapMapper<Self::Storage, TokenIdentifier, Address>;

    #[proxy]
    fn liquidity_pool_proxy(&self, sc_address: Address) -> liquidity_pool::Proxy<Self::SendApi>;

    #[proxy]
    fn router_proxy(&self, sc_address: Address) -> router::Proxy<Self::SendApi>;
}
