// price aggregator constants

pub const PRICE_AGGREGATOR_WASM_PATH: &'static str =
    "../aggregator_mock/output/aggregator-mock.wasm";
pub const DOLLAR_TICKER: &[u8] = b"USD";
pub const USDC_TICKER: &[u8] = b"USDC";
pub const USDC_TOKEN_ID: &[u8] = b"USDC-123456";
pub const EGLD_TICKER: &[u8] = b"WEGLD";
pub const EGLD_TOKEN_ID: &[u8] = b"WEGLD-123456";
pub const LEND_EGLD: &[u8] = b"LWEGLD-abcdef";
pub const BORROW_EGLD: &[u8] = b"BWEGLD-abcdef";
pub const EGLD_PRICE_IN_DOLLARS: u64 = 20_000; // $200
pub const EGLD_PRICE_DROPPED_IN_DOLLARS: u64 = 14_000; // $140
pub const USDC_PRICE_IN_DOLLARS: u64 = 100; // $1
pub const PRICE_DECIMALS: usize = 2;
pub const R_BASE: u64 = 0;
pub const R_SLOPE1: u64 = 40_000_000;
pub const R_SLOPE2: u64 = 1_000_000_000;
pub const U_OPTIMAL: u64 = 800_000_000;
pub const RESERVE_FACTOR: u64 = 100_000_000;
pub const LIQ_THRESOLD: u64 = 700_000_000;
pub const LEND_USDC_TOKEN_ID: &[u8] = b"LUSDC-123456";
pub const BORROW_USDC_TOKEN_ID: &[u8] = b"BUSDC-123456";

// lending pool constants

pub const LENDING_POOL_WASM_PATH: &'static str = "output/lending-pool.wasm";

// liquidity pool constants

pub const LIQUIDITY_POOL_WASM_PATH: &'static str = "../liquidity_pool/output/liquidity-pool.wasm";
