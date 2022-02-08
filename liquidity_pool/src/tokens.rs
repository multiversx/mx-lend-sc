elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use common_structs::LEND_TOKEN_PREFIX;

use super::math;
use super::storage;
use super::utils;

#[elrond_wasm::module]
pub trait TokensModule:
    storage::StorageModule
    + utils::UtilsModule
    + math::MathModule
    + price_aggregator_proxy::PriceAggregatorModule
{
    #[only_owner]
    #[payable("EGLD")]
    #[endpoint]
    fn issue(
        &self,
        plain_ticker: ManagedBuffer,
        token_ticker: TokenIdentifier,
        token_prefix: ManagedBuffer,
        #[payment_amount] issue_cost: BigUint,
    ) -> AsyncCall {
        require!(
            token_ticker == self.pool_asset().get(),
            "wrong ESDT asset identifier"
        );

        let issue_data = self.prepare_issue_data(token_prefix.clone(), plain_ticker);
        require!(
            !issue_data.name.is_empty(),
            "invalid input. could not prepare issue data"
        );
        require!(
            issue_data.is_empty_ticker,
            "token already issued for this identifier"
        );

        self.send()
            .esdt_system_sc_proxy()
            .issue_semi_fungible(
                issue_cost,
                &issue_data.name,
                &issue_data.ticker,
                SemiFungibleTokenProperties {
                    can_freeze: true,
                    can_wipe: true,
                    can_pause: true,
                    can_change_owner: true,
                    can_upgrade: true,
                    can_add_special_roles: true,
                },
            )
            .async_call()
            .with_callback(self.callbacks().issue_callback(&token_prefix))
    }

    #[only_owner]
    #[endpoint(setLendTokensRoles)]
    fn set_lend_token_roles(&self, roles: Vec<EsdtLocalRole>) -> AsyncCall {
        require!(!self.lend_token().is_empty(), "token not issued yet");

        self.set_roles(self.lend_token().get(), roles)
    }

    #[only_owner]
    #[endpoint(setBorrowTokenRoles)]
    fn set_borrow_token_roles(&self, roles: Vec<EsdtLocalRole>) -> AsyncCall {
        require!(!self.borrow_token().is_empty(), "token not issued yet");

        self.set_roles(self.borrow_token().get(), roles)
    }

    fn set_roles(&self, token: TokenIdentifier, roles: Vec<EsdtLocalRole>) -> AsyncCall {
        self.send()
            .esdt_system_sc_proxy()
            .set_special_roles(
                &self.blockchain().get_sc_address(),
                &token,
                (&roles[..]).into_iter().cloned(),
            )
            .async_call()
    }

    fn mint_position_tokens(&self, token_id: &TokenIdentifier, amount: &BigUint) -> u64 {
        let big_zero = BigUint::zero();
        let empty_buffer = ManagedBuffer::new();
        let empty_vec = ManagedVec::from_raw_handle(empty_buffer.get_raw_handle());

        self.send().esdt_nft_create(
            token_id,
            amount,
            &empty_buffer,
            &big_zero,
            &empty_buffer,
            &(),
            &empty_vec,
        )
    }

    #[callback]
    fn issue_callback(
        &self,
        prefix: &ManagedBuffer,
        #[call_result] result: ManagedAsyncCallResult<TokenIdentifier>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(token_id) => {
                if prefix.to_boxed_bytes() == BoxedBytes::from(LEND_TOKEN_PREFIX) {
                    self.lend_token().set(&token_id);
                } else {
                    self.borrow_token().set(&token_id);
                }

                self.send_callback_result(token_id);
            }
            ManagedAsyncCallResult::Err(_) => {
                let caller = self.blockchain().get_owner_address();
                let (returned_tokens, token_id) = self.call_value().payment_token_pair();
                if token_id.is_egld() && returned_tokens > 0 {
                    self.send()
                        .direct(&caller, &token_id, 0, &returned_tokens, &[]);
                }
            }
        }
    }

    fn send_callback_result(&self, token: TokenIdentifier) {
        let owner = self.blockchain().get_owner_address();
        self.lending_pool_proxy(owner)
            .set_ticker_after_issue(token)
            .execute_on_dest_context();
    }

    #[proxy]
    fn lending_pool_proxy(
        &self,
        sc_address: ManagedAddress,
    ) -> lending_pool_proxy_mod::Proxy<Self::Api>;
}

// can't simply import, we would have a circular dependency
mod lending_pool_proxy_mod {
    elrond_wasm::imports!();

    #[elrond_wasm::proxy]
    pub trait LendingPool {
        #[endpoint(setTickerAfterIssue)]
        fn set_ticker_after_issue(&self, token_ticker: TokenIdentifier);
    }
}
