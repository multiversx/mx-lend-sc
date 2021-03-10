#![no_std]

use elrond_wasm::{only_owner, require, HexCallDataSerializer};

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

const ESDT_SYSTEM_SC_ADDRESS: [u8; 32] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0xff, 0xff,
];

const ESDT_ISSUE_COST: u64 = 5000000000000000000;
const ESDT_ISSUE_STRING: &[u8] = b"issue";
// const ESDT_MINT_STRING: &[u8] = b"mint";
const EMPTY_DATA: &[u8] = b"";

const E_TOKEN_PREFIX: &[u8] = b"E";
const E_TOKEN_NAME: &[u8] = b"IntBearing";

const EMPTY_TOKEN_ID: &[u8] = b"EGLD";

#[derive(TopEncode, TopDecode)]
pub enum EsdtOperation<BigUint: BigUintApi> {
    None,
    Issue,
    Mint(BigUint), // amount minted
}

#[elrond_wasm_derive::contract(ETokenHandlerImpl)]
pub trait ETokenHandler {
    #[storage_set("temporaryStorageEsdtOperation")]
    fn set_temporary_storage_esdt_operation(
        &self,
        original_tx_hash: &H256,
        esdt_operation: &EsdtOperation<BigUint>,
    );

    #[storage_get("temporaryStorageEsdtOperation")]
    fn get_temporary_storage_esdt_operation(
        &self,
        original_tx_hash: &H256,
    ) -> EsdtOperation<BigUint>;

    #[storage_clear("temporaryStorageEsdtOperation")]
    fn clear_temporary_storage_esdt_operation(&self, original_tx_hash: &H256);

    #[storage_set("supportedTokens")]
    fn set_supported_tokens(&self, esdt_token: &TokenIdentifier, e_token: &TokenIdentifier);

    #[view(getSupportedTokens)]
    #[storage_get("supportedTokens")]
    fn get_supported_tokens(&self, esdt_token: &TokenIdentifier) -> TokenIdentifier;

    #[storage_set("balance")]
    fn set_e_token_balance(&self, token: &TokenIdentifier, balance: &BigUint);

    #[view(getETokenBalance)]
    #[storage_get("balance")]
    fn get_e_token_balance(&self, token: &TokenIdentifier) -> BigUint;

    #[storage_set("latestEsdtIdentifier")]
    fn set_latest_esdt_identifier(&self, token: &TokenIdentifier);

    #[view(latestEsdtIndetifier)]
    #[storage_get("latestEsdtIdentifier")]
    fn get_latest_esdt_indetifier(&self) -> TokenIdentifier;

    #[init]
    fn init(&self) {}

    #[payable("EGLD")]
    #[endpoint]
    fn issue(
        &self,
        token_ticker: TokenIdentifier,
        ticker_as_name: BoxedBytes,
        initial_supply: BigUint,
        num_decimals: u8,
        #[payment] issue_cost: BigUint,
    ) -> SCResult<()> {
        only_owner!(self, "only owner can issue new eTokens");
        require!(
            issue_cost == BigUint::from(ESDT_ISSUE_COST),
            "you need exactly 5 egld to issue egld"
        );

        let existing_e_token = self.get_supported_tokens(&token_ticker);
        require!(
            existing_e_token.as_name() == EMPTY_TOKEN_ID,
            "eToken already issued for this identifier"
        );

        self.set_latest_esdt_identifier(&token_ticker);

        let e_token_name = [E_TOKEN_NAME, ticker_as_name.as_slice()].concat();
        let e_token_ticker = [E_TOKEN_PREFIX, ticker_as_name.as_slice()].concat();

        self.issue_esdt_token(
            e_token_name.as_slice(),
            e_token_ticker.as_slice(),
            &initial_supply,
            num_decimals,
        );

        Ok(())
    }

    #[payable("*")]
    #[endpoint]
    fn swap_esdt(
        &self,
        initial_caller: Address,
        #[payment_token] token_to_deposit: TokenIdentifier,
        #[payment] value: BigUint,
    ) -> SCResult<()> {
        only_owner!(self, "only owner can call this function");

        require!(value > 0, "payment must be greater than 0");
        require!(
            initial_caller != Address::zero(),
            "invalid address provided"
        );

        let result = self.get_supported_tokens(&token_to_deposit);
        require!(result.as_name() != EMPTY_TOKEN_ID, "token not supported");

        // TODO: check if e_token(ticker).balance > swap_value, otherwise mint

        let mut balance = self.get_e_token_balance(&result);
        balance -= value.clone();
        self.set_e_token_balance(&result, &balance);

        self.send().direct_esdt_via_transf_exec(
            &initial_caller,
            &result.as_slice(),
            &value,
            EMPTY_DATA,
        );

        Ok(())
    }

    #[endpoint]
    fn issue_esdt_token(
        &self,
        token_display_name: &[u8],
        token_ticker: &[u8],
        initial_supply: &BigUint,
        num_decimals: u8,
    ) {
        let mut serializer = HexCallDataSerializer::new(ESDT_ISSUE_STRING);

        serializer.push_argument_bytes(token_display_name);
        serializer.push_argument_bytes(token_ticker);
        serializer.push_argument_bytes(&initial_supply.to_bytes_be());
        serializer.push_argument_bytes(&[num_decimals]);

        serializer.push_argument_bytes(&b"canFreeze"[..]);
        serializer.push_argument_bytes(&b"false"[..]);

        serializer.push_argument_bytes(&b"canWipe"[..]);
        serializer.push_argument_bytes(&b"false"[..]);

        serializer.push_argument_bytes(&b"canPause"[..]);
        serializer.push_argument_bytes(&b"false"[..]);

        serializer.push_argument_bytes(&b"canMint"[..]);
        serializer.push_argument_bytes(&b"true"[..]);

        serializer.push_argument_bytes(&b"canBurn"[..]);
        serializer.push_argument_bytes(&b"false"[..]);

        serializer.push_argument_bytes(&b"canChangeOwner"[..]);
        serializer.push_argument_bytes(&b"false"[..]);

        serializer.push_argument_bytes(&b"canUpgrade"[..]);
        serializer.push_argument_bytes(&b"true"[..]);

        self.set_temporary_storage_esdt_operation(&self.get_tx_hash(), &EsdtOperation::Issue);

        self.send().async_call_raw(
            &Address::from(ESDT_SYSTEM_SC_ADDRESS),
            &BigUint::from(ESDT_ISSUE_COST),
            serializer.as_slice(),
        );
    }

    #[callback_raw]
    fn callback_raw(&self, #[var_args] result: AsyncCallResult<VarArgs<BoxedBytes>>) {
        let success = match result {
            AsyncCallResult::Ok(_) => true,
            AsyncCallResult::Err(_) => false,
        };
        let original_tx_hash = self.get_tx_hash();

        let esdt_operation = self.get_temporary_storage_esdt_operation(&original_tx_hash);
        match esdt_operation {
            EsdtOperation::None => return,
            EsdtOperation::Issue => self.perform_esdt_issue_callback(success),
            EsdtOperation::Mint(amount) => self.perform_esdt_mint_callback(success, &amount),
        };

        self.clear_temporary_storage_esdt_operation(&original_tx_hash);
    }

    fn perform_esdt_issue_callback(&self, success: bool) {
        // callback is called with ESDTTransfer of the newly issued token, with the amount requested,
        // so we can get the token identifier and amount from the call data
        let token_identifier = self.call_value().token();
        let initial_supply = self.call_value().esdt_value();

        if success {
            let last_esdt_identifier = self.get_latest_esdt_indetifier();

            self.set_supported_tokens(&last_esdt_identifier, &token_identifier);
            self.set_e_token_balance(&token_identifier, &initial_supply);
        }

        // nothing to do in case of error
    }

    fn perform_esdt_mint_callback(&self, success: bool, _amount: &BigUint) {
        if success {
            //self.add_total_esdt_token(amount);
        }

        // nothing to do in case of error
    }

    /*  fn add_total_esdt_token(&self, amount: &BigUint) {
        let mut total_wrapped = self.get_esdt_token_balance();
        total_wrapped += amount;
        self.set_esdt_token_balance(&total_wrapped);
    }*/
}
