#![no_std]
#![allow(unused_attributes)]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use common_structs::{DepositMetadata, BP, ESDT_ISSUE_COST, SECONDS_PER_YEAR};

#[elrond_wasm::contract]
pub trait SafetyModule {
    #[init]
    fn init(&self, wegld_token: TokenIdentifier, depositors_apy: Self::BigUint) {
        self.wegld_token().set(&wegld_token);
        self.deposit_apy().set(&depositors_apy);
    }

    #[only_owner]
    #[endpoint(addPool)]
    fn add_pool(&self, token: TokenIdentifier, address: &Address) -> SCResult<()> {
        self.pools(token).set(address);

        Ok(())
    }

    #[only_owner]
    #[endpoint(removePool)]
    fn remove_pool(&self, token: TokenIdentifier) -> SCResult<()> {
        self.pools(token).clear();

        Ok(())
    }

    #[payable("*")]
    #[endpoint(fund)]
    fn fund(
        self,
        #[payment_token] token: TokenIdentifier,
        #[payment_amount] payment: Self::BigUint,
        #[var_args] caller: OptionalArg<Address>,
    ) -> SCResult<()> {
        require!(payment > 0, "amount must be greater than 0");
        require!(token == self.wegld_token().get(), "invalid token");

        let caller_address = caller
            .into_option()
            .unwrap_or_else(|| self.blockchain().get_caller());

        let deposit_metadata = DepositMetadata {
            timestamp: self.blockchain().get_block_timestamp(),
        };

        self.mint_deposit_nft(&deposit_metadata, payment.clone());

        let nft_token = self.nft_token().get();

        let nonce = self
            .blockchain()
            .get_current_esdt_nft_nonce(&self.blockchain().get_sc_address(), &nft_token);

        self.send()
            .direct(&caller_address, &nft_token, nonce, &payment, &[]);

        Ok(())
    }

    #[only_owner]
    #[payable("EGLD")]
    #[endpoint(nftIssue)]
    fn nft_issue(
        &self,
        #[payment_amount] issue_cost: Self::BigUint,
        token_display_name: BoxedBytes,
        token_ticker: BoxedBytes,
    ) -> SCResult<AsyncCall<Self::SendApi>> {
        require!(issue_cost == ESDT_ISSUE_COST, "wrong ESDT asset identifier");

        Ok(ESDTSystemSmartContractProxy::new_proxy_obj(self.send())
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
            .with_callback(self.callbacks().nft_issue_callback(&caller)))
    }

    #[callback]
    fn nft_issue_callback(
        &self,
        caller: &Address,
        #[call_result] result: AsyncCallResult<TokenIdentifier>,
    ) {
        match result {
            AsyncCallResult::Ok(token_identifier) => {
                self.nft_token().set(&token_identifier);
                self.last_error_message().clear();
            }
            AsyncCallResult::Err(message) => {
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
        #[payment_amount] payment: Self::BigUint,
    ) -> SCResult<()> {
        require!(payment > 0, "amount must be greater than 0");

        self.convert_to_wegld(token, payment);

        Ok(())
    }

    #[endpoint(takeFunds)]
    fn take_funds(&self, pool_token: TokenIdentifier, amount: Self::BigUint) -> SCResult<()> {
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

        Ok(())
    }

    #[payable("*")]
    #[endpoint(withdraw)]
    fn withdraw(&self, #[payment_amount] amount: Self::BigUint) -> SCResult<Self::BigUint> {
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

        let nft_metadata = nft_info.decode_attributes::<DepositMetadata>()?;
        let time_in_pool = self.blockchain().get_block_timestamp() - nft_metadata.timestamp;

        require!(time_in_pool > 0, "invalid timestamp");

        let withdraw_amount =
            self.calculate_amount_for_withdrawal(amount.clone(), Self::BigUint::from(time_in_pool));

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

        Ok(withdraw_amount)
    }

    #[only_owner]
    #[endpoint(setLocalRolesNftToken)]
    fn set_local_roles_nft_token(
        &self,
        #[var_args] roles: VarArgs<EsdtLocalRole>,
    ) -> SCResult<AsyncCall<Self::SendApi>> {
        require!(!self.nft_token().is_empty(), "No nft token issued");

        let token = self.nft_token().get();
        Ok(self.set_local_roles(token, roles))
    }

    #[callback]
    fn change_roles_callback(&self, #[call_result] result: AsyncCallResult<()>) {
        match result {
            AsyncCallResult::Ok(()) => {
                self.last_error_message().clear();
            }
            AsyncCallResult::Err(message) => {
                self.last_error_message().set(&message.err_msg);
            }
        }
    }

    fn set_local_roles(
        &self,
        token: TokenIdentifier,
        #[var_args] roles: VarArgs<EsdtLocalRole>,
    ) -> AsyncCall<Self::SendApi> {
        ESDTSystemSmartContractProxy::new_proxy_obj(self.send())
            .set_special_roles(
                &self.blockchain().get_sc_address(),
                &token,
                roles.as_slice(),
            )
            .async_call()
            .with_callback(self.callbacks().change_roles_callback())
    }

    fn calculate_amount_for_withdrawal(
        self,
        deposit_amount: Self::BigUint,
        time: Self::BigUint,
    ) -> Self::BigUint {
        let percent = (time * self.deposit_apy().get()) / Self::BigUint::from(SECONDS_PER_YEAR);

        deposit_amount.clone() + ((percent * deposit_amount) / Self::BigUint::from(BP))
    }

    fn nft_burn(&self, token_identifier: TokenIdentifier, nonce: u64, amount: Self::BigUint) {
        self.send()
            .esdt_local_burn(&token_identifier, nonce, &amount);
    }

    fn mint_deposit_nft(self, deposit_metadata: &DepositMetadata, amount: Self::BigUint) {
        self.send().esdt_nft_create::<DepositMetadata>(
            &self.nft_token().get(),
            &amount,
            &BoxedBytes::empty(),
            &Self::BigUint::zero(),
            &BoxedBytes::empty(),
            deposit_metadata,
            &[BoxedBytes::empty()],
        );
    }

    fn convert_wegld(&self, _pool_token: TokenIdentifier, _amount: Self::BigUint) {
        //TODO:  integration with dex
    }

    fn convert_to_wegld(&self, _pool_token: TokenIdentifier, _amount: Self::BigUint) {
        //TODO:  integration with dex
    }

    #[view]
    #[storage_mapper("pools")]
    fn pools(&self, token: TokenIdentifier) -> SingleValueMapper<Self::Storage, Address>;

    #[view]
    #[storage_mapper("wegld_token")]
    fn wegld_token(&self) -> SingleValueMapper<Self::Storage, TokenIdentifier>;

    #[view]
    #[storage_mapper("deposit_apy")]
    fn deposit_apy(&self) -> SingleValueMapper<Self::Storage, Self::BigUint>;

    #[view(nftToken)]
    #[storage_mapper("nftToken")]
    fn nft_token(&self) -> SingleValueMapper<Self::Storage, TokenIdentifier>;

    #[view(lastErrorMessage)]
    #[storage_mapper("lastErrorMessage")]
    fn last_error_message(&self) -> SingleValueMapper<Self::Storage, BoxedBytes>;
}
