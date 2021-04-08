#![no_std]
#![allow(unused_attributes)]

use elrond_wasm::{only_owner, require};

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

const SECONDS_PER_YEAR: u64 = 31536000;
const BP: u64 = 1000000000;
const ESDT_ISSUE_COST: u64 = 5000000000000000000;

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub struct DepositMetadata {
    pub timestamp: u64,
}

#[elrond_wasm_derive::contract(SafetyModuleImpl)]
pub trait SafetyModule {
    #[init]
    fn init(&self, wegld_token: TokenIdentifier, depositors_apy: BigUint) {
        self.wegld_token().set(&wegld_token);
        self.deposit_apy().set(&depositors_apy);
    }

    #[endpoint(addPool)]
    fn add_pool(&self, token: TokenIdentifier, address: &Address) -> SCResult<()> {
        only_owner!(self, "Only owner may call this function!");

        self.pools(token).set(address);

        Ok(())
    }

    #[endpoint(removePool)]
    fn remove_pool(&self, token: TokenIdentifier) -> SCResult<()> {
        only_owner!(self, "Only owner may call this function!");

        self.pools(token).clear();

        Ok(())
    }

    #[payable("*")]
    #[endpoint(fund)]
    fn fund(
        self,
        #[payment_token] token: TokenIdentifier,
        #[payment] payment: BigUint,
    ) -> SCResult<()> {
        require!(payment > 0, "amount must be greater than 0");
        require!(token == self.wegld_token().get(), "invalid token");

        let caller_address = self.get_caller();

        let deposit_metadata = DepositMetadata {
            timestamp: self.get_block_timestamp(),
        };

        self.mint_deposit_nft(deposit_metadata, payment.clone());

        let nft_token = self.nft_token().get();

        let nonce =
            self.get_current_esdt_nft_nonce(&self.get_sc_address(), nft_token.as_esdt_identifier());

        self.send().direct_esdt_nft_via_transfer_exec(
            &caller_address,
            &nft_token.as_esdt_identifier(),
            nonce,
            &payment,
            &[],
        );

        Ok(())
    }

    #[payable("EGLD")]
    #[endpoint(nftIssue)]
    fn nft_issue(
        &self,
        #[payment] issue_cost: BigUint,
        token_display_name: BoxedBytes,
        token_ticker: BoxedBytes,
    ) -> SCResult<AsyncCall<BigUint>> {
        let caller = self.get_caller();

        only_owner!(self, "only owner can issue new tokens");
        require!(
            issue_cost == BigUint::from(ESDT_ISSUE_COST),
            "wrong ESDT asset identifier"
        );

        Ok(ESDTSystemSmartContractProxy::new()
            .issue_non_fungible(
                issue_cost,
                &token_display_name,
                &token_ticker,
                NonFungibleTokenProperties {
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
                    self.send().direct_egld(caller, &returned_tokens, &[]);
                }

                self.last_error_message().set(&message.err_msg);
            }
        }
    }

    #[payable("*")]
    #[endpoint(fundFromPool)]
    fn fund_from_pool(
        &self,
        #[payment] payment: BigUint,
        #[payment_token] token: TokenIdentifier,
    ) -> SCResult<()> {
        require!(payment > 0, "amount must be greater than 0");

        self.convert_to_wegld(token, payment);

        Ok(())
    }

    #[endpoint(takeFunds)]
    fn take_funds(&self, pool_token: TokenIdentifier, amount: BigUint) -> SCResult<()> {
        require!(amount > 0, "amount must be greater than 0");

        let caller_address = self.get_caller();

        require!(
            !self.pools(pool_token.clone()).is_empty(),
            "non-existent pool"
        );
        require!(
            caller_address == self.pools(pool_token.clone()).get(),
            "invalid caller address"
        );

        self.convert_wegld(pool_token.clone(), amount.clone());

        self.send().direct(
            &caller_address,
            &pool_token,
            &amount,
            b"successful transfer",
        );

        Ok(())
    }

    #[payable("*")]
    #[endpoint(withdraw)]
    fn withdraw(&self, #[payment] amount: BigUint) -> SCResult<()> {
        let caller_address = self.get_caller();
        let nft_type = self.call_value().token();
        let nft_nonce = self.call_value().esdt_token_nonce();

        require!(
            self.call_value().esdt_token_type() == EsdtTokenType::NonFungible,
            "Only Non-Fungible tokens"
        );
        require!(amount > 0, "amount must be greater than 0");
        require!(nft_type == self.wegld_token().get(), "invalid token");

        let nft_info = self.get_esdt_token_data(
            &self.get_sc_address(),
            nft_type.as_esdt_identifier(),
            nft_nonce,
        );

        let nft_metadata: DepositMetadata;
        match DepositMetadata::top_decode(nft_info.attributes.clone().as_slice()) {
            Result::Ok(decoded) => {
                nft_metadata = decoded;
            }
            Result::Err(message) => {
                self.last_error_message()
                    .set(&BoxedBytes::from(message.message_bytes()));
                return sc_error!("could not parse token metadata");
            }
        }

        let time_in_pool = self.get_block_timestamp() - nft_metadata.timestamp;

        require!(time_in_pool > 0, "invalid timestamp");

        let withdraw_amount =
            self.calculate_amount_for_withdrawal(amount.clone(), BigUint::from(time_in_pool));

        let contract_balance = self.get_esdt_balance(
            &self.get_sc_address(),
            self.wegld_token().get().as_esdt_identifier(),
            0,
        );

        require!(
            withdraw_amount <= contract_balance,
            "the amount withdrawn is too high"
        );

        self.nft_burn(nft_type, nft_nonce, amount.clone());

        self.send().direct(
            &caller_address,
            &self.nft_token().get(),
            &amount,
            b"successful withdrawal",
        );

        Ok(())
    }

    #[view]
    fn calculate_amount_for_withdrawal(self, deposit_amount: BigUint, time: BigUint) -> BigUint {
        let percent = (time * self.deposit_apy().get()) / BigUint::from(SECONDS_PER_YEAR);

        return deposit_amount.clone()
            + ((percent.clone() * deposit_amount.clone()) / BigUint::from(BP));
    }

    fn nft_burn(&self, token_identifier: TokenIdentifier, nonce: u64, amount: BigUint) {
        self.send().esdt_nft_burn(
            self.get_gas_left(),
            token_identifier.as_esdt_identifier(),
            nonce,
            &amount,
        );
    }

    fn mint_deposit_nft(self, deposit_metadata: DepositMetadata, amount: BigUint) {
        self.send().esdt_nft_create::<DepositMetadata>(
            self.get_gas_left(),
            self.nft_token().get().as_esdt_identifier(),
            &amount,
            &BoxedBytes::empty(),
            &BigUint::zero(),
            &H256::zero(),
            &deposit_metadata,
            &[],
        )
    }

    fn convert_wegld(&self, pool_token: TokenIdentifier, amount: BigUint) {
        //TODO:  integration with dex
    }

    fn convert_to_wegld(&self, pool_token: TokenIdentifier, amount: BigUint) {
        //TODO:  integration with dex
    }

    //Storage
    #[view]
    #[storage_mapper("pools")]
    fn pools(&self, token: TokenIdentifier) -> SingleValueMapper<Self::Storage, Address>;

    #[view]
    #[storage_mapper("wegld_token")]
    fn wegld_token(&self) -> SingleValueMapper<Self::Storage, TokenIdentifier>;

    #[view]
    #[storage_mapper("deposit_apy")]
    fn deposit_apy(&self) -> SingleValueMapper<Self::Storage, BigUint>;

    #[view(nftToken)]
    #[storage_mapper("nftToken")]
    fn nft_token(&self) -> SingleValueMapper<Self::Storage, TokenIdentifier>;

    #[view(lastErrorMessage)]
    #[storage_mapper("lastErrorMessage")]
    fn last_error_message(&self) -> SingleValueMapper<Self::Storage, BoxedBytes>;

}
