#![no_std]

multiversx_sc::imports!();

pub const GWEI_STRING: &[u8] = b"GWEI";

pub type AggregatorResultAsMultiValue<M> =
    MultiValue5<u32, ManagedBuffer<M>, ManagedBuffer<M>, BigUint<M>, u8>;

mod price_aggregator_proxy_mod {
    multiversx_sc::imports!();

    #[multiversx_sc::proxy]
    pub trait PriceAggregator {
        #[view(latestPriceFeedOptional)]
        fn latest_price_feed_optional(
            &self,
            from: ManagedBuffer,
            to: ManagedBuffer,
        ) -> OptionalValue<super::AggregatorResultAsMultiValue<Self::Api>>;
    }
}

pub struct AggregatorResult<M: ManagedTypeApi> {
    pub round_id: u32,
    pub from_token_name: ManagedBuffer<M>,
    pub to_token_name: ManagedBuffer<M>,
    pub price: BigUint<M>,
    pub decimals: u8,
}

impl<M: ManagedTypeApi> From<AggregatorResultAsMultiValue<M>> for AggregatorResult<M> {
    fn from(multi_result: AggregatorResultAsMultiValue<M>) -> Self {
        let (round_id, from_token_name, to_token_name, price, decimals) = multi_result.into_tuple();

        AggregatorResult {
            round_id,
            from_token_name,
            to_token_name,
            price,
            decimals,
        }
    }
}

#[multiversx_sc::module]
pub trait PriceAggregatorModule {
    #[only_owner]
    #[endpoint(setPriceAggregatorAddress)]
    fn set_price_aggregator_address(&self, address: ManagedAddress) {
        require!(
            self.blockchain().is_smart_contract(&address),
            "Invalid price aggregator address"
        );

        self.price_aggregator_address().set(&address);
    }

    fn get_price_for_pair(
        &self,
        from_ticker: ManagedBuffer,
        to_ticker: ManagedBuffer,
    ) -> Option<BigUint> {
        self.get_full_result_for_pair(from_ticker, to_ticker)
            .map(|aggregator_result| aggregator_result.price)
    }

    fn get_full_result_for_pair(
        &self,
        from_ticker: ManagedBuffer,
        to_ticker: ManagedBuffer,
    ) -> Option<AggregatorResult<Self::Api>> {
        if self.price_aggregator_address().is_empty() {
            return None;
        }
        let price_aggregator_address = self.price_aggregator_address().get();

        let result: OptionalValue<AggregatorResultAsMultiValue<Self::Api>> = self
            .aggregator_proxy(price_aggregator_address)
            .latest_price_feed_optional(from_ticker, to_ticker)
            .execute_on_dest_context();

        result.into_option().map(AggregatorResult::from)
    }

    #[proxy]
    fn aggregator_proxy(
        &self,
        address: ManagedAddress,
    ) -> price_aggregator_proxy_mod::Proxy<Self::Api>;

    #[view(getAggregatorAddress)]
    #[storage_mapper("priceAggregatorAddress")]
    fn price_aggregator_address(&self) -> SingleValueMapper<ManagedAddress>;
}
