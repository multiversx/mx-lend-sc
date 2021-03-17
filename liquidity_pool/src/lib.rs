#![no_std]

use elrond_wasm::{HexCallDataSerializer, only_owner, require, sc_error};

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

const ESDT_SYSTEM_SC_ADDRESS: [u8; 32] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0xff, 0xff,
];

const ESDT_ISSUE_STRING: &[u8] = b"issue";
const ESDT_ISSUE_COST: u64 = 5000000000000000000;

const LEND_TOKEN_PREFIX: &[u8] = b"L";
const BORROW_TOKEN_PREFIX: &[u8] = b"B";
const LEND_TOKEN_NAME: &[u8] = b"IntBearing";
const DEBT_TOKEN_NAME: &[u8] = b"DebtBearing";

const EMPTY_TOKEN_ID: &[u8] = b"EGLD";

#[derive(TopEncode, TopDecode)]
pub enum EsdtOperation<BigUint: BigUintApi> {
    None,
    Issue,
    Mint(BigUint), // amount minted
}

#[elrond_wasm_derive::contract(LiquidityPoolImpl)]
pub trait LiquidityPool {
    
    #[init]
    fn init(&self, asset: TokenIdentifier) {
        self.pool_asset().set(&asset); 
    }

    #[payable("*")]
    #[endpoint]
    fn deposit_asset(
        &self,
        initial_caller: Address,
        #[payment_token] asset: TokenIdentifier, 
        #[payment] amount: BigUint
    ) -> SCResult<()> {
        require!(amount > 0, "payment must be greater than 0");
        require!(!initial_caller.is_zero(), "invalid address provided");

        let pool_asset = self.pool_asset().get();
        require!(
            asset.clone() == pool_asset.clone(), 
            "asset not supported for this liquidity pool"
        );

        let lend_token = self.lend_token().get();

        let mut lend_reserve = self.reserves().get(&lend_token.clone()).unwrap_or(BigUint::zero());
        let mut asset_reserve = self.reserves().get(&pool_asset.clone()).unwrap_or(BigUint::zero());
        
        require!(lend_reserve != BigUint::zero(), "lend reserve empty");
        require!(asset_reserve != BigUint::zero(), "asset reserve empty");

        if !(lend_reserve.clone() > amount.clone()) {
            // mint more lend tokens
        }
        
        self.send().direct(&initial_caller, &lend_token, &amount, &[]);

        lend_reserve -= amount.clone();
        asset_reserve += amount.clone();

        self.reserves().insert(lend_token, lend_reserve);
        self.reserves().insert(pool_asset, asset_reserve);

        Ok(())
    }

    #[payable("EGLD")]
    #[endpoint]
    fn issue(
        &self,
        token_ticker: TokenIdentifier,
        ticker_as_name: BoxedBytes,
        token_prefix: BoxedBytes,
        token_supply: BigUint,
        num_decimals: u8,
        #[payment] issue_cost: BigUint
    ) -> SCResult<()> {

        only_owner!(self, "only owner can issue new tokens");
        require!(
            issue_cost == BigUint::from(ESDT_ISSUE_COST),
            "payment should be exactly 5 EGLD"
        );
        let pool_asset = self.pool_asset().get();
        require!(
            token_ticker.clone() == pool_asset.clone(),
            "wrong ESDT asset identifier"
        );

        let existing_token;
        let interest_token_name;

        if token_prefix == BoxedBytes::from(LEND_TOKEN_PREFIX) {
            existing_token = self.lend_token().get();
            interest_token_name = [LEND_TOKEN_NAME, ticker_as_name.as_slice()].concat();
            self.latest_prefix().set(&BoxedBytes::from(LEND_TOKEN_PREFIX));
        } else if token_prefix == BoxedBytes::from(BORROW_TOKEN_PREFIX) {
            existing_token = self.borrow_token().get();
            interest_token_name = [DEBT_TOKEN_NAME, ticker_as_name.as_slice()].concat();
            self.latest_prefix().set(&BoxedBytes::from(BORROW_TOKEN_PREFIX));
        } else {
            return sc_error!("wrong token prefix")
        }
        
        let interest_token_ticker = [token_prefix.as_slice(), ticker_as_name.as_slice()].concat();
        
        require!(
            existing_token.as_name() == EMPTY_TOKEN_ID,
            "token already issued for this identifier"
        );

        self.issue_esdt(
            interest_token_name.as_slice(),
            interest_token_ticker.as_slice(),
            &token_supply,
            num_decimals
        );

        Ok(())
    }

    #[endpoint]
    fn issue_esdt(
        &self,
        token_display_name: &[u8],
        token_ticker: &[u8],
        supply: &BigUint,
        num_decimals: u8
    ) {
        let mut serializer = HexCallDataSerializer::new(ESDT_ISSUE_STRING);

        serializer.push_argument_bytes(token_display_name);
        serializer.push_argument_bytes(token_ticker);
        serializer.push_argument_bytes(&supply.to_bytes_be());
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
            EsdtOperation::Mint(_amount) => ()
        }

        self.clear_temporary_storage_esdt_operation(&original_tx_hash);
    }

    fn perform_esdt_issue_callback(&self, success: bool) {
        let token_identifier = self.call_value().token();
        let initial_supply = self.call_value().esdt_value();

        if success {
            let latest_prefix = self.latest_prefix().get();

            if latest_prefix == BoxedBytes::from(LEND_TOKEN_PREFIX) {
                self.lend_token().set(&token_identifier);
                self.reserves().insert(token_identifier.clone(), initial_supply);
                self.send_callback_result(token_identifier, b"setLendTokenAddress");
            } else {
                self.borrow_token().set(&token_identifier);
                self.reserves().insert(token_identifier.clone(), initial_supply);
                self.send_callback_result(token_identifier, b"setBorrowTokenAddress");
            }
        }

        // nothing to do in case of error
    }

    fn send_callback_result(&self, token: TokenIdentifier, endpoint: &[u8]) {
        let owner = self.get_owner_address();

        let mut args = ArgBuffer::new();
        args.push_argument_bytes(token.as_slice());

        self.send().execute_on_dest_context(
            self.get_gas_left(),
            &owner,
            &BigUint::zero(),
            endpoint,
            &args
        )
    }

    /// VIEWS

    #[view(getReserve)]
    fn get_reserve(&self, token: TokenIdentifier) -> BigUint {
        self.reserves().get(&token).unwrap_or(BigUint::zero())
    }

    #[view(poolAsset)]
    fn get_pool_asset(&self) -> TokenIdentifier {
        self.pool_asset().get()
    }

    #[view(lendToken)]
    fn get_lend_token(&self) -> TokenIdentifier {
        self.lend_token().get()
    }

    #[view(borrowToken)]
    fn get_borrow_token(&self) -> TokenIdentifier {
        self.borrow_token().get()
    }

    /// pool asset

    #[storage_mapper("pool_asset")]
    fn pool_asset(&self) -> SingleValueMapper<Self::Storage, TokenIdentifier>;

    ///
    /// lend token supported for asset
    
    #[storage_mapper("lend_token")]
    fn lend_token(&self) -> SingleValueMapper<Self::Storage, TokenIdentifier>;

    ///  
    /// borrow token supported for collateral

    #[storage_mapper("borrow_token")]
    fn borrow_token(&self) -> SingleValueMapper<Self::Storage, TokenIdentifier>;
    
    ///
    /// pool reserves

    #[storage_mapper("reserves")]
    fn reserves(&self) -> MapMapper<Self::Storage, TokenIdentifier, BigUint>;

    ///
    /// [set, get, clear] ESDT operation type

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

    ///
    /// temporary token prefix for ESDT operation

    #[view(latestPrefix)]
    #[storage_mapper("latest_prefix")]
    fn latest_prefix(&self) -> SingleValueMapper<Self::Storage, BoxedBytes>;
}
