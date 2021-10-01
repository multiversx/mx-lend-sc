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

const DEFAULT_PRICE: u64 = 1_000u64;
const DEFAULT_PRICE_DECIMALS: u8 = 2u8;

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
            from.clone(),
            to.clone(),
            self.get_price_or_default(&from, &to),
            DEFAULT_PRICE_DECIMALS,
        )))
    }

    fn get_price_or_default(&self, from: &BoxedBytes, to: &BoxedBytes) -> Self::BigUint {
        if self.latest_price_feed(from, to).is_empty() {
            DEFAULT_PRICE.into()
        } else {
            self.latest_price_feed(from, to).get()
        }
    }

    #[endpoint(setLatestPriceFeed)]
    fn set_latest_price_feed(&self, from: &BoxedBytes, to: &BoxedBytes, price: Self::BigUint) {
        self.latest_price_feed(from, to).set(&price)
    }

    #[storage_mapper("latest_price_feed")]
    fn latest_price_feed(
        &self,
        from: &BoxedBytes,
        to: &BoxedBytes,
    ) -> SingleValueMapper<Self::Storage, Self::BigUint>;
}
