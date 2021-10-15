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
    ) -> SCResult<AsyncCall> {
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

        Ok(self
            .send()
            .esdt_system_sc_proxy()
            .issue_semi_fungible(
                issue_cost,
                &issue_data.name,
                &ManagedBuffer::new_from_bytes(
                    self.type_manager(),
                    issue_data.ticker.to_esdt_identifier().as_slice(),
                ),
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
            .with_callback(self.callbacks().issue_callback(&token_prefix)))
    }

    #[only_owner]
    #[endpoint(setLendTokensRoles)]
    fn set_lend_token_roles(&self, roles: Vec<EsdtLocalRole>) -> SCResult<AsyncCall> {
        require!(!self.lend_token().is_empty(), "token not issued yet");
        Ok(self.set_roles(self.lend_token().get(), roles))
    }

    #[only_owner]
    #[endpoint(setBorrowTokenRoles)]
    fn set_borrow_token_roles(&self, roles: Vec<EsdtLocalRole>) -> SCResult<AsyncCall> {
        require!(!self.borrow_token().is_empty(), "token not issued yet");
        Ok(self.set_roles(self.borrow_token().get(), roles))
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
            .with_callback(self.callbacks().set_roles_callback())
    }

    fn mint_position_tokens(&self, token_id: &TokenIdentifier, amount: &BigUint) -> u64 {
        let mut uris = ManagedVec::new(self.type_manager());
        uris.push(self.types().managed_buffer_new());

        self.send().esdt_nft_create(
            token_id,
            amount,
            &self.types().managed_buffer_new(),
            &self.types().big_uint_zero(),
            &self.types().managed_buffer_new(),
            &(),
            &uris,
        )
    }

    #[callback]
    fn issue_callback(
        &self,
        prefix: &ManagedBuffer,
        #[call_result] result: ManagedAsyncCallResult<TokenIdentifier>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(ticker) => {
                if prefix.to_boxed_bytes() == BoxedBytes::from(LEND_TOKEN_PREFIX) {
                    self.lend_token().set(&ticker);
                } else {
                    self.borrow_token().set(&ticker);
                }
                self.last_error().clear();
                self.send_callback_result(ticker, b"setTickerAfterIssue");
            }
            ManagedAsyncCallResult::Err(message) => {
                let caller = self.blockchain().get_owner_address();
                let (returned_tokens, token_id) = self.call_value().payment_token_pair();
                if token_id.is_egld() && returned_tokens > 0 {
                    self.send()
                        .direct(&caller, &token_id, 0, &returned_tokens, &[]);
                }
                self.last_error().set(&message.err_msg);
            }
        }
    }

    #[callback]
    fn set_roles_callback(&self, #[call_result] result: ManagedAsyncCallResult<()>) {
        match result {
            ManagedAsyncCallResult::Ok(()) => {
                self.last_error().clear();
            }
            ManagedAsyncCallResult::Err(message) => {
                self.last_error().set(&message.err_msg);
            }
        }
    }

    fn send_callback_result(&self, token: TokenIdentifier, endpoint: &[u8]) {
        let owner = self.blockchain().get_owner_address();

        let mut args = ManagedArgBuffer::new_empty(self.type_manager());
        args.push_arg(token);

        let expected_gas = self.blockchain().get_gas_left() / 2;

        self.raw_vm_api().execute_on_dest_context_raw(
            expected_gas,
            &owner,
            &self.types().big_uint_zero(),
            &ManagedBuffer::new_from_bytes(self.type_manager(), endpoint),
            &args,
        );
    }
}
