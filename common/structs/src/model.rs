#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

pub const BP: u64 = 1000000000;
pub const SECONDS_PER_YEAR: u64 = 31536000;
pub const ESDT_ISSUE_COST: u64 = 5000000000000000000;
pub const LEND_TOKEN_PREFIX: &[u8] = b"L";
pub const BORROW_TOKEN_PREFIX: &[u8] = b"B";

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct PoolParams<BigUint: BigUintApi> {
    pub r_base: BigUint,
    pub r_slope1: BigUint,
    pub r_slope2: BigUint,
    pub u_optimal: BigUint,
    pub reserve_factor: BigUint,
}

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct IssueData {
    pub name: BoxedBytes,
    pub ticker: TokenIdentifier,
    pub is_empty_ticker: bool,
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, PartialEq, Clone)]
pub struct TokenAmountPair<BigUint: BigUintApi> {
    pub token_id: TokenIdentifier,
    pub nonce: u64,
    pub amount: BigUint,
}

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub struct DepositPosition<BigUint: BigUintApi> {
    pub timestamp: u64,
    pub amount: BigUint,
}

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi, PartialEq, Clone)]
pub struct BorrowPosition<BigUint: BigUintApi> {
    pub timestamp: u64,
    pub lend_tokens: TokenAmountPair<BigUint>,
    pub borrowed_amount: BigUint,
    pub collateral_token_id: TokenIdentifier,
}

impl<BigUint: BigUintApi> TokenAmountPair<BigUint> {
    pub fn new(token_id: TokenIdentifier, nonce: u64, amount: BigUint) -> Self {
        TokenAmountPair {
            token_id,
            nonce,
            amount,
        }
    }
}

impl<BigUint: BigUintApi> DepositPosition<BigUint> {
    pub fn new(timestamp: u64, amount: BigUint) -> Self {
        DepositPosition { timestamp, amount }
    }
}

impl<BigUint: BigUintApi> BorrowPosition<BigUint> {
    pub fn new(
        timestamp: u64,
        lend_tokens: TokenAmountPair<BigUint>,
        borrowed_amount: BigUint,
        collateral_token_id: TokenIdentifier,
    ) -> Self {
        BorrowPosition {
            timestamp,
            lend_tokens,
            borrowed_amount,
            collateral_token_id,
        }
    }
}
