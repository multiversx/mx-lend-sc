#![no_std]

pub mod library;
use core::borrow::Borrow;

pub use library::*;

pub mod models;
pub use models::*;

mod tokens;
use tokens::*;

mod storage;
use storage::*;

mod liquidity_pool;
mod utils;

use liquidity_pool::*;
use utils::*;

elrond_wasm::imports!();

use elrond_wasm::*;
use elrond_wasm::types::{TokenIdentifier, Address, SCResult, H256, VarArgs, EsdtLocalRole, AsyncCall, BoxedBytes, AsyncCallResult, OptionalArg, ArgBuffer, MultiResultVec, MultiArgVec};
use elrond_wasm::esdt::{ESDTSystemSmartContractProxy, SemiFungibleTokenProperties};
use elrond_wasm::storage::mappers::{SingleValueMapper, MapMapper};



elrond_wasm::derive_imports!();

const ESDT_ISSUE_COST: u64 = 5000000000000000000;

const LEND_TOKEN_PREFIX: &[u8] = b"L";
const BORROW_TOKEN_PREFIX: &[u8] = b"B";
const LEND_TOKEN_NAME: &[u8] = b"IntBearing";
const DEBT_TOKEN_NAME: &[u8] = b"DebtBearing";

#[elrond_wasm_derive::contract]
pub trait LiquidityPool:
    storage::StorageModule
    + tokens::TokensModule
    + library::LibraryModule
    + liquidity_pool::LiquidityPoolModule
    + utils::UtilsModule {
   /* #[module(LibraryModuleImpl)]
    fn library_module(&self) -> LibraryModuleImpl<T, BigInt, BigUint>;

    #[module(TokensModuleImpl)]
    fn tokens_module(&self) -> TokensModuleImpl<T, BigInt, BigUint>;

    #[module(StorageModuleImpl)]
    fn storage_module(&self) -> StorageModuleImpl<T, BigInt, BigUint>;

    #[module(LiquidityPoolModuleImpl)]
    fn liquidity_pool_module(&self) -> LiquidityPoolModuleImpl<T, BigInt, BigUint>;*/

    #[init]
    fn init(
        &self,
        asset: TokenIdentifier,
        lending_pool: Address,
        r_base: Self::BigUint,
        r_slope1: Self::BigUint,
        r_slope2: Self::BigUint,
        u_optimal: Self::BigUint,
        reserve_factor: Self::BigUint,
    ) {
        /*self.library_module().init();
        self.tokens_module().init();
        self.storage_module().init();
        self.liquidity_pool_module().init();*/

        self.pool_asset().set(&asset);
        self.set_lending_pool(lending_pool);
        self.debt_nonce().set(&1u64);
        self.reserve_data().set(&ReserveData {
            r_base,
            r_slope1,
            r_slope2,
            u_optimal,
            reserve_factor,
        });
    }

    #[payable("*")]
    #[endpoint(deposit_asset)]
    fn deposit_asset_endpoint(
        &self,
        initial_caller: Address,
        #[payment_token] asset: TokenIdentifier,
        #[payment] amount: Self::BigUint,
    ) -> SCResult<()> {
       self.deposit_asset(initial_caller, asset, amount)
    }

    #[endpoint(borrow)]
    fn borrow_endpoint(
        &self,
        initial_caller: Address,
        lend_token: TokenIdentifier,
        amount: Self::BigUint,
        timestamp: u64,
    ) -> SCResult<()> {
        self.borrow(initial_caller, lend_token, amount, timestamp)
    }

    #[payable("*")]
    #[endpoint(lockBTokens)]
    fn lock_b_tokens_endpoint(
        &self,
        initial_caller: Address,
        #[payment_token] borrow_token: TokenIdentifier,
        #[payment] amount: Self::BigUint,
    ) -> SCResult<H256> {
       self.lock_b_tokens(initial_caller, borrow_token, amount)
    }

    #[payable("*")]
    #[endpoint(repay)]
    fn repay_endpoint(
        &self,
        unique_id: BoxedBytes,
        #[payment_token] asset: TokenIdentifier,
        #[payment] amount: Self::BigUint,
    ) -> SCResult<RepayPostion<Self::BigUint>> {
        self.repay(unique_id, asset, amount)
    }

    #[endpoint(mintLendTokens)]
    fn mint_l_tokens_endpoint(
        &self,
        initial_caller: Address,
        lend_token: TokenIdentifier,
        amount: Self::BigUint,
        interest_timestamp: u64,
    ) -> SCResult<()> {
        self.mint_l_tokens(initial_caller, lend_token,amount,interest_timestamp);
        Ok(())
    }


    #[payable("*")]
    #[endpoint(burnLendTokens)]
    fn burn_l_tokens_endpoint(
        &self,
        initial_caller: Address,
        #[payment_token]lend_token: TokenIdentifier,
        #[payment]amount: Self::BigUint,
    ) -> SCResult<()> {
        self.burn_l_tokens(initial_caller,lend_token,amount)
    }

    #[payable("*")]
    #[endpoint(withdraw)]
    fn withdraw_endpoint(
        &self,
        initial_caller: Address,
        #[payment_token] lend_token: TokenIdentifier,
        #[payment] amount: Self::BigUint,
    ) -> SCResult<()> {
        self.withdraw(initial_caller, lend_token, amount)
    }

    #[payable("*")]
    #[endpoint(liquidate)]
    fn liquidate_endpoint(
        &self,
        position_id: BoxedBytes,
        #[payment_token] token: TokenIdentifier,
        #[payment] amount: Self::BigUint,
    ) -> SCResult<LiquidateData<Self::BigUint>> {
        self.liquidate(position_id, token, amount)
    }

    #[payable("EGLD")]
    #[endpoint(issue)]
    fn issue_endpoint(
        &self,
        plain_ticker: BoxedBytes,
        token_ticker: TokenIdentifier,
        token_prefix: BoxedBytes,
        #[payment] issue_cost: Self::BigUint,
    ) -> SCResult<AsyncCall<Self::SendApi>> {
        self.issue(plain_ticker, token_ticker, token_prefix, issue_cost)
    }

    #[endpoint(setLendTokenRoles)]
    fn set_lend_token_roles_endpoint(
        &self,
        #[var_args] roles: VarArgs<EsdtLocalRole>,
    ) -> SCResult<AsyncCall<Self::SendApi>> {
        self.set_lend_token_roles(roles)
    }

    #[endpoint(setBorrowTokenRoles)]
    fn set_borrow_token_roles_endpoint(
        &self,
        #[var_args] roles: VarArgs<EsdtLocalRole>,
    ) -> SCResult<AsyncCall<Self::SendApi>> {
        self.set_borrow_token_roles(roles)
    }


    /// VIEWS

    #[view(repayPositionsIds)]
    fn get_repay_positions_ids(&self) -> MultiResultVec<BoxedBytes> {
        let mut result = MultiArgVec::new();
        for (key,_) in self.repay_position().iter() {
            result.push(key.into());
        }
        result
    }


    #[view(repayPosition)]
    fn view_repay_position(&self, position_id: BoxedBytes) -> SCResult<RepayPostion<Self::BigUint>> {
        return Ok(self.repay_position().get(&position_id).unwrap())
    }

    #[view(debtPosition)]
    fn view_debt_position(&self, position_id: BoxedBytes) -> SCResult<DebtPosition<Self::BigUint>> {
        return Ok(self.debt_positions().get(&position_id).unwrap())
    }


    #[view(getBorrowRate)]
    fn get_borrow_rate(&self) -> Self::BigUint {
        let reserve_data = self.reserve_data().get();
        self._get_borrow_rate(reserve_data, OptionalArg::None)
    }

    #[view(getDepositRate)]
    fn get_deposit_rate(&self) -> Self::BigUint {
        let utilisation = self.get_capital_utilisation();
        let reserve_data = self.reserve_data().get();
        let reserve_factor = reserve_data.reserve_factor.clone();
        let borrow_rate =
            self._get_borrow_rate(reserve_data, OptionalArg::Some(utilisation.clone()));

        self.compute_deposit_rate(utilisation, borrow_rate, reserve_factor)
    }

    #[view(getDebtInterest)]
    fn get_debt_interest(&self, amount: Self::BigUint, timestamp: u64) -> Self::BigUint {
        let now = self.blockchain().get_block_timestamp();
        let time_diff = Self::BigUint::from(now - timestamp);

        let borrow_rate = self.get_borrow_rate();

        self.compute_debt(amount, time_diff, borrow_rate)
    }

    #[view(getPositionInterest)]
    fn get_debt_position_interest(&self, position_id: BoxedBytes) -> Self::BigUint {
        let mut debt_position = self.debt_positions().get(&position_id).unwrap_or_default();
        let interest = self.get_debt_interest(debt_position.size.clone(), debt_position.timestamp);
        return interest;
    }

    #[view(getCapitalUtilisation)]
    fn view_capital_utilisation(&self) -> Self::BigUint {
        self.get_capital_utilisation()
    }

    #[view(getReserve)]
    fn view_reserve(&self) -> Self::BigUint {
        self.reserves()
            .get(&self.pool_asset().get())
            .unwrap_or_else(Self::BigUint::zero)
    }

    #[view(poolAsset)]
    fn view_pool_asset(&self) -> TokenIdentifier {
        self.pool_asset().get()
    }

    #[view(lendToken)]
    fn view_lend_token(&self) -> TokenIdentifier {
        self.lend_token().get()
    }

    #[view(borrowToken)]
    fn view_borrow_token(&self) -> TokenIdentifier {
        self.borrow_token().get()
    }

    
    
    /// health factor threshold
    #[endpoint(setHealthFactorThreshold)]
    fn endpoint_health_factor_threshold(&self, health_factor_threashdol: u32) {
        self.set_health_factor_threshold(health_factor_threashdol);
    }

    #[view(healthFactorThreshold)]
    fn view_health_factor_threshold(&self) -> u32{
        self.get_health_factor_threshold()
    }


    #[view(getLendingPool)]
    fn view_lending_pool(&self) -> Address{
        self.get_lending_pool()
    }


    #[view(totalBorrow)]
    fn view_total_borrow(&self) -> Self::BigUint{
        self.get_total_borrow()
    }


    #[view(assetReserve)]
    fn view_asset_reserve(&self) -> Self::BigUint{
        self.get_asset_reserve()
    }


    #[view(withdrawAmount)]
    fn view_withdraw_amount(&self) -> Self::BigUint{
        self.get_withdraw_amount()
    }
    

    #[view(repayPositionAmount)]
    fn view_repay_position_amount(&self) -> Self::BigUint{
        self.get_repay_position_amount()
    }

    #[view(repayPositionIdentifier)]
    fn view_repay_position_id(&self) -> TokenIdentifier{
        self.get_repay_position_id()
    }

  

    #[view(repayPositionNonce)]
    fn view_repay_position_nonce(&self) -> u64{
        self.get_repay_position_nonce()
    }
    
}
