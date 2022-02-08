#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[elrond_wasm::module]
pub trait TokenSendModule {
    fn send_fft_tokens(
        &self,
        dest: &ManagedAddress,
        token_id: &TokenIdentifier,
        amount: &BigUint,
        accept_funds_func: &OptionalArg<ManagedBuffer>,
    ) {
        let (function, gas_limit) = self.get_func_and_gas_limit_from_opt(accept_funds_func);
        let _ = Self::Api::send_api_impl().direct_esdt_execute(
            dest,
            token_id,
            amount,
            gas_limit,
            &function,
            &ManagedArgBuffer::new_empty(),
        );
    }

    fn send_nft_tokens(
        &self,
        dest: &ManagedAddress,
        token_id: &TokenIdentifier,
        nonce: u64,
        amount: &BigUint,
        accept_funds_func: &OptionalArg<ManagedBuffer>,
    ) {
        let (function, gas_limit) = self.get_func_and_gas_limit_from_opt(accept_funds_func);
        let _ = Self::Api::send_api_impl().direct_esdt_nft_execute(
            dest,
            token_id,
            nonce,
            amount,
            gas_limit,
            &function,
            &ManagedArgBuffer::new_empty(),
        );
    }

    fn get_func_and_gas_limit_from_opt(
        &self,
        accept_funds_func: &OptionalArg<ManagedBuffer>,
    ) -> (ManagedBuffer, u64) {
        match accept_funds_func {
            OptionalArg::Some(accept_funds_func) => (
                accept_funds_func.clone(),
                self.transfer_exec_gas_limit().get(),
            ),
            OptionalArg::None => (ManagedBuffer::new(), 0u64),
        }
    }

    #[view(getTransferExecGasLimit)]
    #[storage_mapper("transfer_exec_gas_limit")]
    fn transfer_exec_gas_limit(&self) -> SingleValueMapper<u64>;
}
