#![no_std]
#![allow(clippy::too_many_arguments)]

pub mod library;

pub use library::*;

pub mod models;
pub use models::*;

mod tokens;

mod storage;

mod liquidity_pool;
mod utils;

elrond_wasm::imports!();

use elrond_wasm::types::{
    Address, AsyncCall, BoxedBytes, EsdtLocalRole, MultiArgVec, MultiResultVec, SCResult,
    TokenIdentifier, VarArgs, H256,
};
use elrond_wasm::*;

elrond_wasm::derive_imports!();

const LEND_TOKEN_PREFIX: &[u8] = b"L";
const BORROW_TOKEN_PREFIX: &[u8] = b"B";
const LEND_TOKEN_NAME: &[u8] = b"IntBearing";
const DEBT_TOKEN_NAME: &[u8] = b"DebtBearing";

#[elrond_wasm::contract]
pub trait LiquidityPool:
    storage::StorageModule
    + tokens::TokensModule
    + library::LibraryModule
    + liquidity_pool::LiquidityPoolModule
    + utils::UtilsModule
{
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
        self.pool_asset().set(&asset);
        self.lending_pool().set(&lending_pool);
        self.debt_nonce().set_if_empty(&1u64);
        self.pool_params().set(&PoolParams {
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
        #[payment_amount] amount: Self::BigUint,
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
        #[payment_amount] amount: Self::BigUint,
    ) -> SCResult<H256> {
        self.lock_b_tokens(initial_caller, borrow_token, amount)
    }

    #[payable("*")]
    #[endpoint(repay)]
    fn repay_endpoint(
        &self,
        unique_id: BoxedBytes,
        #[payment_token] asset: TokenIdentifier,
        #[payment_amount] amount: Self::BigUint,
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
        self.mint_l_tokens(initial_caller, lend_token, amount, interest_timestamp)?;
        Ok(())
    }

    #[payable("*")]
    #[endpoint(burnLendTokens)]
    fn burn_l_tokens_endpoint(
        &self,
        initial_caller: Address,
        #[payment_token] payment_token_id: TokenIdentifier,
        #[payment_nonce] payment_token_nonce: u64,
        #[payment_amount] payment_amount: Self::BigUint,
    ) -> SCResult<()> {
        self.burn_l_tokens(
            payment_token_id,
            payment_token_nonce,
            payment_amount,
            initial_caller,
        )
    }

    #[payable("*")]
    #[endpoint(withdraw)]
    fn withdraw_endpoint(
        &self,
        initial_caller: Address,
        #[payment_token] lend_token: TokenIdentifier,
        #[payment_amount] amount: Self::BigUint,
    ) -> SCResult<()> {
        self.withdraw(initial_caller, lend_token, amount)
    }

    #[payable("*")]
    #[endpoint(liquidate)]
    fn liquidate_endpoint(
        &self,
        position_id: BoxedBytes,
        #[payment_token] token: TokenIdentifier,
        #[payment_amount] amount: Self::BigUint,
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
        #[payment_amount] issue_cost: Self::BigUint,
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

    #[view(repayPositionsIds)]
    fn get_repay_positions_ids(&self) -> MultiResultVec<BoxedBytes> {
        let mut result = MultiArgVec::new();
        for (key, _) in self.repay_position().iter() {
            result.push(key);
        }
        result
    }

    #[view(repayPosition)]
    fn view_repay_position(
        &self,
        position_id: BoxedBytes,
    ) -> SCResult<RepayPostion<Self::BigUint>> {
        Ok(self.repay_position().get(&position_id).unwrap())
    }

    #[view(debtPosition)]
    fn view_debt_position(&self, position_id: BoxedBytes) -> SCResult<DebtPosition<Self::BigUint>> {
        Ok(self.debt_positions().get(&position_id).unwrap())
    }

    #[view(getBorrowRate)]
    fn view_borrow_rate(&self) -> Self::BigUint {
        self.get_borrow_rate()
    }

    #[view(getDepositRate)]
    fn view_deposit_rate(&self) -> Self::BigUint {
        self.get_deposit_rate()
    }

    #[view(getDebtInterest)]
    fn view_debt_interest(&self, amount: Self::BigUint, timestamp: u64) -> Self::BigUint {
        self.get_debt_interest(amount, timestamp)
    }

    #[view(getPositionInterest)]
    fn get_debt_position_interest(&self, position_id: BoxedBytes) -> Self::BigUint {
        let debt_position = self.debt_positions().get(&position_id).unwrap_or_default();
        self.get_debt_interest(debt_position.size.clone(), debt_position.timestamp)
    }

    #[view(getCapitalUtilisation)]
    fn view_capital_utilisation(&self) -> Self::BigUint {
        self.get_capital_utilisation()
    }

    #[view(getReserve)]
    fn view_reserve(&self) -> Self::BigUint {
        self.reserves(&self.pool_asset().get()).get()
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

    #[endpoint(setHealthFactorThreshold)]
    fn endpoint_health_factor_threshold(&self, health_factor_threashdol: u32) {
        self.health_factor_threshold()
            .set(&health_factor_threashdol);
    }

    #[view(healthFactorThreshold)]
    fn view_health_factor_threshold(&self) -> u32 {
        self.health_factor_threshold().get()
    }

    #[view(getLendingPool)]
    fn view_lending_pool(&self) -> Address {
        self.lending_pool().get()
    }

    #[view(totalBorrow)]
    fn view_total_borrow(&self) -> Self::BigUint {
        self.total_borrow().get()
    }

    #[view(repayPositionAmount)]
    fn view_repay_position_amount(&self) -> Self::BigUint {
        self.repay_position_amount().get()
    }

    #[view(repayPositionIdentifier)]
    fn view_repay_position_id(&self) -> TokenIdentifier {
        self.repay_position_id().get()
    }

    #[view(repayPositionNonce)]
    fn view_repay_position_nonce(&self) -> u64 {
        self.repay_position_nonce().get()
    }
}
