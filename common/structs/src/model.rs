#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

pub const SECONDS_PER_YEAR: u64 = 31536000;
pub const BP: u64 = 1000000000;
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
pub struct TokenUUID {
    pub token_id: TokenIdentifier,
    pub nonce: u64,
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, PartialEq, Clone)]
pub struct TokenAmountPair<BigUint: BigUintApi> {
    pub token_uuid: TokenUUID,
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
}

impl TokenUUID {
    pub fn new(token_id: TokenIdentifier, nonce: u64) -> Self {
        TokenUUID { token_id, nonce }
    }
}

impl<BigUint: BigUintApi> TokenAmountPair<BigUint> {
    pub fn new(token_uuid: TokenUUID, amount: BigUint) -> Self {
        TokenAmountPair { token_uuid, amount }
    }

    pub fn get_token_id(&self) -> TokenIdentifier {
        self.token_uuid.token_id.clone()
    }

    pub fn get_token_nonce(&self) -> u64 {
        self.token_uuid.nonce
    }

    pub fn get_amount(&self) -> BigUint {
        self.amount.clone()
    }

    pub fn get_token_id_as_ref(&self) -> &TokenIdentifier {
        &self.token_uuid.token_id
    }

    pub fn get_amount_as_ref(&self) -> &BigUint {
        &self.amount
    }
}

impl<BigUint: BigUintApi> DepositPosition<BigUint> {
    pub fn new(timestamp: u64, amount: BigUint) -> Self {
        DepositPosition { timestamp, amount }
    }
}

impl<BigUint: BigUintApi> BorrowPosition<BigUint> {
    pub fn new(timestamp: u64, lend_tokens: TokenAmountPair<BigUint>) -> Self {
        BorrowPosition {
            timestamp,
            lend_tokens,
        }
    }
}
