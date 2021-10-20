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
        accept_funds_func: &Option<ManagedBuffer>,
    ) -> SCResult<()> {
        let (function, gas_limit) = self.get_func_and_gas_limit_from_opt(accept_funds_func);

        self.raw_vm_api()
            .direct_esdt_execute(
                dest,
                token_id,
                amount,
                gas_limit,
                &function,
                &ManagedArgBuffer::new_empty(self.type_manager()),
            )
            .into()
    }

    fn send_nft_tokens(
        &self,
        dest: &ManagedAddress,
        token_id: &TokenIdentifier,
        nonce: u64,
        amount: &BigUint,
        accept_funds_func: &Option<ManagedBuffer>,
    ) -> SCResult<()> {
        let (function, gas_limit) = self.get_func_and_gas_limit_from_opt(accept_funds_func);

        self.raw_vm_api()
            .direct_esdt_nft_execute(
                dest,
                token_id,
                nonce,
                amount,
                gas_limit,
                &function,
                &ManagedArgBuffer::new_empty(self.type_manager()),
            )
            .into()
    }

    fn get_func_and_gas_limit_from_opt(
        &self,
        accept_funds_func: &Option<ManagedBuffer>,
    ) -> (ManagedBuffer, u64) {
        let mut gas_limit = 0u64;
        let function = accept_funds_func
            .clone()
            .unwrap_or_else(|| ManagedBuffer::new());

        if !function.is_empty() {
            gas_limit = self.transfer_exec_gas_limit().get();
        }

        (function, gas_limit)
    }

    #[view(getTransferExecGasLimit)]
    #[storage_mapper("transfer_exec_gas_limit")]
    fn transfer_exec_gas_limit(&self) -> SingleValueMapper<u64>;
}
