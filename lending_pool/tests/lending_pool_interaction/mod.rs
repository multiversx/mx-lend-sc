use elrond_wasm::types::{Address, EsdtLocalRole};
use elrond_wasm_debug::{
    managed_address, managed_biguint, managed_token_id, rust_biguint,
    testing_framework::{BlockchainStateWrapper, ContractObjWrapper},
    DebugApi,
};
use lending_pool::router::RouterModule;
use liquidity_pool::storage::StorageModule;
use liquidity_pool::LiquidityPool;
use price_aggregator_proxy::PriceAggregatorModule;

use crate::{
    constants::{
        BORROW_EGLD, BORROW_USDC_TOKEN_ID, EGLD_TOKEN_ID, LEND_EGLD, LEND_USDC_TOKEN_ID,
        LIQ_THRESOLD, RESERVE_FACTOR, R_BASE, R_SLOPE1, R_SLOPE2, USDC_TOKEN_ID, U_OPTIMAL,
    },
    setup::*,
};

pub struct LendingSetup<LendingPoolObjBuilder, LiquidityPoolObjBuilder, PriceAggregatorObjBuilder>
where
    LendingPoolObjBuilder: 'static + Copy + Fn() -> lending_pool::ContractObj<DebugApi>,
    LiquidityPoolObjBuilder: 'static + Copy + Fn() -> liquidity_pool::ContractObj<DebugApi>,
    PriceAggregatorObjBuilder: 'static + Copy + Fn() -> aggregator_mock::ContractObj<DebugApi>,
{
    pub owner_addr: Address,
    pub first_user_addr: Address,
    pub second_user_addr: Address,
    pub price_aggregator_wrapper:
        ContractObjWrapper<aggregator_mock::ContractObj<DebugApi>, PriceAggregatorObjBuilder>,
    pub b_mock: BlockchainStateWrapper,
    pub lending_pool_wrapper:
        ContractObjWrapper<lending_pool::ContractObj<DebugApi>, LendingPoolObjBuilder>,
    pub liquidity_pool_usdc_wrapper:
        ContractObjWrapper<liquidity_pool::ContractObj<DebugApi>, LiquidityPoolObjBuilder>,
    pub liquidity_pool_egld_wrapper:
        ContractObjWrapper<liquidity_pool::ContractObj<DebugApi>, LiquidityPoolObjBuilder>,
}

impl<LendingPoolObjBuilder, LiquidityPoolObjBuilder, PriceAggregatorObjBuilder>
    LendingSetup<LendingPoolObjBuilder, LiquidityPoolObjBuilder, PriceAggregatorObjBuilder>
where
    LendingPoolObjBuilder: 'static + Copy + Fn() -> lending_pool::ContractObj<DebugApi>,
    LiquidityPoolObjBuilder: 'static + Copy + Fn() -> liquidity_pool::ContractObj<DebugApi>,
    PriceAggregatorObjBuilder: 'static + Copy + Fn() -> aggregator_mock::ContractObj<DebugApi>,
{
    /* Deploys Lending SC with a template Liquidity Pool */
    pub fn deploy_lending(
        lending_pool_builder: LendingPoolObjBuilder,
        liquidity_pool_builder: LiquidityPoolObjBuilder,
        price_aggregator_builder: PriceAggregatorObjBuilder,
    ) -> Self {
        let rust_zero = rust_biguint!(0u64);
        let mut b_mock = BlockchainStateWrapper::new();
        let owner_addr = b_mock.create_user_account(&rust_zero);
        let first_user_addr = b_mock.create_user_account(&rust_biguint!(100_000_000));
        let second_user_addr = b_mock.create_user_account(&rust_biguint!(100_000_000));

        let price_aggregator_wrapper =
            setup_price_aggregator(&owner_addr, &mut b_mock, price_aggregator_builder);

        let lending_pool_wrapper = setup_lending_pool(
            &owner_addr,
            &mut b_mock,
            lending_pool_builder,
            &Address::zero(),
        );

        let liquidity_pool_usdc_wrapper = b_mock.create_sc_account(
            &rust_biguint!(0u64),
            Some(&lending_pool_wrapper.address_ref()),
            liquidity_pool_builder,
            "liq_pool_template_other",
        );

        b_mock
            .execute_tx(
                &owner_addr,
                &liquidity_pool_usdc_wrapper,
                &rust_biguint!(0),
                |sc| {
                    sc.init(
                        managed_token_id!(USDC_TOKEN_ID),
                        managed_biguint!(R_BASE),
                        managed_biguint!(R_SLOPE1),
                        managed_biguint!(R_SLOPE2),
                        managed_biguint!(U_OPTIMAL),
                        managed_biguint!(RESERVE_FACTOR),
                        managed_biguint!(LIQ_THRESOLD),
                    );
                    sc.lend_token().set(managed_token_id!(LEND_USDC_TOKEN_ID));
                    sc.borrow_token()
                        .set(managed_token_id!(BORROW_USDC_TOKEN_ID));
                    sc.set_price_aggregator_address(managed_address!(
                        &price_aggregator_wrapper.address_ref()
                    ));
                },
            )
            .assert_ok();

        b_mock
            .execute_tx(
                &owner_addr,
                &lending_pool_wrapper,
                &rust_biguint!(0),
                |sc| {
                    sc.pools_map().insert(
                        managed_token_id!(USDC_TOKEN_ID),
                        managed_address!(&liquidity_pool_usdc_wrapper.address_ref()),
                    );
                    sc.pools_allowed()
                        .insert(managed_address!(&liquidity_pool_usdc_wrapper.address_ref()));
                },
            )
            .assert_ok();

        b_mock.set_esdt_balance(
            &liquidity_pool_usdc_wrapper.address_ref(),
            LEND_USDC_TOKEN_ID,
            &rust_biguint!(0),
        );

        b_mock.set_esdt_local_roles(
            liquidity_pool_usdc_wrapper.address_ref(),
            LEND_USDC_TOKEN_ID,
            &[
                EsdtLocalRole::NftCreate,
                EsdtLocalRole::NftAddQuantity,
                EsdtLocalRole::NftBurn,
            ],
        );

        b_mock.set_esdt_balance(
            &liquidity_pool_usdc_wrapper.address_ref(),
            BORROW_USDC_TOKEN_ID,
            &rust_biguint!(0),
        );

        b_mock.set_esdt_local_roles(
            liquidity_pool_usdc_wrapper.address_ref(),
            BORROW_USDC_TOKEN_ID,
            &[
                EsdtLocalRole::NftCreate,
                EsdtLocalRole::NftAddQuantity,
                EsdtLocalRole::NftBurn,
            ],
        );

        let liquidity_pool_egld_wrapper = b_mock.create_sc_account(
            &rust_biguint!(0u64),
            Some(&lending_pool_wrapper.address_ref()),
            liquidity_pool_builder,
            "liq_pool_template_other",
        );

        b_mock
            .execute_tx(
                &owner_addr,
                &liquidity_pool_egld_wrapper,
                &rust_biguint!(0),
                |sc| {
                    sc.init(
                        managed_token_id!(EGLD_TOKEN_ID),
                        managed_biguint!(R_BASE),
                        managed_biguint!(R_SLOPE1),
                        managed_biguint!(R_SLOPE2),
                        managed_biguint!(U_OPTIMAL),
                        managed_biguint!(RESERVE_FACTOR),
                        managed_biguint!(LIQ_THRESOLD),
                    );
                    sc.lend_token().set(managed_token_id!(LEND_EGLD));
                    sc.borrow_token().set(managed_token_id!(BORROW_EGLD));
                    sc.set_price_aggregator_address(managed_address!(
                        &price_aggregator_wrapper.address_ref()
                    ));
                },
            )
            .assert_ok();

        b_mock
            .execute_tx(
                &owner_addr,
                &lending_pool_wrapper,
                &rust_biguint!(0),
                |sc| {
                    sc.pools_map().insert(
                        managed_token_id!(EGLD_TOKEN_ID),
                        managed_address!(&liquidity_pool_egld_wrapper.address_ref()),
                    );
                    sc.pools_allowed()
                        .insert(managed_address!(&liquidity_pool_egld_wrapper.address_ref()));
                },
            )
            .assert_ok();

        b_mock.set_esdt_balance(
            &liquidity_pool_egld_wrapper.address_ref(),
            LEND_EGLD,
            &rust_biguint!(0),
        );

        b_mock.set_esdt_local_roles(
            liquidity_pool_egld_wrapper.address_ref(),
            LEND_EGLD,
            &[
                EsdtLocalRole::NftCreate,
                EsdtLocalRole::NftAddQuantity,
                EsdtLocalRole::NftBurn,
            ],
        );

        b_mock.set_esdt_balance(
            &liquidity_pool_egld_wrapper.address_ref(),
            BORROW_EGLD,
            &rust_biguint!(0),
        );

        b_mock.set_esdt_local_roles(
            liquidity_pool_egld_wrapper.address_ref(),
            BORROW_EGLD,
            &[
                EsdtLocalRole::NftCreate,
                EsdtLocalRole::NftAddQuantity,
                EsdtLocalRole::NftBurn,
            ],
        );

        Self {
            owner_addr,
            first_user_addr,
            second_user_addr,
            price_aggregator_wrapper,
            b_mock,
            lending_pool_wrapper,
            liquidity_pool_usdc_wrapper: liquidity_pool_usdc_wrapper,
            liquidity_pool_egld_wrapper: liquidity_pool_egld_wrapper,
        }
    }
}
