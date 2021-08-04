#![no_std]

elrond_wasm::imports!();

mod proxy;
pub use proxy::*;

const TICKER_SEPARATOR: u8 = b'-';

#[elrond_wasm_derive::module]
pub trait PriceOracleModule {
    #[only_owner]
    #[endpoint(setAggregatorAddress)]
    fn set_aggregator_address(&self, address: Address) -> SCResult<()> {
        self.fee_estimator_contract_address().set(&address);
        Ok(())
    }

    fn get_price_for_pair(
        &self,
        from: TokenIdentifier,
        to: TokenIdentifier,
    ) -> Option<Self::BigUint> {
        let aggregator_address = self.aggregator_address().get();
        if aggregator_address.is_zero() {
            return None;
        }

        let from_ticker = self.get_token_ticker(from);
        let to_ticker = self.get_token_ticker(to);

        let result: OptionalResult<AggregatorResultAsMultiResult<Self::BigUint>> = self
            .aggregator_proxy(aggregator_address)
            .latest_price_feed_optional(from_ticker, to_ticker)
            .execute_on_dest_context();

        result
            .into_option()
            .map(|multi_result| AggregatorResult::from(multi_result).price)
    }

    fn get_token_ticker(&self, token_id: TokenIdentifier) -> BoxedBytes {
        for (i, char) in token_id.as_esdt_identifier().iter().enumerate() {
            if *char == TICKER_SEPARATOR {
                return token_id.as_esdt_identifier()[..i].into();
            }
        }

        token_id.into_boxed_bytes()
    }

    #[proxy]
    fn aggregator_proxy(&self, address: Address) -> proxy::Proxy<Self::SendApi>;

    #[view(aggregatorAddress)]
    #[storage_mapper("aggregatorAddress")]
    fn aggregator_address(&self) -> SingleValueMapper<Self::Storage, Address>;
}
