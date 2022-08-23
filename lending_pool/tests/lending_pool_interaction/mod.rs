use elrond_wasm::{
    elrond_codec::Empty,
    types::{Address, BigUint, EsdtLocalRole},
};
use elrond_wasm_debug::{
    managed_address, managed_biguint, managed_token_id, rust_biguint,
    testing_framework::{BlockchainStateWrapper, ContractObjWrapper},
    DebugApi,
};
use lending_pool::{
    router::RouterModule, storage::LendingStorageModule, AccountTokenModule, BorrowPosition,
    DepositPosition, LendingPool, BP,
};
use liquidity_pool::LiquidityPool;
use liquidity_pool::{liq_storage::StorageModule, liquidity::LiquidityModule};
use price_aggregator_proxy::PriceAggregatorModule;

use crate::{
    constants::{
        ACCOUNT_TOKEN, EGLD_TOKEN_ID, LIQ_THRESOLD, RESERVE_FACTOR, R_BASE, R_SLOPE1, R_SLOPE2,
        USDC_TOKEN_ID, U_OPTIMAL,
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
    pub third_user_addr: Address,
    pub fourth_user_addr: Address,
    pub fifth_user_addr: Address,
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
        let third_user_addr = b_mock.create_user_account(&rust_biguint!(100_000_000));
        let fourth_user_addr = b_mock.create_user_account(&rust_biguint!(100_000_000));
        let fifth_user_addr = b_mock.create_user_account(&rust_biguint!(100_000_000));

        let price_aggregator_wrapper =
            setup_price_aggregator(&owner_addr, &mut b_mock, price_aggregator_builder);

        let lending_pool_wrapper = setup_lending_pool(
            &owner_addr,
            &mut b_mock,
            lending_pool_builder,
            price_aggregator_builder,
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
                    sc.set_asset_liquidation_bonus(
                        managed_token_id!(USDC_TOKEN_ID),
                        managed_biguint!(BP / 20),
                    );
                },
            )
            .assert_ok();

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

                    sc.set_asset_liquidation_bonus(
                        managed_token_id!(EGLD_TOKEN_ID),
                        managed_biguint!(BP / 20),
                    );
                },
            )
            .assert_ok();

        b_mock.set_esdt_local_roles(
            lending_pool_wrapper.address_ref(),
            ACCOUNT_TOKEN,
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
            third_user_addr,
            fourth_user_addr,
            fifth_user_addr,
            price_aggregator_wrapper,
            b_mock,
            lending_pool_wrapper,
            liquidity_pool_usdc_wrapper: liquidity_pool_usdc_wrapper,
            liquidity_pool_egld_wrapper: liquidity_pool_egld_wrapper,
        }
    }

    pub fn enter_market(&mut self, user_addr: &Address) -> u64 {
        let mut account_nonce = 0;
        self.b_mock
            .execute_tx(
                &user_addr,
                &self.lending_pool_wrapper,
                &rust_biguint!(0),
                |sc| {
                    account_nonce = sc.enter_market();
                    assert!(account_nonce != 0, "Account nonce didn't change");
                    assert!(
                        sc.account_positions().contains(&account_nonce),
                        "User didn't enter the market succefully!"
                    );
                },
            )
            .assert_ok();

        account_nonce
    }

    pub fn exit_market(&mut self, user_addr: &Address, account_nonce: u64) {
        self.b_mock
            .execute_esdt_transfer(
                &user_addr,
                &self.lending_pool_wrapper,
                ACCOUNT_TOKEN,
                account_nonce,
                &rust_biguint!(1),
                |sc| {
                    sc.exit_market();
                    assert!(
                        !sc.account_positions().contains(&account_nonce),
                        "User didn't exit the market succefully!"
                    );
                },
            )
            .assert_ok();
    }

    pub fn add_collateral(
        &mut self,
        user_addr: &Address,
        token_id: &[u8],
        initial_amount: u64,
        owner_nonce: u64,
        add_amount: u64,
        expected_reserves_after_deposit: u64,
    ) {
        let liquidity_pool_wrapper = match token_id {
            USDC_TOKEN_ID => &self.liquidity_pool_usdc_wrapper,
            EGLD_TOKEN_ID => &self.liquidity_pool_egld_wrapper,
            _ => todo!(),
        };

        self.b_mock
            .execute_esdt_transfer(
                &user_addr,
                &liquidity_pool_wrapper,
                token_id,
                0,
                &rust_biguint!(add_amount),
                |sc| {
                    sc.add_collateral(DepositPosition::new(
                        managed_token_id!(token_id),
                        managed_biguint!(initial_amount),
                        owner_nonce,
                        1,
                        managed_biguint!(BP),
                    ));
                },
            )
            .assert_ok();

        self.b_mock
            .execute_query(&liquidity_pool_wrapper, |sc| {
                let actual_deposited_collateral = sc.reserves().get();
                let expected_collateral = elrond_wasm::types::BigUint::from_bytes_be(
                    &expected_reserves_after_deposit.to_be_bytes(),
                );
                assert_eq!(
                    actual_deposited_collateral, expected_collateral,
                    "Reserve tokens in Liquidity Pool doesn't match!"
                );
            })
            .assert_ok();
    }

    pub fn remove_collateral(
        &mut self,
        user_addr: &Address,
        token_id: &[u8],
        initial_amount: u64,
        owner_nonce: u64,
        remove_amount: u64,
        expected_collateral_after_deposit: u64,
    ) {
        let liquidity_pool_wrapper = match token_id {
            USDC_TOKEN_ID => &self.liquidity_pool_usdc_wrapper,
            EGLD_TOKEN_ID => &self.liquidity_pool_egld_wrapper,
            _ => todo!(),
        };

        self.b_mock
            .execute_esdt_transfer(
                &user_addr,
                &liquidity_pool_wrapper,
                ACCOUNT_TOKEN,
                owner_nonce,
                &rust_biguint!(0),
                |sc| {
                    sc.remove_collateral(
                        managed_address!(&user_addr),
                        managed_biguint!(remove_amount),
                        DepositPosition::new(
                            managed_token_id!(token_id),
                            managed_biguint!(initial_amount),
                            owner_nonce,
                            1,
                            managed_biguint!(BP),
                        ),
                    );
                },
            )
            .assert_ok();

        self.b_mock
            .execute_query(&liquidity_pool_wrapper, |sc| {
                let actual_deposited_collateral = sc.reserves().get();
                let expected_collateral = elrond_wasm::types::BigUint::from_bytes_be(
                    &expected_collateral_after_deposit.to_be_bytes(),
                );
                assert_eq!(
                    actual_deposited_collateral, expected_collateral,
                    "Reserve tokens in Liquidity Pool doesn't match!"
                );
            })
            .assert_ok();
    }

    pub fn borrow(
        &mut self,
        user_addr: &Address,
        token_id: &[u8],
        initial_amount: u64,
        owner_nonce: u64,
        borrow_amount: u64,
        expected_reserves_after_borrow: u64,
        expected_borrowed_amount_after_borrow: u64,
    ) {
        let liquidity_pool_wrapper = match token_id {
            USDC_TOKEN_ID => &self.liquidity_pool_usdc_wrapper,
            EGLD_TOKEN_ID => &self.liquidity_pool_egld_wrapper,
            _ => todo!(),
        };

        self.b_mock
            .execute_esdt_transfer(
                &user_addr,
                &liquidity_pool_wrapper,
                ACCOUNT_TOKEN,
                owner_nonce,
                &rust_biguint!(0),
                |sc| {
                    sc.borrow(
                        managed_address!(&user_addr),
                        managed_biguint!(borrow_amount),
                        BorrowPosition::new(
                            managed_token_id!(token_id),
                            managed_biguint!(initial_amount),
                            owner_nonce,
                            1,
                            managed_biguint!(BP),
                        ),
                    );
                },
            )
            .assert_ok();

        self.b_mock
            .execute_query(&liquidity_pool_wrapper, |sc| {
                let actual_deposited_collateral = sc.reserves().get();
                let expected_collateral = elrond_wasm::types::BigUint::from_bytes_be(
                    &expected_reserves_after_borrow.to_be_bytes(),
                );
                assert_eq!(
                    actual_deposited_collateral, expected_collateral,
                    "Reserve tokens in Liquidity Pool doesn't match!"
                );
            })
            .assert_ok();

        self.b_mock
            .execute_query(&liquidity_pool_wrapper, |sc| {
                let actual_borrowed_amount = sc.borrowed_amount().get();
                let expected_borrowed = elrond_wasm::types::BigUint::from_bytes_be(
                    &expected_borrowed_amount_after_borrow.to_be_bytes(),
                );
                assert_eq!(
                    actual_borrowed_amount, expected_borrowed,
                    "Borrowed amount in Liquidity Pool doesn't match!"
                );
            })
            .assert_ok();
    }

    pub fn repay(
        &mut self,
        user_addr: &Address,
        token_id: &[u8],
        initial_amount: u64,
        owner_nonce: u64,
        repay_amount: u64,
        expected_reserves_after_repay: u64,
        expected_borrowed_amount_after_repay: u64,
    ) {
        let liquidity_pool_wrapper = match token_id {
            USDC_TOKEN_ID => &self.liquidity_pool_usdc_wrapper,
            EGLD_TOKEN_ID => &self.liquidity_pool_egld_wrapper,
            _ => todo!(),
        };

        self.b_mock
            .execute_esdt_transfer(
                &user_addr,
                &liquidity_pool_wrapper,
                USDC_TOKEN_ID,
                0,
                &rust_biguint!(repay_amount),
                |sc| {
                    sc.repay(
                        managed_address!(&user_addr),
                        BorrowPosition::new(
                            managed_token_id!(token_id),
                            managed_biguint!(initial_amount),
                            owner_nonce,
                            1,
                            managed_biguint!(BP),
                        ),
                    );
                },
            )
            .assert_ok();

        self.b_mock
            .execute_query(&liquidity_pool_wrapper, |sc| {
                let actual_deposited_collateral = sc.reserves().get();
                let expected_collateral = elrond_wasm::types::BigUint::from_bytes_be(
                    &expected_reserves_after_repay.to_be_bytes(),
                );
                assert_eq!(
                    actual_deposited_collateral, expected_collateral,
                    "Reserve tokens in Liquidity Pool doesn't match!"
                );
            })
            .assert_ok();

        self.b_mock
            .execute_query(&liquidity_pool_wrapper, |sc| {
                let actual_borrowed_amount = sc.borrowed_amount().get();
                let expected_borrowed = elrond_wasm::types::BigUint::from_bytes_be(
                    &expected_borrowed_amount_after_repay.to_be_bytes(),
                );
                assert_eq!(
                    actual_borrowed_amount, expected_borrowed,
                    "Borrowed amount in Liquidity Pool doesn't match!"
                );
            })
            .assert_ok();
    }

    pub fn liquidate(
        mut self,
        liquidator_user: &Address,
        liquidatee_user: &Address,
        liquidatee_nonce: u64,
        liquidation_amount: u64,
        liquidator_expected_amount: u64,
        contract_reserves_exected_amount: u64,
    ) {
        self.b_mock
            .execute_esdt_transfer(
                &liquidator_user,
                &self.lending_pool_wrapper,
                USDC_TOKEN_ID,
                0,
                &rust_biguint!(liquidation_amount),
                |sc| {
                    let nft_account_amount = BigUint::from(1u64);
                    let nft_token_payment = sc.account_token().nft_create_and_send(
                        &managed_address!(&liquidatee_user),
                        nft_account_amount,
                        &Empty,
                    );
                    sc.account_positions().insert(nft_token_payment.token_nonce);

                    sc.deposit_positions(liquidatee_nonce).insert(
                        managed_token_id!(USDC_TOKEN_ID),
                        DepositPosition::new(
                            managed_token_id!(USDC_TOKEN_ID),
                            managed_biguint!(1000),
                            liquidatee_nonce,
                            1,
                            managed_biguint!(BP),
                        ),
                    );
                    sc.deposit_positions(liquidatee_nonce).insert(
                        managed_token_id!(EGLD_TOKEN_ID),
                        DepositPosition::new(
                            managed_token_id!(EGLD_TOKEN_ID),
                            managed_biguint!(4),
                            liquidatee_nonce,
                            1,
                            managed_biguint!(BP),
                        ),
                    );

                    sc.borrow_positions(liquidatee_nonce).insert(
                        managed_token_id!(USDC_TOKEN_ID),
                        BorrowPosition::new(
                            managed_token_id!(USDC_TOKEN_ID),
                            managed_biguint!(600),
                            liquidatee_nonce,
                            2,
                            BigUint::from(BP),
                        ),
                    );

                    let threshold = BigUint::from(BP / 2);
                    sc.liquidate(liquidatee_nonce, threshold);
                },
            )
            .assert_ok();

        self.b_mock.check_esdt_balance(
            &liquidator_user,
            EGLD_TOKEN_ID,
            &rust_biguint!(liquidator_expected_amount),
        );

        self.b_mock.check_esdt_balance(
            &self.liquidity_pool_egld_wrapper.address_ref(),
            EGLD_TOKEN_ID,
            &rust_biguint!(contract_reserves_exected_amount),
        );
    }

    pub fn get_liquidity_pool_wrapper(
        &self,
        token_id: &[u8],
    ) -> &ContractObjWrapper<liquidity_pool::ContractObj<DebugApi>, LiquidityPoolObjBuilder> {
        match token_id {
            USDC_TOKEN_ID => &self.liquidity_pool_usdc_wrapper,
            EGLD_TOKEN_ID => &self.liquidity_pool_egld_wrapper,
            _ => todo!(),
        }
    }

    pub fn check_reserves(
        &mut self,
        expected_deposited_collateral: u64,
        liquidity_pool_wrapper: &ContractObjWrapper<
            liquidity_pool::ContractObj<DebugApi>,
            LiquidityPoolObjBuilder,
        >,
    ) {
        self.b_mock
            .execute_query(&liquidity_pool_wrapper, |sc| {
                let actual_deposited_collateral = sc.reserves().get();
                let expected_collateral = elrond_wasm::types::BigUint::from_bytes_be(
                    &expected_deposited_collateral.to_be_bytes(),
                );
                assert_eq!(
                    actual_deposited_collateral, expected_collateral,
                    "Reserve tokens in Liquidity Pool doesn't match!"
                );
            })
            .assert_ok();
    }

    pub fn check_borrowed_amount(
        &mut self,
        expected_borrowed_amount: u64,
        liquidity_pool_wrapper: &ContractObjWrapper<
            liquidity_pool::ContractObj<DebugApi>,
            LiquidityPoolObjBuilder,
        >,
    ) {
        self.b_mock
            .execute_query(&liquidity_pool_wrapper, |sc| {
                let actual_borrowed_amount = sc.borrowed_amount().get();
                let expected_borrowed = elrond_wasm::types::BigUint::from_bytes_be(
                    &expected_borrowed_amount.to_be_bytes(),
                );
                assert_eq!(
                    actual_borrowed_amount, expected_borrowed,
                    "Borrowed amount in Liquidity Pool doesn't match!"
                );
            })
            .assert_ok();
    }
}
