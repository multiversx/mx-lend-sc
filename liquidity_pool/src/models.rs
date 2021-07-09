use elrond_wasm::{
    api::BigUintApi,
    types::{BoxedBytes, TokenIdentifier},
};

elrond_wasm::derive_imports!();

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct InterestMetadata {
    pub timestamp: u64,
}

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct ReserveData<BigUint: BigUintApi> {
    pub r_base: BigUint,         // base ratio
    pub r_slope1: BigUint,       // slope before optimal utilisation
    pub r_slope2: BigUint,       // slope after optimal utilisation
    pub u_optimal: BigUint,      // optimal capital utilisation
    pub reserve_factor: BigUint, // safety module percentage fee
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

#[derive(TopEncode, TopDecode, TypeAbi, PartialEq, Clone)]
pub struct RepayPostion<BigUint: BigUintApi> {
    pub identifier: TokenIdentifier,
    pub amount: BigUint,
    pub nonce: u64,
    pub borrow_timestamp: u64,
    pub collateral_identifier: TokenIdentifier,
    pub collateral_amount: BigUint,
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

impl<BigUint: BigUintApi> Default for RepayPostion<BigUint> {
    fn default() -> Self {
        RepayPostion {
            identifier: TokenIdentifier::egld(),
            amount: BigUint::zero(),
            nonce: 0u64,
            borrow_timestamp: 0u64,
            collateral_identifier: TokenIdentifier::egld(),
            collateral_amount: BigUint::zero(),
            collateral_timestamp: 0u64,
        }
    }
}
