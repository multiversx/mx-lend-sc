use constants::*;

use elrond_wasm_debug::{managed_biguint, rust_biguint};
use lending_pool::BP;
use lending_pool_interaction::LendingSetup;
use liquidity_pool::liq_storage::StorageModule;

pub mod constants;
pub mod lending_pool_interaction;
pub mod setup;

#[test]
fn setup_all_test() {
    let _ = LendingSetup::deploy_lending(
        lending_pool::contract_obj,
        liquidity_pool::contract_obj,
        aggregator_mock::contract_obj,
    );
}

#[test]
fn enter_market() {
    let mut lending_setup = LendingSetup::deploy_lending(
        lending_pool::contract_obj,
        liquidity_pool::contract_obj,
        aggregator_mock::contract_obj,
    );
    let user_addr = lending_setup.first_user_addr.clone();

    lending_setup.enter_market(&user_addr);
}

#[test]
fn exit_market() {
    let mut lending_setup = LendingSetup::deploy_lending(
        lending_pool::contract_obj,
        liquidity_pool::contract_obj,
        aggregator_mock::contract_obj,
    );
    let user_addr = lending_setup.first_user_addr.clone();

    let account_nonce = lending_setup.enter_market(&user_addr);
    lending_setup.exit_market(&user_addr, account_nonce);
}

#[test]
fn deposit_test() {
    let mut lending_setup = LendingSetup::deploy_lending(
        lending_pool::contract_obj,
        liquidity_pool::contract_obj,
        aggregator_mock::contract_obj,
    );

    let user_addr = lending_setup.first_user_addr.clone();
    let account_nonce = lending_setup.enter_market(&user_addr);

    lending_setup
        .b_mock
        .set_esdt_balance(&user_addr, USDC_TOKEN_ID, &rust_biguint!(1_000));

    lending_setup.add_collateral(&user_addr, USDC_TOKEN_ID, 0, account_nonce, 1_000, 1_000);
}

#[test]
fn withdraw_test() {
    let mut lending_setup = LendingSetup::deploy_lending(
        lending_pool::contract_obj,
        liquidity_pool::contract_obj,
        aggregator_mock::contract_obj,
    );
    let user_addr = lending_setup.first_user_addr.clone();
    let account_nonce = lending_setup.enter_market(&user_addr);

    lending_setup
        .b_mock
        .set_esdt_balance(&user_addr, USDC_TOKEN_ID, &rust_biguint!(1_000));

    lending_setup.add_collateral(&user_addr, USDC_TOKEN_ID, 0, account_nonce, 1_000, 1_000);

    lending_setup.remove_collateral(
        &user_addr,
        USDC_TOKEN_ID,
        1_000,
        account_nonce,
        750,
        250,
        11,
        1_000_000_000,
    );
}

#[test]
fn borrow_test() {
    let mut lending_setup = LendingSetup::deploy_lending(
        lending_pool::contract_obj,
        liquidity_pool::contract_obj,
        aggregator_mock::contract_obj,
    );
    let user_addr = lending_setup.first_user_addr.clone();
    let account_nonce = lending_setup.enter_market(&user_addr);

    lending_setup
        .b_mock
        .set_esdt_balance(&user_addr, USDC_TOKEN_ID, &rust_biguint!(1_000));

    lending_setup.add_collateral(&user_addr, USDC_TOKEN_ID, 0, account_nonce, 1_000, 1_000);

    lending_setup.borrow(
        &user_addr,
        USDC_TOKEN_ID,
        0,
        account_nonce,
        250,
        750,
        250,
        1,
        1_000_000_000,
    );
}

#[test]
fn repay_test() {
    let mut lending_setup = LendingSetup::deploy_lending(
        lending_pool::contract_obj,
        liquidity_pool::contract_obj,
        aggregator_mock::contract_obj,
    );
    let user_addr = lending_setup.first_user_addr.clone();
    let account_nonce = lending_setup.enter_market(&user_addr);

    lending_setup
        .b_mock
        .set_esdt_balance(&user_addr, USDC_TOKEN_ID, &rust_biguint!(1_000));

    lending_setup.add_collateral(&user_addr, USDC_TOKEN_ID, 0, account_nonce, 1_000, 1_000);
    lending_setup.borrow(
        &user_addr,
        USDC_TOKEN_ID,
        0,
        account_nonce,
        250,
        750,
        250,
        1,
        1_000_000_000,
    );
    lending_setup.repay(
        &user_addr,
        USDC_TOKEN_ID,
        250,
        account_nonce,
        100,
        850,
        150,
        1,
        1_000_000_000,
    );
}

#[test]
fn add_collateral_test() {
    let mut lending_setup = LendingSetup::deploy_lending(
        lending_pool::contract_obj,
        liquidity_pool::contract_obj,
        aggregator_mock::contract_obj,
    );

    let user_addr = lending_setup.first_user_addr.clone();
    let account_nonce = lending_setup.enter_market(&user_addr);

    lending_setup
        .b_mock
        .set_esdt_balance(&user_addr, USDC_TOKEN_ID, &rust_biguint!(1_000));

    lending_setup.add_collateral(&user_addr, USDC_TOKEN_ID, 0, account_nonce, 500, 500);
    lending_setup.add_collateral(&user_addr, USDC_TOKEN_ID, 500, account_nonce, 200, 700);
    lending_setup.add_collateral(&user_addr, USDC_TOKEN_ID, 700, account_nonce, 150, 850);
}

#[test]
fn remove_collateral_test() {
    let mut lending_setup = LendingSetup::deploy_lending(
        lending_pool::contract_obj,
        liquidity_pool::contract_obj,
        aggregator_mock::contract_obj,
    );

    let user_addr = lending_setup.first_user_addr.clone();
    let account_nonce = lending_setup.enter_market(&user_addr);

    lending_setup
        .b_mock
        .set_esdt_balance(&user_addr, USDC_TOKEN_ID, &rust_biguint!(1_000));

    lending_setup.add_collateral(&user_addr, USDC_TOKEN_ID, 0, account_nonce, 700, 700);
    lending_setup.remove_collateral(
        &user_addr,
        USDC_TOKEN_ID,
        700,
        account_nonce,
        300,
        400,
        1,
        1_000_000_000,
    );
    lending_setup.add_collateral(&user_addr, USDC_TOKEN_ID, 400, account_nonce, 150, 550);
}

#[test]
fn liquidate_test() {
    let mut lending_setup = LendingSetup::deploy_lending(
        lending_pool::contract_obj,
        liquidity_pool::contract_obj,
        aggregator_mock::contract_obj,
    );

    let liquidatee_user = lending_setup.first_user_addr.clone();
    let liquidator_user = lending_setup.second_user_addr.clone();

    let liquidatee_account_nonce = lending_setup.enter_market(&liquidatee_user);

    lending_setup
        .b_mock
        .set_esdt_balance(&liquidatee_user, USDC_TOKEN_ID, &rust_biguint!(1_000));

    lending_setup.add_collateral(
        &liquidatee_user,
        USDC_TOKEN_ID,
        0,
        liquidatee_account_nonce,
        1_000,
        1_000,
    );
    lending_setup.borrow(
        &liquidatee_user,
        USDC_TOKEN_ID,
        0,
        liquidatee_account_nonce,
        600,
        400,
        600,
        1,
        1_000_000_000,
    );

    lending_setup
        .b_mock
        .set_esdt_balance(&liquidator_user, USDC_TOKEN_ID, &rust_biguint!(300));

    lending_setup.liquidate(
        &liquidator_user,
        &liquidatee_user,
        liquidatee_account_nonce,
        300,
        315,
        85,
    );
}

#[test]
fn scenario1() {
    let mut lending_setup = LendingSetup::deploy_lending(
        lending_pool::contract_obj,
        liquidity_pool::contract_obj,
        aggregator_mock::contract_obj,
    );
    let supplier_addr = lending_setup.first_user_addr.clone();
    let borrower_addr = lending_setup.second_user_addr.clone();
    let supplier_nonce = lending_setup.enter_market(&supplier_addr);
    let borrower_nonce = lending_setup.enter_market(&borrower_addr);

    lending_setup.b_mock.set_block_round(3);

    // Supply/Deposit
    lending_setup
        .b_mock
        .set_esdt_balance(&supplier_addr, USDC_TOKEN_ID, &rust_biguint!(2_000));

    lending_setup.add_collateral(
        &supplier_addr,
        USDC_TOKEN_ID,
        0,
        supplier_nonce,
        2_000,
        2_000,
    );

    // lending_setup.check_index(managed_biguint!(DECIMALS), USDC_TOKEN_ID);

    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let borrow_index = sc.borrow_index().get();
            assert_eq!(borrow_index, managed_biguint!(1_000_000_000));
        })
        .assert_ok();

    lending_setup.b_mock.set_block_round(5);

    lending_setup.borrow(
        &borrower_addr,
        USDC_TOKEN_ID,
        0,
        borrower_nonce,
        1_000,
        1_000,
        1_000,
        5,
        1_000_000_000,
    );

    // lending_setup.check_index(managed_biguint!(DECIMALS), USDC_TOKEN_ID);
    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let borrow_index = sc.borrow_index().get();
            assert_eq!(borrow_index, managed_biguint!(1_000_000_000));
        })
        .assert_ok();

    lending_setup.b_mock.set_block_round(7);

    // Repay
    lending_setup
        .b_mock
        .set_esdt_balance(&borrower_addr, USDC_TOKEN_ID, &rust_biguint!(1_050));

    lending_setup.repay(
        &borrower_addr,
        USDC_TOKEN_ID,
        1_000,
        borrower_nonce,
        1_050,
        2_050,
        0,
        7,
        1_000_000_000,
    );

    lending_setup
        .b_mock
        .check_esdt_balance(&borrower_addr, USDC_TOKEN_ID, &rust_biguint!(0));

    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let borrow_index = sc.borrow_index().get();
            assert_eq!(borrow_index, managed_biguint!(1_050_000_000));
        })
        .assert_ok();

    lending_setup.b_mock.set_block_round(11);

    lending_setup.remove_collateral(
        &supplier_addr,
        USDC_TOKEN_ID,
        2_050,
        supplier_nonce,
        2_000,
        0,
        11,
        1_000_000_000,
    );

    lending_setup
        .b_mock
        .check_esdt_balance(&supplier_addr, USDC_TOKEN_ID, &rust_biguint!(2_050));
}

#[test]
fn scenario2() {
    let mut lending_setup = LendingSetup::deploy_lending(
        lending_pool::contract_obj,
        liquidity_pool::contract_obj,
        aggregator_mock::contract_obj,
    );
    let supplier_addr = lending_setup.first_user_addr.clone();
    let borrower_addr = lending_setup.second_user_addr.clone();
    let supplier_nonce = lending_setup.enter_market(&supplier_addr);
    let borrower_nonce = lending_setup.enter_market(&borrower_addr);

    lending_setup.b_mock.set_block_round(3);

    // Supply/Deposit
    lending_setup
        .b_mock
        .set_esdt_balance(&supplier_addr, USDC_TOKEN_ID, &rust_biguint!(2_000));

    lending_setup.add_collateral(
        &supplier_addr,
        USDC_TOKEN_ID,
        0,
        supplier_nonce,
        2_000,
        2_000,
    );

    // lending_setup.check_index(managed_biguint!(DECIMALS), USDC_TOKEN_ID);

    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let borrow_index = sc.borrow_index().get();
            assert_eq!(borrow_index, 1_000_000_000);
        })
        .assert_ok();

    lending_setup.b_mock.set_block_round(5);

    lending_setup.borrow(
        &borrower_addr,
        USDC_TOKEN_ID,
        0,
        borrower_nonce,
        1_000,
        1_000,
        1_000,
        5,
        1_000_000_000,
    );

    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let borrow_index = sc.borrow_index().get();
            assert_eq!(borrow_index, 1_000_000_000);
        })
        .assert_ok();

    lending_setup.b_mock.set_block_round(7);

    // Repay 600 USDC + interest (30 USDC)
    lending_setup
        .b_mock
        .set_esdt_balance(&borrower_addr, USDC_TOKEN_ID, &rust_biguint!(650));

    lending_setup.repay(
        &borrower_addr,
        USDC_TOKEN_ID,
        1_000,
        borrower_nonce,
        650,
        1_650,
        400,
        5,
        1_000_000_000,
    );

    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let borrow_index = sc.borrow_index().get();
            assert_eq!(borrow_index, 1_050_000_000);
        })
        .assert_ok();

    lending_setup.b_mock.set_block_round(9);

    // repay the rest of 400 USDC
    lending_setup
        .b_mock
        .set_esdt_balance(&borrower_addr, USDC_TOKEN_ID, &rust_biguint!(408));

    lending_setup.repay(
        &borrower_addr,
        USDC_TOKEN_ID,
        400,
        borrower_nonce,
        408,
        2_058,
        0,
        7,
        1_050_000_000,
    );

    lending_setup
        .b_mock
        .check_esdt_balance(&borrower_addr, USDC_TOKEN_ID, &rust_biguint!(0));

    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let borrow_index = sc.borrow_index().get();
            assert_eq!(borrow_index, 1_070_000_000);
        })
        .assert_ok();

    lending_setup.b_mock.set_block_round(11);

    lending_setup.remove_collateral(
        &supplier_addr,
        USDC_TOKEN_ID,
        2_058,
        supplier_nonce,
        2_000,
        0,
        11,
        1_000_000_000,
    );

    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let borrow_index = sc.borrow_index().get();
            assert_eq!(borrow_index, 1_070_000_000);
        })
        .assert_ok();
}

#[test]
fn scenario3() {
    let mut lending_setup = LendingSetup::deploy_lending(
        lending_pool::contract_obj,
        liquidity_pool::contract_obj,
        aggregator_mock::contract_obj,
    );
    let supplier_addr = lending_setup.first_user_addr.clone();
    let borrower_addr = lending_setup.second_user_addr.clone();
    let supplier_nonce = lending_setup.enter_market(&supplier_addr);
    let borrower_nonce = lending_setup.enter_market(&borrower_addr);

    lending_setup.b_mock.set_block_round(3);

    // Supply/Deposit
    lending_setup
        .b_mock
        .set_esdt_balance(&supplier_addr, USDC_TOKEN_ID, &rust_biguint!(2_000));

    lending_setup.add_collateral(
        &supplier_addr,
        USDC_TOKEN_ID,
        0,
        supplier_nonce,
        2_000,
        2_000,
    );

    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let borrow_index = sc.borrow_index().get();
            assert_eq!(borrow_index, 1_000_000_000);
        })
        .assert_ok();

    lending_setup.b_mock.set_block_round(5);

    // Borrow
    lending_setup.borrow(
        &borrower_addr,
        USDC_TOKEN_ID,
        2_000,
        borrower_nonce,
        1_000,
        1_000,
        1_000,
        5,
        1_000_000_000,
    );

    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let borrow_index = sc.borrow_index().get();
            assert_eq!(borrow_index, 1_000_000_000);
        })
        .assert_ok();

    lending_setup.b_mock.set_block_round(7);

    // Repay
    lending_setup
        .b_mock
        .set_esdt_balance(&borrower_addr, USDC_TOKEN_ID, &rust_biguint!(1_050));

    lending_setup.repay(
        &borrower_addr,
        USDC_TOKEN_ID,
        1_000,
        borrower_nonce,
        1_050,
        2_050,
        0,
        5,
        1_000_000_000,
    );

    lending_setup
        .b_mock
        .check_esdt_balance(&borrower_addr, USDC_TOKEN_ID, &rust_biguint!(0));

    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let borrow_index = sc.borrow_index().get();
            assert_eq!(borrow_index, 1_050_000_000);
        })
        .assert_ok();

    lending_setup.b_mock.set_block_round(11);

    // Withdraw (400 USDC) - round 1
    lending_setup.remove_collateral(
        &supplier_addr,
        USDC_TOKEN_ID,
        2_050,
        supplier_nonce,
        400,
        1_640,
        11,
        1_000_000_000,
    );

    lending_setup
        .b_mock
        .check_esdt_balance(&supplier_addr, USDC_TOKEN_ID, &rust_biguint!(410));

    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let borrow_index = sc.borrow_index().get();
            assert_eq!(borrow_index, 1_050_000_000);
        })
        .assert_ok();

    lending_setup.b_mock.set_block_round(15);

    // Withdraw (400 USDC) - round 2
    lending_setup.remove_collateral(
        &supplier_addr,
        USDC_TOKEN_ID,
        1_640,
        supplier_nonce,
        400,
        1_230,
        15,
        1_000_000_000,
    );

    lending_setup
        .b_mock
        .check_esdt_balance(&supplier_addr, USDC_TOKEN_ID, &rust_biguint!(820));

    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let borrow_index = sc.borrow_index().get();
            assert_eq!(borrow_index, 1_050_000_000);
        })
        .assert_ok();
    lending_setup.b_mock.set_block_round(23);

    // Withdraw (400 USDC) - round 3

    lending_setup.remove_collateral(
        &supplier_addr,
        USDC_TOKEN_ID,
        1_230,
        supplier_nonce,
        400,
        820,
        25,
        1_000_000_000,
    );

    lending_setup
        .b_mock
        .check_esdt_balance(&supplier_addr, USDC_TOKEN_ID, &rust_biguint!(1_230));

    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let borrow_index = sc.borrow_index().get();
            assert_eq!(borrow_index, 1_050_000_000);
        })
        .assert_ok();

    // Withdraw (400 USDC) - round 4
    lending_setup.b_mock.set_block_round(27);

    lending_setup.remove_collateral(
        &supplier_addr,
        USDC_TOKEN_ID,
        820,
        supplier_nonce,
        400,
        410,
        27,
        1_000_000_000,
    );

    lending_setup
        .b_mock
        .check_esdt_balance(&supplier_addr, USDC_TOKEN_ID, &rust_biguint!(1_640));

    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let borrow_index = sc.borrow_index().get();
            assert_eq!(borrow_index, 1_050_000_000);
        })
        .assert_ok();

    lending_setup.b_mock.set_block_round(32);

    // Withdraw (400 USDC) - round 5

    lending_setup.remove_collateral(
        &supplier_addr,
        USDC_TOKEN_ID,
        1_640,
        supplier_nonce,
        400,
        0,
        32,
        1_000_000_000,
    );

    lending_setup
        .b_mock
        .check_esdt_balance(&supplier_addr, USDC_TOKEN_ID, &rust_biguint!(2_050));

    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let borrow_index = sc.borrow_index().get();
            assert_eq!(borrow_index, 1_050_000_000);
        })
        .assert_ok();
}

#[test]
fn scenario4() {
    let mut lending_setup = LendingSetup::deploy_lending(
        lending_pool::contract_obj,
        liquidity_pool::contract_obj,
        aggregator_mock::contract_obj,
    );
    let supplier_addr = lending_setup.first_user_addr.clone();
    let supplier2_addr = lending_setup.second_user_addr.clone();
    let borrower_addr = lending_setup.third_user_addr.clone();
    let supplier_nonce = lending_setup.enter_market(&supplier_addr);
    let supplier2_nonce = lending_setup.enter_market(&supplier2_addr);
    let borrower_nonce = lending_setup.enter_market(&borrower_addr);

    lending_setup.b_mock.set_block_round(3);

    // Supply/Deposit
    lending_setup
        .b_mock
        .set_esdt_balance(&supplier_addr, USDC_TOKEN_ID, &rust_biguint!(1_000));

    lending_setup.add_collateral(
        &supplier_addr,
        USDC_TOKEN_ID,
        0,
        supplier_nonce,
        1_000,
        1_000,
    );

    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let borrow_index = sc.borrow_index().get();
            assert_eq!(borrow_index, 1_000_000_000);
        })
        .assert_ok();

    lending_setup.b_mock.set_block_round(5);

    // Supply/Deposit
    lending_setup
        .b_mock
        .set_esdt_balance(&supplier2_addr, USDC_TOKEN_ID, &rust_biguint!(1_000));

    lending_setup.add_collateral(
        &supplier2_addr,
        USDC_TOKEN_ID,
        0,
        supplier2_nonce,
        1_000,
        2_000,
    );

    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let borrow_index = sc.borrow_index().get();
            assert_eq!(borrow_index, 1_000_000_000);
        })
        .assert_ok();

    lending_setup.b_mock.set_block_round(7);

    // Borrow
    lending_setup.borrow(
        &borrower_addr,
        USDC_TOKEN_ID,
        0,
        borrower_nonce,
        1_000,
        1_000,
        1_000,
        5,
        1_000_000_000,
    );

    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let borrow_index = sc.borrow_index().get();
            assert_eq!(borrow_index, 1_000_000_000);
        })
        .assert_ok();

    lending_setup.b_mock.set_block_round(9);

    // Repay
    lending_setup
        .b_mock
        .set_esdt_balance(&borrower_addr, USDC_TOKEN_ID, &rust_biguint!(1_050));

    lending_setup.repay(
        &borrower_addr,
        USDC_TOKEN_ID,
        1_000,
        borrower_nonce,
        1_050,
        2_050,
        0,
        5,
        1_000_000_000,
    );

    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let borrow_index = sc.borrow_index().get();
            assert_eq!(borrow_index, 1_050_000_000);
        })
        .assert_ok();

    lending_setup.b_mock.set_block_round(11);

    // Withdraw - Supplier1
    lending_setup.remove_collateral(
        &supplier_addr,
        USDC_TOKEN_ID,
        1_025,
        supplier_nonce,
        1_000,
        1_025,
        11,
        1_000_000_000,
    );

    lending_setup
        .b_mock
        .check_esdt_balance(&supplier_addr, USDC_TOKEN_ID, &rust_biguint!(1_025));

    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let borrow_index = sc.borrow_index().get();
            assert_eq!(borrow_index, 1_050_000_000);
        })
        .assert_ok();

    lending_setup.b_mock.set_block_round(13);

    // Withdraw - Supplier 2

    lending_setup.remove_collateral(
        &supplier2_addr,
        USDC_TOKEN_ID,
        1_025,
        supplier2_nonce,
        1_000,
        0,
        13,
        1_000_000_000,
    );

    lending_setup
        .b_mock
        .check_esdt_balance(&supplier2_addr, USDC_TOKEN_ID, &rust_biguint!(1_025));

    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let borrow_index = sc.borrow_index().get();
            assert_eq!(borrow_index, 1_050_000_000);
        })
        .assert_ok();
}

#[test]
fn scenario5() {
    let mut lending_setup = LendingSetup::deploy_lending(
        lending_pool::contract_obj,
        liquidity_pool::contract_obj,
        aggregator_mock::contract_obj,
    );
    let supplier_addr = lending_setup.first_user_addr.clone();
    let supplier2_addr = lending_setup.second_user_addr.clone();
    let borrower_addr = lending_setup.third_user_addr.clone();
    let supplier_nonce = lending_setup.enter_market(&supplier_addr);
    let supplier2_nonce = lending_setup.enter_market(&supplier2_addr);
    let borrower_nonce = lending_setup.enter_market(&borrower_addr);

    lending_setup.b_mock.set_block_round(3);
    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let borrow_index = sc.borrow_index().get();
            assert_eq!(borrow_index, 1_000_000_000);
        })
        .assert_ok();

    // Supply/Deposit
    lending_setup
        .b_mock
        .set_esdt_balance(&supplier_addr, USDC_TOKEN_ID, &rust_biguint!(2_000));

    lending_setup.add_collateral(
        &supplier_addr,
        USDC_TOKEN_ID,
        0,
        supplier_nonce,
        2_000,
        2_000,
    );

    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let borrow_index = sc.borrow_index().get();
            assert_eq!(borrow_index, 1_000_000_000);
        })
        .assert_ok();

    lending_setup.b_mock.set_block_round(5);

    // Borrow
    lending_setup.borrow(
        &borrower_addr,
        USDC_TOKEN_ID,
        0,
        borrower_nonce,
        400,
        1_600,
        400,
        5,
        1_000_000_000,
    );

    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let borrow_index = sc.borrow_index().get();
            assert_eq!(borrow_index, 1_000_000_000);
        })
        .assert_ok();

    lending_setup.b_mock.set_block_round(7);

    // Supply/Deposit
    lending_setup
        .b_mock
        .set_esdt_balance(&supplier2_addr, USDC_TOKEN_ID, &rust_biguint!(2_000));

    lending_setup.add_collateral(
        &supplier2_addr,
        USDC_TOKEN_ID,
        0,
        supplier2_nonce,
        2_000,
        3_600,
    );

    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let borrow_index = sc.borrow_index().get();
            assert_eq!(borrow_index, 1_020_000_000);
        })
        .assert_ok();

    lending_setup.b_mock.set_block_round(9);

    // Repay
    lending_setup
        .b_mock
        .set_esdt_balance(&borrower_addr, USDC_TOKEN_ID, &rust_biguint!(412));

    lending_setup.repay(
        &borrower_addr,
        USDC_TOKEN_ID,
        400,
        borrower_nonce,
        412,
        4_012,
        0,
        5,
        1_000_000_000,
    );

    lending_setup
        .b_mock
        .check_esdt_balance(&borrower_addr, USDC_TOKEN_ID, &rust_biguint!(0));

    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let borrow_index = sc.borrow_index().get();
            assert_eq!(borrow_index, 1_030_000_000);
        })
        .assert_ok();

    lending_setup.b_mock.set_block_round(11);

    // Withdraw - Supplier1
    lending_setup.remove_collateral(
        &supplier_addr,
        USDC_TOKEN_ID,
        2_010,
        supplier_nonce,
        2_000,
        2_002,
        11,
        1_000_000_000,
    );

    lending_setup
        .b_mock
        .check_esdt_balance(&supplier_addr, USDC_TOKEN_ID, &rust_biguint!(2_010));

    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let borrow_index = sc.borrow_index().get();
            assert_eq!(borrow_index, 1_030_000_000);
        })
        .assert_ok();

    lending_setup.b_mock.set_block_round(13);

    // Withdraw - Supplier 2
    lending_setup.remove_collateral(
        &supplier2_addr,
        USDC_TOKEN_ID,
        2_002,
        supplier2_nonce,
        2_000,
        0,
        13,
        1_004_000_000,
    );

    lending_setup
        .b_mock
        .check_esdt_balance(&supplier2_addr, USDC_TOKEN_ID, &rust_biguint!(2_002));

    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let borrow_index = sc.borrow_index().get();
            assert_eq!(borrow_index, 1_030_000_000);
        })
        .assert_ok();
}

#[test]
fn scenario6() {
    let mut lending_setup = LendingSetup::deploy_lending(
        lending_pool::contract_obj,
        liquidity_pool::contract_obj,
        aggregator_mock::contract_obj,
    );

    let alice_addr = lending_setup.first_user_addr.clone();
    let bob_addr = lending_setup.second_user_addr.clone();
    let charlie_addr = lending_setup.third_user_addr.clone();

    let alice_nonce = lending_setup.enter_market(&alice_addr);
    let bob_nonce = lending_setup.enter_market(&bob_addr);
    let charlie_nonce = lending_setup.enter_market(&charlie_addr);

    lending_setup.b_mock.set_block_round(3);
    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let borrow_index = sc.borrow_index().get();
            assert_eq!(borrow_index, 1_000_000_000);
        })
        .assert_ok();

    // Supply/Deposit - Alice
    lending_setup
        .b_mock
        .set_esdt_balance(&alice_addr, USDC_TOKEN_ID, &rust_biguint!(40_000 * BP));

    lending_setup.add_collateral(&alice_addr, USDC_TOKEN_ID, 0, alice_nonce, 40_000, 40_000);

    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let borrow_index = sc.borrow_index().get();
            assert_eq!(borrow_index, 1_000_000_000);
        })
        .assert_ok();

    lending_setup.b_mock.set_block_round(5);

    // Supply/Deposit - Bob
    lending_setup
        .b_mock
        .set_esdt_balance(&bob_addr, USDC_TOKEN_ID, &rust_biguint!(20_000));

    lending_setup.add_collateral(&bob_addr, USDC_TOKEN_ID, 0, bob_nonce, 20_000, 60_000);

    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let borrow_index = sc.borrow_index().get();
            assert_eq!(borrow_index, 1_000_000_000);
        })
        .assert_ok();

    lending_setup.b_mock.set_block_round(7);

    // Borrow - Bob
    lending_setup
        .b_mock
        .set_esdt_balance(&bob_addr, USDC_TOKEN_ID, &rust_biguint!(15_000));

    lending_setup.borrow(
        &bob_addr,
        USDC_TOKEN_ID,
        0,
        bob_nonce,
        15_000,
        45_000,
        15_000,
        5,
        1_000_000_000,
    );

    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let borrow_index = sc.borrow_index().get();
            assert_eq!(borrow_index, 1_000_000_000);
        })
        .assert_ok();

    lending_setup.b_mock.set_block_round(9);

    // Borrow - Alice
    lending_setup
        .b_mock
        .set_esdt_balance(&alice_addr, USDC_TOKEN_ID, &rust_biguint!(6_000));

    lending_setup.borrow(
        &alice_addr,
        USDC_TOKEN_ID,
        0,
        alice_nonce,
        6_000,
        39_000,
        21_000,
        9,
        1_025_000_000,
    );

    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let borrow_index = sc.borrow_index().get();
            assert_eq!(borrow_index, 1_025_000_000);
        })
        .assert_ok();

    lending_setup.b_mock.set_block_round(11);

    // Borrow - Charlie
    lending_setup
        .b_mock
        .set_esdt_balance(&charlie_addr, USDC_TOKEN_ID, &rust_biguint!(12_000));

    lending_setup.borrow(
        &charlie_addr,
        USDC_TOKEN_ID,
        0,
        charlie_nonce,
        12_000,
        27_000,
        33_000,
        11,
        1_060_000_000,
    );

    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let borrow_index = sc.borrow_index().get();
            assert_eq!(borrow_index, 1_060_000_000);
        })
        .assert_ok();

    lending_setup.b_mock.set_block_round(13);

    // Repay - Charlie
    lending_setup
        .b_mock
        .set_esdt_balance(&charlie_addr, USDC_TOKEN_ID, &rust_biguint!(12_660));

    lending_setup.repay(
        &charlie_addr,
        USDC_TOKEN_ID,
        12_000,
        charlie_nonce,
        12_660,
        39_660,
        21_000,
        11,
        1_060_000_000,
    );

    lending_setup
        .b_mock
        .check_esdt_balance(&charlie_addr, USDC_TOKEN_ID, &rust_biguint!(0));

    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let borrow_index = sc.borrow_index().get();
            assert_eq!(borrow_index, 1_115_000_000);
        })
        .assert_ok();

    lending_setup.b_mock.set_block_round(15);

    // Repay - Alice (1)
    lending_setup
        .b_mock
        .set_esdt_balance(&alice_addr, USDC_TOKEN_ID, &rust_biguint!(3_750));

    lending_setup.repay(
        &alice_addr,
        USDC_TOKEN_ID,
        6_000,
        alice_nonce,
        3_750,
        43_410,
        18_000,
        9,
        1_025_000_000,
    );

    lending_setup
        .b_mock
        .check_esdt_balance(&alice_addr, USDC_TOKEN_ID, &rust_biguint!(0));

    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let borrow_index = sc.borrow_index().get();
            assert_eq!(borrow_index, 1_150_000_000);
        })
        .assert_ok();

    lending_setup.b_mock.set_block_round(17);

    // Repay - Alice (2)
    lending_setup
        .b_mock
        .set_esdt_balance(&alice_addr, USDC_TOKEN_ID, &rust_biguint!(3_090));

    lending_setup.repay(
        &alice_addr,
        USDC_TOKEN_ID,
        3_000,
        alice_nonce,
        3_090,
        46_500,
        15_000,
        15,
        1_150_000_000,
    );

    lending_setup
        .b_mock
        .check_esdt_balance(&alice_addr, USDC_TOKEN_ID, &rust_biguint!(0));

    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let borrow_index = sc.borrow_index().get();
            assert_eq!(borrow_index, 1_180_000_000);
        })
        .assert_ok();

    lending_setup.b_mock.set_block_round(19);

    // Repay - Bob (1)
    lending_setup
        .b_mock
        .set_esdt_balance(&bob_addr, USDC_TOKEN_ID, &rust_biguint!(6_075));

    lending_setup.repay(
        &bob_addr,
        USDC_TOKEN_ID,
        15_000,
        bob_nonce,
        6_075,
        52_575,
        12_000,
        7,
        1_000_000_000,
    );

    lending_setup
        .b_mock
        .check_esdt_balance(&bob_addr, USDC_TOKEN_ID, &rust_biguint!(0));

    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let borrow_index = sc.borrow_index().get();
            assert_eq!(borrow_index, 1_205_000_000);
        })
        .assert_ok();

    lending_setup.b_mock.set_block_round(21);

    // Repay - Bob (2)
    lending_setup
        .b_mock
        .set_esdt_balance(&bob_addr, USDC_TOKEN_ID, &rust_biguint!(12_240));

    lending_setup.repay(
        &bob_addr,
        USDC_TOKEN_ID,
        12_000,
        bob_nonce,
        12_240,
        64_815,
        0,
        19,
        1_205_000_000,
    );

    lending_setup
        .b_mock
        .check_esdt_balance(&bob_addr, USDC_TOKEN_ID, &rust_biguint!(0));

    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let borrow_index = sc.borrow_index().get();
            assert_eq!(borrow_index, 1_225_000_000);
        })
        .assert_ok();

    lending_setup.b_mock.set_block_round(23);

    // Withdraw - Supplier1

    lending_setup.remove_collateral(
        &bob_addr,
        USDC_TOKEN_ID,
        21_605,
        bob_nonce,
        20_000,
        43_210,
        5,
        1_000_000_000,
    );

    lending_setup
        .b_mock
        .check_esdt_balance(&bob_addr, USDC_TOKEN_ID, &rust_biguint!(21_605));

    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let borrow_index = sc.borrow_index().get();
            assert_eq!(borrow_index, 1_225_000_000);
        })
        .assert_ok();

    // Withdraw - Supplier 2

    lending_setup.b_mock.set_block_round(25);

    lending_setup.remove_collateral(
        &alice_addr,
        USDC_TOKEN_ID,
        43_210,
        alice_nonce,
        40_000,
        0,
        3,
        1_000_000_000,
    );

    lending_setup
        .b_mock
        .check_esdt_balance(&alice_addr, USDC_TOKEN_ID, &rust_biguint!(43_210));

    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let borrow_index = sc.borrow_index().get();
            assert_eq!(borrow_index, 1_225_000_000);
        })
        .assert_ok();
}

#[test]
fn scenario7() {
    let mut lending_setup = LendingSetup::deploy_lending(
        lending_pool::contract_obj,
        liquidity_pool::contract_obj,
        aggregator_mock::contract_obj,
    );
    let supplier_addr = lending_setup.first_user_addr.clone();
    let supplier2_addr = lending_setup.second_user_addr.clone();
    let borrower_addr = lending_setup.third_user_addr.clone();
    let borrower2_addr = lending_setup.third_user_addr.clone();
    let supplier_nonce = lending_setup.enter_market(&supplier_addr);
    let supplier2_nonce = lending_setup.enter_market(&supplier2_addr);
    let borrower_nonce = lending_setup.enter_market(&borrower_addr);
    let borrower2_nonce = lending_setup.enter_market(&borrower2_addr);

    // Supply/Deposit
    lending_setup.b_mock.set_block_round(5);

    lending_setup
        .b_mock
        .set_esdt_balance(&supplier_addr, USDC_TOKEN_ID, &rust_biguint!(3_000));

    lending_setup.add_collateral(
        &supplier_addr,
        USDC_TOKEN_ID,
        0,
        supplier_nonce,
        3_000,
        3_000,
    );

    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let borrow_index = sc.borrow_index().get();
            assert_eq!(borrow_index, 1_000_000_000);
        })
        .assert_ok();

    lending_setup.b_mock.set_block_round(5);

    // Supply/Deposit
    lending_setup
        .b_mock
        .set_esdt_balance(&supplier2_addr, USDC_TOKEN_ID, &rust_biguint!(2_000));

    lending_setup.add_collateral(
        &supplier2_addr,
        USDC_TOKEN_ID,
        0,
        supplier2_nonce,
        2_000,
        5_000,
    );

    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let borrow_index = sc.borrow_index().get();
            assert_eq!(borrow_index, 1_000_000_000);
        })
        .assert_ok();

    lending_setup.b_mock.set_block_round(7);

    // Borrow
    lending_setup.borrow(
        &borrower_addr,
        USDC_TOKEN_ID,
        0,
        borrower_nonce,
        1_000,
        4_000,
        1_000,
        7,
        1_000_000_000,
    );

    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let borrow_index = sc.borrow_index().get();
            assert_eq!(borrow_index, 1_000_000_000);
        })
        .assert_ok();

    lending_setup.b_mock.set_block_round(8);

    // Borrow
    lending_setup.borrow(
        &borrower2_addr,
        USDC_TOKEN_ID,
        0,
        borrower2_nonce,
        2_000,
        2_000,
        3_000,
        8,
        1_010_000_000,
    );

    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let borrow_index = sc.borrow_index().get();
            assert_eq!(borrow_index, 1_010_000_000);
        })
        .assert_ok();

    // Still same round - 8
    // Repay
    lending_setup
        .b_mock
        .set_esdt_balance(&borrower_addr, USDC_TOKEN_ID, &rust_biguint!(1_010));

    lending_setup.repay(
        &borrower_addr,
        USDC_TOKEN_ID,
        1_000,
        borrower_nonce,
        1_010,
        3_010,
        2_000,
        7,
        1_000_000_000,
    );

    lending_setup
        .b_mock
        .check_esdt_balance(&borrower_addr, USDC_TOKEN_ID, &rust_biguint!(0));

    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let borrow_index = sc.borrow_index().get();
            assert_eq!(borrow_index, 1_010_000_000);
        })
        .assert_ok();

    lending_setup.b_mock.set_block_round(11);

    // Repay - Dave (2000 USD)
    lending_setup
        .b_mock
        .set_esdt_balance(&borrower2_addr, USDC_TOKEN_ID, &rust_biguint!(2_120));

    lending_setup.repay(
        &borrower2_addr,
        USDC_TOKEN_ID,
        2_000,
        borrower2_nonce,
        2_120,
        5_130,
        0,
        8,
        1_010_000_000,
    );

    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let borrow_index = sc.borrow_index().get();
            assert_eq!(borrow_index, 1_070_000_000);
        })
        .assert_ok();

    lending_setup.b_mock.set_block_round(13);

    // Withdraw - Supplier1
    lending_setup.remove_collateral(
        &supplier_addr,
        USDC_TOKEN_ID,
        3_000,
        supplier_nonce,
        3_000,
        2_052,
        3,
        1_000_000_000,
    );

    lending_setup
        .b_mock
        .check_esdt_balance(&supplier_addr, USDC_TOKEN_ID, &rust_biguint!(3_078));

    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let borrow_index = sc.borrow_index().get();
            assert_eq!(borrow_index, 1_070_000_000);
        })
        .assert_ok();

    lending_setup.b_mock.set_block_round(15);

    // Withdraw - Supplier 2
    lending_setup.remove_collateral(
        &supplier2_addr,
        USDC_TOKEN_ID,
        2_000,
        supplier2_nonce,
        2_000,
        0,
        3,
        1_000_000_000,
    );

    lending_setup
        .b_mock
        .check_esdt_balance(&supplier2_addr, USDC_TOKEN_ID, &rust_biguint!(2_052));

    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let borrow_index = sc.borrow_index().get();
            assert_eq!(borrow_index, 1_070_000_000);
        })
        .assert_ok();
}
