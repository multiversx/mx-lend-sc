#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub struct AggregatorResult<M: ManagedTypeApi> {
    pub round_id: u32,
    pub from_token_name: ManagedBuffer<M>,
    pub to_token_name: ManagedBuffer<M>,
    pub price: BigUint<M>,
    pub decimals: u8,
}

pub type AggregatorResultAsMultiValue<M> =
    MultiValue5<u32, ManagedBuffer<M>, ManagedBuffer<M>, BigUint<M>, u8>;

const DEFAULT_PRICE: u64 = 1_000;
const DEFAULT_PRICE_DECIMALS: u8 = 2;

#[multiversx_sc::contract]
pub trait PriceAggregatorMock {
    #[init]
    fn init(&self) {}

    #[view(latestPriceFeedOptional)]
    fn latest_price_feed_optional(
        &self,
        from: ManagedBuffer,
        to: ManagedBuffer,
    ) -> OptionalValue<AggregatorResultAsMultiValue<Self::Api>> {
        OptionalValue::Some(MultiValue5::from((
            1u32,
            from.clone(),
            to.clone(),
            self.get_price_or_default(&from, &to),
            DEFAULT_PRICE_DECIMALS,
        )))
    }

    fn get_price_or_default(&self, from: &ManagedBuffer, to: &ManagedBuffer) -> BigUint {
        if self.latest_price_feed(from, to).is_empty() {
            BigUint::from(DEFAULT_PRICE)
        } else {
            self.latest_price_feed(from, to).get()
        }
    }

    #[endpoint(setLatestPriceFeed)]
    fn set_latest_price_feed(&self, from: ManagedBuffer, to: ManagedBuffer, price: BigUint) {
        self.latest_price_feed(&from, &to).set(&price)
    }

    #[storage_mapper("latest_price_feed")]
    fn latest_price_feed(
        &self,
        from: &ManagedBuffer,
        to: &ManagedBuffer,
    ) -> SingleValueMapper<BigUint>;
}
