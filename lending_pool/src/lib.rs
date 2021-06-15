#![no_std]
#![allow(non_snake_case)]

use elrond_wasm::{contract_call, only_owner, require, sc_error};

mod proxies;
use proxies::*;

pub mod models;
pub use models::*;



use elrond_wasm::*;


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
        #[payment] amount: BigUint,
        #[var_args] caller: OptionalArg<Address>
    ) -> SCResult<()> {

        let lend_token = self.call_value().token();
        let nft_nonce = self.call_value().esdt_token_nonce();

        let initial_caller = caller.into_option().unwrap_or_else(|| self.get_caller());

        require!(amount > 0, "amount must be greater than 0");
        require!(!initial_caller.is_zero(), "invalid address provided");

        let pool_address = self.get_pool_address(lend_token.clone());
        require!(!pool_address.is_zero(), "invalid liquidity pool address");

        let mut args = ArgBuffer::new();
      /*  args.push_argument_bytes(lend_token.as_esdt_identifier());
        args.push_argument_bytes(&nft_nonce.to_be_bytes()[..]);
        args.push_argument_bytes(amount.to_bytes_be().as_slice());
        args.push_argument_bytes(pool_address.as_bytes());
        args.push_argument_bytes(b"withdraw");*/
        args.push_argument_bytes(initial_caller.as_bytes());

        /*self.send().execute_on_dest_context_raw(
            self.get_gas_left()/2,
            &self.get_sc_address(),
            &BigUint::zero(),
            ESDT_TRANSFER_STRING,
            &args,
        );*/

       
        self.send().direct_esdt_nft_execute(
            &pool_address,
            &lend_token.as_esdt_identifier(),
            nft_nonce,
            &amount,
            self.get_gas_left()/2,
            b"withdraw",
            &args
        );

        Ok(())
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

        let nft_nonce = self.call_value().esdt_token_nonce();

        let mut args = ArgBuffer::new();
        args.push_argument_bytes(initial_caller.as_bytes());
         
        self.send().direct_esdt_nft_execute(
            &asset_address,
            &borrow_token.as_esdt_identifier(),
            nft_nonce,
            &amount,
            self.get_gas_left()/2,
            b"lockBTokens",
            &args
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
            .with_token_transfer(asset, amount)
            .repay(repay_unique_id)
            .execute_on_dest_context(self.get_gas_left(), self.send());

        let collateral_token_address = self.get_pool_address(results.collateral_identifier.clone());

        require!(
            collateral_token_address != Address::zero(),
            "asset is not supported"
        );
        
        contract_call!(self, collateral_token_address, LiquidtyPoolProxy)
            .mintLendTokens(caller,results.collateral_identifier, results.amount, results.collateral_timestamp)
            .execute_on_dest_context(self.get_gas_left() / 2, self.send());

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

        /*let collateral_token_address = self
            .pools_map()
            .get(&results.collateral_token)
            .unwrap_or_else(Address::zero);*/

        let collateral_token_address = self.get_pool_address(results.collateral_token.clone());

        self.set_token_identifier_liq(results.collateral_token.clone());
        self.set_token_amount_liq(results.amount.clone());
       
        require!(
            collateral_token_address != Address::zero(),
            "asset is not supported"
        );

        let mut args_mint_lend = ArgBuffer::new();
        args_mint_lend.push_argument_bytes(caller.as_bytes());
        args_mint_lend.push_argument_bytes(&self.get_block_timestamp().to_be_bytes()[..]);


        contract_call!(self, collateral_token_address, LiquidtyPoolProxy)
            .mintLendTokens(caller, results.collateral_token, results.amount, self.get_block_timestamp())
            .execute_on_dest_context(self.get_gas_left() / 2, self.send());
        
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

        let nft_nonce = self.call_value().esdt_token_nonce();

        let esdt_nft_data = self.get_esdt_token_data(
            &self.get_sc_address(),
            asset_collateral_lend_token.as_esdt_identifier(),
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

        contract_call!(self, borrow_token_pool_address, LiquidtyPoolProxy)
            .borrow(initial_caller.clone(), asset_collateral_lend_token.clone(), amount.clone(), metadata.timestamp)
            .execute_on_dest_context(self.get_gas_left()/2, self.send());


        /*contract_call!(self, collateral_token_pool_address, LiquidtyPoolProxy)
            .with_token_transfer(asset_collateral_lend_token, amount)
            .burnLendTokens(initial_caller)
            .execute_on_dest_context(self.get_gas_left(), self.send());*/

        
        let mut args_burn_lend = ArgBuffer::new();
        args_burn_lend.push_argument_bytes(initial_caller.as_bytes());

        self.send().direct_esdt_nft_execute(
            &collateral_token_pool_address,
            &asset_collateral_lend_token.as_esdt_identifier(),
            nft_nonce,
            &amount,
            self.get_gas_left()/2,
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
            let pool_address = contract_call!(self, router_address, RouterProxy)
                .getPoolAddress(asset.clone())
                .execute_on_dest_context(self.get_gas_left(), self.send());

            self.pools_map().insert(asset, pool_address.clone());
            return pool_address;
        }
        return self.pools_map().get(&asset).unwrap_or_else(Address::zero);
    }

    #[endpoint(setTickerAfterIssue)]
    fn set_ticker_after_issue(&self, token_ticker: TokenIdentifier) -> SCResult<()> {
        let caller = self.get_caller();
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
    fn set_token_amount_liq(&self, amount: BigUint);

    #[view(tokenAmountLiq)]
    #[storage_get("tokenAmountLiq")]
    fn get_token_amount_liq(&self) -> BigUint;

    /// router address
    #[storage_mapper("router")]
    fn router(&self) -> SingleValueMapper<Self::Storage, Address>;

    //
    /// mapping for tokens to their liquidity pools addresses
    #[storage_mapper("pools_map")]
    fn pools_map(&self) -> MapMapper<Self::Storage, TokenIdentifier, Address>;
}
