#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

pub const SECONDS_PER_YEAR: u64 = 31536000;
pub const BP: u64 = 1000000000;
pub const ESDT_ISSUE_COST: u64 = 5000000000000000000;
pub const LEND_TOKEN_PREFIX: &[u8] = b"L";
pub const BORROW_TOKEN_PREFIX: &[u8] = b"B";

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub struct DepositMetadata {
    pub timestamp: u64,
}

#[derive(TopEncode, TopDecode, TypeAbi, Default)]
pub struct InterestMetadata {
    pub timestamp: u64,
}

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

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi, PartialEq, Clone)]
pub struct DebtPosition<BigUint: BigUintApi> {
    pub size: BigUint,
    pub health_factor: u32,
    pub is_liquidated: bool,
    pub timestamp: u64,
    pub collateral_amount: BigUint,
    pub collateral_identifier: TokenIdentifier,
}

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct LiquidateData<BigUint: BigUintApi> {
    pub collateral_token: TokenIdentifier,
    pub amount: BigUint,
}

#[derive(TopEncode, TopDecode, TypeAbi, Clone)]
pub struct DebtMetadata<BigUint: BigUintApi> {
    pub timestamp: u64,
    pub collateral_amount: BigUint,
    pub collateral_identifier: TokenIdentifier,
    pub collateral_timestamp: u64,
}

impl<BigUint: BigUintApi> Default for DebtPosition<BigUint> {
    fn default() -> Self {
        DebtPosition {
            size: BigUint::zero(),
            health_factor: 0u32,
            is_liquidated: bool::default(),
            timestamp: 0u64,
            collateral_amount: BigUint::zero(),
            collateral_identifier: TokenIdentifier::egld(),
        }
    }
}

impl InterestMetadata {
    pub fn new(timestamp: u64) -> Self {
        InterestMetadata { timestamp }
    }
}
