elrond_wasm::derive_imports!();

use elrond_wasm::{api::BigUintApi, types::TokenIdentifier};

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct InterestMetadata {
    pub timestamp: u64,
}

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct DebtMetadata<BigUint: BigUintApi> {
    pub timestamp: u64,
    pub collateral_amount: BigUint,
    pub collateral_identifier: TokenIdentifier,
    pub colletareal_timestamp: u64,
}

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct RepayPostion<BigUint: BigUintApi> {
    pub identifier: TokenIdentifier,
    pub amount: BigUint,
    pub nonce: u64,
    pub collateral_identifier: TokenIdentifier,
    pub collateral_amount: BigUint,
    pub collateral_timestamp: u64,
}

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct LiquidateData<BigUint: BigUintApi> {
    pub collateral_token: TokenIdentifier,
    pub amount: BigUint,
}
