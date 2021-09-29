#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

pub struct AggregatorResult<BigUint: BigUintApi> {
    pub round_id: u32,
    pub from_token_name: BoxedBytes,
    pub to_token_name: BoxedBytes,
    pub price: BigUint,
    pub decimals: u8,
}

pub type AggregatorResultAsMultiResult<BigUint> =
    MultiResult5<u32, BoxedBytes, BoxedBytes, BigUint, u8>;

#[elrond_wasm::contract]
pub trait PriceAggregatorMock {
    #[init]
    fn init(&self) {}

    #[view(latestPriceFeedOptional)]
    fn latest_price_feed_optional(
        &self,
        from: BoxedBytes,
        to: BoxedBytes,
    ) -> OptionalResult<AggregatorResultAsMultiResult<Self::BigUint>> {
        OptionalArg::Some(MultiResult5::from((
            1u32,
            from,
            to,
            Self::BigUint::from(1000u32),
            2u8,
        )))
    }
}
