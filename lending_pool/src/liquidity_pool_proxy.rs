use crate::RepayPostion;
use crate::LiquidateData;

elrond_wasm::imports!();

#[elrond_wasm_derive::callable(LiquidtyPoolProxy)]
pub trait LiquidtyPoolProxyImpl {
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
        #[payment_token] asset: TokenIdentifier,
        #[payment] amount: BigUint
    ) -> ContractCall<BigUint, RepayPostion<BigUint>>;

	#[payable("*")]
	fn liquidate(
		&self,
		position_id: H256,
		#[payment_token] token: TokenIdentifier,
		#[payment] amount: BigUint
	) -> ContractCall<BigUint, LiquidateData<BigUint>>;

	#[payable("*")]
	fn reject_funds(&self) -> ContractCall<BigUint, ()>;

	fn retrieve_funds(&self, token: TokenIdentifier, amount: BigUint) -> ContractCall<BigUint, ()>;
}