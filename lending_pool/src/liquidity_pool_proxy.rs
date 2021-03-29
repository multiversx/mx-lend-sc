use crate::RepayPostion;

elrond_wasm::imports!();

#[elrond_wasm_derive::callable(LiquidtyPoolProxy)]
pub trait LiquidtyPool {
	fn echo_arguments(
		&self,
		args: &VarArgs<BoxedBytes>,
	) -> ContractCall<BigUint, MultiResultVec<BoxedBytes>>;

	#[payable("*")]
	fn accept_funds(&self) -> ContractCall<BigUint, ()>;

	#[payable("*")]
    fn repay(
        &self,
        intitial_caller: Address,
        unique_id: H256,
        #[payment_token] asset: TokendIdentifier,
        #[payment] amount:BigUint
    ) -> ContractCall<RepayPostion<BigUint>>;

	#[payable("*")]
	fn reject_funds(&self) -> ContractCall<BigUint, ()>;

	fn retrieve_funds(&self, token: TokenIdentifier, amount: BigUint) -> ContractCall<BigUint, ()>;
}