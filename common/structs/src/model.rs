#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

pub const BP: u64 = 1_000_000_000;
pub const SECONDS_PER_YEAR: u64 = 31_536_000;
pub const LEND_TOKEN_PREFIX: u8 = b'L';
pub const BORROW_TOKEN_PREFIX: u8 = b'B';

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

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, PartialEq, Clone)]
pub struct TokenAmountPair<M: ManagedTypeApi> {
    pub token_id: TokenIdentifier<M>,
    pub nonce: u64,
    pub amount: BigUint<M>,
}

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub struct DepositPosition<M: ManagedTypeApi> {
    pub round: u64,
    pub amount: BigUint<M>,
    pub initial_supply_index: BigUint<M>,
}

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi, PartialEq, Clone)]
pub struct BorrowPosition<M: ManagedTypeApi> {
    pub round: u64,
    pub lend_tokens: TokenAmountPair<M>,
    pub borrowed_amount: BigUint<M>,
    pub collateral_token_id: TokenIdentifier<M>,
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
    pub fn new(round: u64, amount: BigUint<M>, initial_supply_index: BigUint<M>) -> Self {
        DepositPosition {
            round,
            amount,
            initial_supply_index,
        }
    }
}

impl<M: ManagedTypeApi> BorrowPosition<M> {
    pub fn new(
        round: u64,
        lend_tokens: TokenAmountPair<M>,
        borrowed_amount: BigUint<M>,
        collateral_token_id: TokenIdentifier<M>,
        initial_borrow_index: BigUint<M>,
    ) -> Self {
        BorrowPosition {
            round,
            lend_tokens,
            borrowed_amount,
            collateral_token_id,
            initial_borrow_index,
        }
    }
}
