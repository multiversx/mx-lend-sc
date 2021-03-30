use elrond_wasm::{
    api::BigUintApi,
    types::{BoxedBytes, TokenIdentifier},
};

elrond_wasm::derive_imports!();

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct IssueData {
    pub name: BoxedBytes,
    pub ticker: TokenIdentifier,
    pub existing_token: TokenIdentifier,
}

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct DebtPosition<BigUint: BigUintApi> {
    pub size: BigUint,
    pub health_factor: u32,
    pub is_liquidated: bool,
    pub collateral_amount: BigUint,
    pub collateral_identifier: TokenIdentifier,
}

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct InterestMetadata {
    pub timestamp: u64,
}

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct DebtMetadata<BigUint: BigUintApi> {
    pub timestamp: u64,
    pub collateral_amount: BigUint,
    pub collateral_identifier: TokenIdentifier,
}

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct ReserveData<BigUint: BigUintApi> {
    pub r_base: BigUint,         // base ratio
    pub r_slope1: BigUint,       // slope before optimal utilisation
    pub r_slope2: BigUint,       // slope after optimal utilisation
    pub u_optimal: BigUint,      // optimal capital utilisation
    pub reserve_factor: BigUint, // safety module percentage fee
}
