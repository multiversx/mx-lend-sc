#![no_std]
#![allow(unused_attributes)]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use common_structs::{DepositPosition, BP, ESDT_ISSUE_COST, SECONDS_PER_YEAR};

#[elrond_wasm::contract]
pub trait SafetyModule {
    #[init]
    fn init(&self, wegld_token: TokenIdentifier, depositors_apy: BigUint) {
        self.wegld_token().set(&wegld_token);
        self.deposit_apy().set(&depositors_apy);
    }

    #[only_owner]
    #[endpoint(addPool)]
    fn add_pool(&self, token: TokenIdentifier, address: &ManagedAddress) {
        self.pools(token).set(address);
    }

    #[only_owner]
    #[endpoint(removePool)]
    fn remove_pool(&self, token: TokenIdentifier) {
        self.pools(token).clear();
    }

    #[payable("*")]
    #[endpoint(fund)]
    fn fund(
        self,
        #[payment_token] token: TokenIdentifier,
        #[payment_amount] payment: BigUint,
        #[var_args] caller: OptionalArg<ManagedAddress>,
    ) {
        require!(payment > 0, "amount must be greater than 0");
        require!(token == self.wegld_token().get(), "invalid token");

        let caller_address = caller
            .into_option()
            .unwrap_or_else(|| self.blockchain().get_caller());

        let timestamp = self.blockchain().get_block_timestamp();
        let deposit_metadata = DepositPosition::new(timestamp, payment.clone());

        let nft_token = self.nft_token().get();
        let nft_nonce = self.mint_deposit_nft(&deposit_metadata, payment.clone());

        self.send()
            .direct(&caller_address, &nft_token, nft_nonce, &payment, &[]);
    }

    #[only_owner]
    #[payable("EGLD")]
    #[endpoint(nftIssue)]
    fn nft_issue(
        &self,
        #[payment_amount] issue_cost: BigUint,
        token_display_name: ManagedBuffer,
        token_ticker: ManagedBuffer,
    ) -> AsyncCall {
        require!(issue_cost == ESDT_ISSUE_COST, "wrong ESDT asset identifier");

        self.send()
            .esdt_system_sc_proxy()
            .issue_semi_fungible(
                issue_cost,
                &token_display_name,
                &token_ticker,
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
            .with_callback(
                self.callbacks()
                    .nft_issue_callback(self.blockchain().get_caller()),
            )
    }

    #[callback]
    fn nft_issue_callback(
        &self,
        caller: ManagedAddress,
        #[call_result] result: ManagedAsyncCallResult<TokenIdentifier>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(token_identifier) => {
                self.nft_token().set(&token_identifier);
                self.last_error_message().clear();
            }
            ManagedAsyncCallResult::Err(message) => {
                // return issue cost to the caller
                let (returned_tokens, token_identifier) = self.call_value().payment_token_pair();
                if token_identifier.is_egld() && returned_tokens > 0 {
                    self.send()
                        .direct(&caller, &token_identifier, 0, &returned_tokens, &[]);
                }

                self.last_error_message().set(&message.err_msg);
            }
        }
    }

    #[payable("*")]
    #[endpoint(fundFromPool)]
    fn fund_from_pool(
        &self,
        #[payment_token] token: TokenIdentifier,
        #[payment_amount] payment: BigUint,
    ) {
        require!(payment > 0, "amount must be greater than 0");

        self.convert_to_wegld(token, payment);
    }

    #[endpoint(takeFunds)]
    fn take_funds(&self, pool_token: TokenIdentifier, amount: BigUint) {
        require!(amount > 0, "amount must be greater than 0");

        let caller_address = self.blockchain().get_caller();

        require!(
            !self.pools(pool_token.clone()).is_empty(),
            "non-existent pool"
        );
        require!(
            caller_address == self.pools(pool_token.clone()).get(),
            "invalid caller address"
        );

        self.convert_wegld(pool_token.clone(), amount.clone());

        self.send()
            .direct(&caller_address, &pool_token, 0, &amount, &[]);
    }

    #[payable("*")]
    #[endpoint(withdraw)]
    fn withdraw(&self, #[payment_amount] amount: BigUint) -> BigUint {
        let caller_address = self.blockchain().get_caller();
        let token_id = self.call_value().token();
        let nft_nonce = self.call_value().esdt_token_nonce();

        require!(amount > 0, "amount must be greater than 0");
        require!(token_id == self.nft_token().get(), "invalid token");

        let nft_info = self.blockchain().get_esdt_token_data(
            &self.blockchain().get_sc_address(),
            &token_id,
            nft_nonce,
        );

        let nft_metadata = nft_info.decode_attributes_or_exit::<DepositPosition<Self::Api>>();
        let time_in_pool = self.blockchain().get_block_timestamp() - nft_metadata.timestamp;

        require!(time_in_pool > 0, "invalid timestamp");

        let withdraw_amount =
            self.calculate_amount_for_withdrawal(amount.clone(), BigUint::from(time_in_pool));

        let wegld_token_id = &self.wegld_token().get();
        let contract_balance = self.blockchain().get_esdt_balance(
            &self.blockchain().get_sc_address(),
            wegld_token_id,
            0,
        );

        require!(
            withdraw_amount <= contract_balance,
            "the amount withdrawn is too high"
        );

        self.nft_burn(token_id, nft_nonce, amount);

        self.send()
            .direct(&caller_address, wegld_token_id, 0, &withdraw_amount, &[]);

        withdraw_amount
    }

    #[only_owner]
    #[endpoint(setLocalRolesNftToken)]
    fn set_local_roles_nft_token(&self, #[var_args] roles: VarArgs<EsdtLocalRole>) -> AsyncCall {
        require!(!self.nft_token().is_empty(), "No nft token issued");

        let token = self.nft_token().get();
        self.set_local_roles(token, roles.into_vec())
    }

    #[callback]
    fn change_roles_callback(&self, #[call_result] result: ManagedAsyncCallResult<()>) {
        match result {
            ManagedAsyncCallResult::Ok(()) => {
                self.last_error_message().clear();
            }
            ManagedAsyncCallResult::Err(message) => {
                self.last_error_message().set(&message.err_msg);
            }
        }
    }

    fn set_local_roles(&self, token: TokenIdentifier, roles: Vec<EsdtLocalRole>) -> AsyncCall {
        self.send()
            .esdt_system_sc_proxy()
            .set_special_roles(
                &self.blockchain().get_sc_address(),
                &token,
                (&roles[..]).into_iter().cloned(),
            )
            .async_call()
            .with_callback(self.callbacks().change_roles_callback())
    }

    fn calculate_amount_for_withdrawal(self, deposit_amount: BigUint, time: BigUint) -> BigUint {
        let percent = (time * self.deposit_apy().get()) / BigUint::from(SECONDS_PER_YEAR);

        deposit_amount.clone() + ((percent * deposit_amount) / BigUint::from(BP))
    }

    fn nft_burn(&self, token_identifier: TokenIdentifier, nonce: u64, amount: BigUint) {
        self.send()
            .esdt_local_burn(&token_identifier, nonce, &amount);
    }

    fn mint_deposit_nft(
        self,
        deposit_metadata: &DepositPosition<Self::Api>,
        amount: BigUint,
    ) -> u64 {
        let big_zero = BigUint::zero();
        let empty_buffer = ManagedBuffer::new();
        let empty_vec = ManagedVec::from_raw_handle(empty_buffer.get_raw_handle());

        self.send().esdt_nft_create(
            &self.nft_token().get(),
            &amount,
            &empty_buffer,
            &big_zero,
            &empty_buffer,
            deposit_metadata,
            &empty_vec,
        )
    }

    fn convert_wegld(&self, _pool_token: TokenIdentifier, _amount: BigUint) {
        //TODO:  integration with dex
    }

    fn convert_to_wegld(&self, _pool_token: TokenIdentifier, _amount: BigUint) {
        //TODO:  integration with dex
    }

    #[view]
    #[storage_mapper("pools")]
    fn pools(&self, token: TokenIdentifier) -> SingleValueMapper<ManagedAddress>;

    #[view]
    #[storage_mapper("wegld_token")]
    fn wegld_token(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view]
    #[storage_mapper("deposit_apy")]
    fn deposit_apy(&self) -> SingleValueMapper<BigUint>;

    #[view(nftToken)]
    #[storage_mapper("nftToken")]
    fn nft_token(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(lastErrorMessage)]
    #[storage_mapper("lastErrorMessage")]
    fn last_error_message(&self) -> SingleValueMapper<ManagedBuffer>;
}
