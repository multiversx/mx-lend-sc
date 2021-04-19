use crate::LiquidateData;
use crate::RepayPostion;

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
        #[payment] amount: BigUint,
    ) -> ContractCall<BigUint, RepayPostion<BigUint>>;

    #[payable("*")]
    fn liquidate(
        &self,
        position_id: H256,
        #[payment_token] token: TokenIdentifier,
        #[payment] amount: BigUint,
    ) -> ContractCall<BigUint, LiquidateData<BigUint>>;


    fn borrow(
        &self,
        initial_caller: Address,
        lend_token: TokenIdentifier,
        amount: BigUint,
        timestamp: u64,
    ) -> ContractCall<BigUint, ()>;

    #[payable("*")]
    fn burnLendTokens(
        &self,
        initial_caller: Address,
    ) -> ContractCall<BigUint, ()>;

    #[payable("*")]
    fn reject_funds(&self) -> ContractCall<BigUint, ()>;

    fn retrieve_funds(&self, token: TokenIdentifier, amount: BigUint) -> ContractCall<BigUint, ()>;

    fn deposit_asset(&self, initial_caller: Address) -> ContractCall<BigUint, ()>;

    fn withdraw(&self, initial_caller: Address) -> ContractCall<BigUint, ()>;
}

#[elrond_wasm_derive::callable(RouterProxy)]
pub trait RouterProxyImpl {
    fn getPoolAddress(&self, asset: TokenIdentifier) -> ContractCall<BigUint, Address>;
}
