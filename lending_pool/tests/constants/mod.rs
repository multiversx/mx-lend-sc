// price aggregator constants

pub const PRICE_AGGREGATOR_WASM_PATH: &'static str =
    "../aggregator_mock/output/aggregator-mock.wasm";
pub const DOLLAR_TICKER: &[u8] = b"USD";
pub const EGLD_TICKER: &[u8] = b"EGLD";
pub const EGLD_PRICE_IN_DOLLARS: u64 = 20_000; // $200
pub const PRICE_DECIMALS: usize = 2;

// lending pool constants

pub const LENDING_POOL_WASM_PATH: &'static str = "output/lending-pool.wasm";

// liquidity pool constants

pub const LIQUIDITY_POOL_WASM_PATH: &'static str = "../liquidity_pool/output/liquidity-pool.wasm";
