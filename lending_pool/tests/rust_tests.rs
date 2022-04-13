use constants::*;
use elrond_wasm_debug::{managed_address, managed_biguint, managed_token_id, rust_biguint};
use lending_pool::TokenAmountPair;
use lending_pool_interaction::LendingSetup;
use liquidity_pool::{liquidity::LiquidityModule, storage::StorageModule};

pub mod constants;
pub mod lending_pool_interaction;
pub mod setup;

#[test]
fn setup_all_test() {
    let _ = LendingSetup::deploy_lending(lending_pool::contract_obj, liquidity_pool::contract_obj);
}

#[test]
fn deposit_test() {
    let mut lending_setup =
        LendingSetup::deploy_lending(lending_pool::contract_obj, liquidity_pool::contract_obj);
    let user_addr = lending_setup.first_user_addr.clone();

    lending_setup
        .b_mock
        .set_esdt_balance(&user_addr, USDC_TOKEN_ID, &rust_biguint!(1_000));

    lending_setup
        .b_mock
        .execute_esdt_transfer(
            &user_addr,
            &lending_setup.liquidity_pool_wrapper,
            USDC_TOKEN_ID,
            0,
            &rust_biguint!(1_000),
            |sc| {
                sc.deposit_asset(managed_address!(&user_addr));
            },
        )
        .assert_ok();

    lending_setup.b_mock.check_nft_balance(
        &user_addr,
        LEND_USDC_TOKEN_ID,
        1,
        &rust_biguint!(1_000),
        &(),
    );

    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_wrapper, |sc| {
            let actual_lend_token = sc.lend_token().get();
            let expected_lend_token = managed_token_id!(LEND_USDC_TOKEN_ID);
            assert_eq!(actual_lend_token, expected_lend_token, "Wrong token");
        })
        .assert_ok();
}

#[test]
fn borrow_test() {
    let mut lending_setup =
        LendingSetup::deploy_lending(lending_pool::contract_obj, liquidity_pool::contract_obj);
    let user_addr = lending_setup.first_user_addr.clone();

    lending_setup
        .b_mock
        .set_esdt_balance(&user_addr, USDC_TOKEN_ID, &rust_biguint!(1_000));

    lending_setup
        .b_mock
        .execute_esdt_transfer(
            &user_addr,
            &lending_setup.liquidity_pool_wrapper,
            USDC_TOKEN_ID,
            0,
            &rust_biguint!(1_000),
            |sc| {
                sc.deposit_asset(managed_address!(&user_addr));
            },
        )
        .assert_ok();

    lending_setup.b_mock.check_nft_balance(
        &user_addr,
        LEND_USDC_TOKEN_ID,
        1,
        &rust_biguint!(1_000),
        &(),
    );

    lending_setup
        .b_mock
        .set_nft_balance(&user_addr, LEND_EGLD, 1, &rust_biguint!(10), &());

    lending_setup
        .b_mock
        .execute_esdt_transfer(
            &user_addr,
            &lending_setup.liquidity_pool_wrapper,
            LEND_EGLD,
            1,
            &rust_biguint!(10),
            |sc| {
                sc.borrow(
                    managed_address!(&user_addr),
                    TokenAmountPair::new(managed_token_id!(EGLD_TOKEN_ID), 0, managed_biguint!(10)),
                    managed_biguint!(500_000_000),
                );
            },
        )
        .assert_ok();

    lending_setup.b_mock.check_nft_balance(
        &user_addr,
        BORROW_USDC_TOKEN_ID,
        1,
        &rust_biguint!(1_000),
        &Vec::<u8>::new(),
    );

    lending_setup.b_mock.check_nft_balance(
        &user_addr,
        USDC_TOKEN_ID,
        0,
        &rust_biguint!(1_000),
        &Vec::<u8>::new(),
    );
}
