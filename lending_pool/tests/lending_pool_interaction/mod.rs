use elrond_wasm::types::Address;
use elrond_wasm_debug::{
    rust_biguint,
    testing_framework::{BlockchainStateWrapper, ContractObjWrapper},
    DebugApi,
};

use crate::setup::*;

pub struct LendingSetup<LendingPoolObjBuilder>
where
    LendingPoolObjBuilder: 'static + Copy + Fn() -> lending_pool::ContractObj<DebugApi>,
{
    pub owner_addr: Address,
    pub first_user_addr: Address,
    pub second_user_addr: Address,
    pub price_aggregator_addr: Address,
    pub b_mock: BlockchainStateWrapper,
    pub lending_pool_wrapper:
        ContractObjWrapper<lending_pool::ContractObj<DebugApi>, LendingPoolObjBuilder>,
}

impl<LendingPoolObjBuilder> LendingSetup<LendingPoolObjBuilder>
where
    LendingPoolObjBuilder: 'static + Copy + Fn() -> lending_pool::ContractObj<DebugApi>,
{
    pub fn new(lending_pool_builder: LendingPoolObjBuilder) -> Self {
        let rust_zero = rust_biguint!(0u64);
        let mut b_mock = BlockchainStateWrapper::new();
        let owner_addr = b_mock.create_user_account(&rust_zero);
        let first_user_addr = b_mock.create_user_account(&rust_biguint!(100_000_000));
        let second_user_addr = b_mock.create_user_account(&rust_biguint!(100_000_000));

        let price_aggregator_addr =
            setup_price_aggregator(&owner_addr, &mut b_mock, aggregator_mock::contract_obj);
        let lending_pool_wrapper =
            setup_lending_pool(&owner_addr, &mut b_mock, lending_pool_builder);

        Self {
            owner_addr,
            first_user_addr,
            second_user_addr,
            price_aggregator_addr,
            b_mock,
            lending_pool_wrapper,
        }
    }
}
