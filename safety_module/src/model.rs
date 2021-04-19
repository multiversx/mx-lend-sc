elrond_wasm::derive_imports!();

pub const SECONDS_PER_YEAR: u64 = 31536000;
pub const BP: u64 = 1000000000;
pub const ESDT_ISSUE_COST: u64 = 5000000000000000000;


#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub struct DepositMetadata {
    pub timestamp: u64,
}