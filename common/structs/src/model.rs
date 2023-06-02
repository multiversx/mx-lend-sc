#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub const BP: u64 = 1_000_000_000_000_000_000;
pub const MAX_THRESHOLD: u64 = BP / 2;
pub const MAX_THRESHOLD_ERROR_MSG: &[u8] =
    b"Cannot liquidate more than 50% of Liquidatee's position!";
pub const SECONDS_PER_YEAR: u64 = 31_536_000;

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct PoolParams<M: ManagedTypeApi> {
    pub r_base: BigUint<M>,
    pub r_slope1: BigUint<M>,
    pub r_slope2: BigUint<M>,
    pub u_optimal: BigUint<M>,
    pub reserve_factor: BigUint<M>,
}

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct IssueData<M: ManagedTypeApi> {
    pub name: ManagedBuffer<M>,
    pub ticker: ManagedBuffer<M>,
    pub is_empty_ticker: bool,
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, Clone)]
pub struct TokenAmountPair<M: ManagedTypeApi> {
    pub token_id: TokenIdentifier<M>,
    pub nonce: u64,
    pub amount: BigUint<M>,
}

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi, Clone)]
pub struct DepositPosition<M: ManagedTypeApi> {
    pub token_id: TokenIdentifier<M>,
    pub amount: BigUint<M>,
    pub owner_nonce: u64,
    pub round: u64,
    pub initial_supply_index: BigUint<M>,
}

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi, Clone)]
pub struct BorrowPosition<M: ManagedTypeApi> {
    pub token_id: TokenIdentifier<M>,
    pub amount: BigUint<M>,
    pub owner_nonce: u64,
    pub round: u64,
    pub initial_borrow_index: BigUint<M>,
}

impl<M: ManagedTypeApi> TokenAmountPair<M> {
    pub fn new(token_id: TokenIdentifier<M>, nonce: u64, amount: BigUint<M>) -> Self {
        TokenAmountPair {
            token_id,
            nonce,
            amount,
        }
    }
}

impl<M: ManagedTypeApi> DepositPosition<M> {
    pub fn new(
        token_id: TokenIdentifier<M>,
        amount: BigUint<M>,
        owner_nonce: u64,
        round: u64,
        initial_supply_index: BigUint<M>,
    ) -> Self {
        DepositPosition {
            token_id,
            amount,
            owner_nonce,
            round,
            initial_supply_index,
        }
    }
}

impl<M: ManagedTypeApi> BorrowPosition<M> {
    pub fn new(
        token_id: TokenIdentifier<M>,
        amount: BigUint<M>,
        owner_nonce: u64,
        round: u64,
        initial_borrow_index: BigUint<M>,
    ) -> Self {
        BorrowPosition {
            token_id,
            amount,
            owner_nonce,
            round,
            initial_borrow_index,
        }
    }
}
