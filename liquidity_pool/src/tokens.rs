elrond_wasm::imports!();
use elrond_wasm::*;
use elrond_wasm::types::{Address, TokenIdentifier, SCResult, BoxedBytes, AsyncCall, VarArgs, EsdtLocalRole, H256, AsyncCallResult, ArgBuffer};
use crate::{InterestMetadata, DebtMetadata, LEND_TOKEN_PREFIX};
use elrond_wasm::esdt::SemiFungibleTokenProperties;
use elrond_wasm::esdt::ESDTSystemSmartContractProxy;



#[elrond_wasm_derive::module]
pub trait TokensModule: crate::storage::StorageModule + crate::utils::UtilsModule + crate::library::LibraryModule{

    fn mint_l_tokens(
        &self,
        initial_caller: Address,
        lend_token: TokenIdentifier,
        amount: Self::BigUint,
        interest_timestamp: u64,
    ) -> SCResult<()> {
        require!(
            self.blockchain().get_caller() == self.lending_pool().get(),
            "can only by called by lending pool"
        );

        require!(
            lend_token == self.lend_token().get(),
            "asset is not supported by this pool"
        );
        require!(amount > 0, "amount must be greater then 0");
        require!(!initial_caller.is_zero(), "invalid address");

        let interest_metadata = InterestMetadata {
            timestamp: interest_timestamp,
        };

        self.mint_interest(amount.clone(), interest_metadata);

        let nonce = self
            .blockchain().get_current_esdt_nft_nonce(&self.blockchain().get_sc_address(), &lend_token);

        self.send().direct(
            &initial_caller,
            &lend_token,
            nonce,
            &amount,
            &[],
        );

        Ok(())
    }


    fn burn_l_tokens(
        &self,
        initial_caller: Address,
        lend_token: TokenIdentifier,
        amount: Self::BigUint,
    ) -> SCResult<()> {
        require!(
            self.blockchain().get_caller() == self.lending_pool().get(),
            "can only by called by lending pool"
        );

        require!(
            lend_token == self.lend_token().get(),
            "asset is not supported by this pool"
        );
        require!(amount > 0, "amount must be greater then 0");
        require!(!initial_caller.is_zero(), "invalid address");

        let nft_nonce = self.blockchain().get_current_esdt_nft_nonce(&self.blockchain().get_sc_address(), &lend_token);

        self.burn(amount.clone(), nft_nonce, lend_token);


        Ok(())
    }


    fn issue(
        &self,
        plain_ticker: BoxedBytes,
        token_ticker: TokenIdentifier,
        token_prefix: BoxedBytes,
        issue_cost: Self::BigUint,
    ) -> SCResult<AsyncCall<Self::SendApi>> {
        require!(
            token_ticker == self.pool_asset().get(),
            "wrong ESDT asset identifier"
        );

        let issue_data = self.prepare_issue_data(token_prefix.clone(), plain_ticker);
        require!(
            issue_data.name != BoxedBytes::zeros(0),
            "invalid input. could not prepare issue data"
        );
        require!(
            issue_data.is_empty_ticker,
            "token already issued for this identifier"
        );

        Ok(ESDTSystemSmartContractProxy::new_proxy_obj(self.send())
            .issue_semi_fungible(
                issue_cost,
                &issue_data.name,
                &BoxedBytes::from(issue_data.ticker.as_esdt_identifier()),
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



    fn set_lend_token_roles(
        &self,
        roles: VarArgs<EsdtLocalRole>,
    ) -> SCResult<AsyncCall<Self::SendApi>> {
        only_owner!(self, "only owner can set roles");
        require!(!self.lend_token().is_empty(), "token not issued yet");
        Ok(self.set_roles(self.lend_token().get(), roles))
    }

    
    fn set_borrow_token_roles(
        &self,
        roles: VarArgs<EsdtLocalRole>,
    ) -> SCResult<AsyncCall<Self::SendApi>> {
        only_owner!(self, "only owner can set roles");
        require!(!self.borrow_token().is_empty(), "token not issued yet");
        Ok(self.set_roles(self.borrow_token().get(), roles))
    }


    fn set_roles(
        &self,
        token: TokenIdentifier,
        roles: VarArgs<EsdtLocalRole>,
    ) -> AsyncCall<Self::SendApi> {
        ESDTSystemSmartContractProxy::new_proxy_obj(self.send())
            .set_special_roles(
                &self.blockchain().get_sc_address(),
                &token,
                roles.as_slice(),
            )
            .async_call()
            .with_callback(self.callbacks().set_roles_callback())
    }

    fn mint_interest(&self, amount: Self::BigUint, metadata: InterestMetadata) {
        self.send().esdt_nft_create::<InterestMetadata>(
            &self.lend_token().get(),
            &amount,
            &BoxedBytes::empty(),
            &Self::BigUint::zero(),
            &BoxedBytes::empty(),
            &metadata,
            &[BoxedBytes::empty()],
        );
    }

    fn mint_debt(&self, amount: Self::BigUint, metadata: DebtMetadata<Self::BigUint>, position_id: H256) {
        self.send().esdt_nft_create::<DebtMetadata<Self::BigUint>>(
            &self.borrow_token().get(),
            &amount,
            &BoxedBytes::empty(),
            &Self::BigUint::zero(),
            &position_id.into_boxed_bytes(),
            &metadata,
            &[BoxedBytes::empty()],
        );
    }

    fn burn(&self, amount: Self::BigUint, nonce: u64, ticker: TokenIdentifier) {
        self.send().esdt_local_burn(
            &ticker,
            nonce,
            &amount,
        );
    }

    #[callback]
    fn issue_callback(
        &self,
        prefix: &BoxedBytes,
        #[call_result] result: AsyncCallResult<TokenIdentifier>,
    ) {
        match result {
            AsyncCallResult::Ok(ticker) => {
                if prefix == &BoxedBytes::from(LEND_TOKEN_PREFIX) {
                    self.lend_token().set(&ticker);
                } else {
                    self.borrow_token().set(&ticker);
                }
                self.last_error().clear();
                self.send_callback_result(ticker, b"setTickerAfterIssue");
            }
            AsyncCallResult::Err(message) => {
                let caller = self.blockchain().get_owner_address();
                let (returned_tokens, token_id) = self.call_value().payment_token_pair();
                if token_id.is_egld() && returned_tokens > 0 {
                    self.send().direct_egld(&caller, &returned_tokens, &[]);
                }
                self.last_error().set(&message.err_msg);
            }
        }
    }

    #[callback]
    fn set_roles_callback(&self, #[call_result] result: AsyncCallResult<()>) {
        match result {
            AsyncCallResult::Ok(()) => {
                self.last_error().clear();
            }
            AsyncCallResult::Err(message) => {
                self.last_error().set(&message.err_msg);
            }
        }
    }


    fn send_callback_result(&self, token: TokenIdentifier, endpoint: &[u8]) {
        let owner = self.blockchain().get_owner_address();

        let mut args = ArgBuffer::new();
        args.push_argument_bytes(token.as_esdt_identifier());

        let expected_gas = self.blockchain().get_gas_left()/2;

        self.send().execute_on_dest_context_raw(
            expected_gas,
            &owner,
            &Self::BigUint::zero(),
            endpoint,
            &args,
        );
    }




}