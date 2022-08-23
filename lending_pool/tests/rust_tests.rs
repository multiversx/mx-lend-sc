use constants::*;

use elrond_wasm_debug::rust_biguint;
use lending_pool_interaction::LendingSetup;

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

    lending_setup.add_collateral(&user_addr, USDC_TOKEN_ID, 0, account_nonce, 1000, 1000);
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

    lending_setup.add_collateral(&user_addr, USDC_TOKEN_ID, 0, account_nonce, 1000, 1000);

    lending_setup.remove_collateral(&user_addr, USDC_TOKEN_ID, 1000, account_nonce, 750, 250);
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

    lending_setup.add_collateral(&user_addr, USDC_TOKEN_ID, 0, account_nonce, 1000, 1000);

    lending_setup.borrow(&user_addr, USDC_TOKEN_ID, 0, account_nonce, 250, 750, 250);
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

    lending_setup.add_collateral(&user_addr, USDC_TOKEN_ID, 0, account_nonce, 1000, 1000);
    lending_setup.borrow(&user_addr, USDC_TOKEN_ID, 0, account_nonce, 250, 750, 250);
    lending_setup.repay(&user_addr, USDC_TOKEN_ID, 250, account_nonce, 100, 850, 150);
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
    lending_setup.remove_collateral(&user_addr, USDC_TOKEN_ID, 700, account_nonce, 300, 400);
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
        1000,
        1000,
    );
    lending_setup.borrow(
        &liquidatee_user,
        USDC_TOKEN_ID,
        0,
        liquidatee_account_nonce,
        600,
        400,
        600,
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
fn liquidate_multiple_tokens_test() {
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

    lending_setup
        .b_mock
        .set_esdt_balance(&liquidatee_user, EGLD_TOKEN_ID, &rust_biguint!(4));

    lending_setup.add_collateral(
        &liquidatee_user,
        USDC_TOKEN_ID,
        0,
        liquidatee_account_nonce,
        1000,
        1000,
    );
    lending_setup.add_collateral(
        &liquidatee_user,
        EGLD_TOKEN_ID,
        0,
        liquidatee_account_nonce,
        4,
        4,
    );
    lending_setup.borrow(
        &liquidatee_user,
        USDC_TOKEN_ID,
        0,
        liquidatee_account_nonce,
        1000,
        0,
        1000,
    );

    lending_setup
        .b_mock
        .set_esdt_balance(&liquidator_user, USDC_TOKEN_ID, &rust_biguint!(500));

    lending_setup.liquidate(
        &liquidator_user,
        &liquidatee_user,
        liquidatee_account_nonce,
        500,
        2,
        2,
    );
}

// #[test]
// fn add_collateral_test() {
//     let mut lending_setup = LendingSetup::deploy_lending(
//         lending_pool::contract_obj,
//         liquidity_pool::contract_obj,
//         aggregator_mock::contract_obj,
//     );
//     let user_addr = lending_setup.first_user_addr.clone();

//     lending_setup
//         .b_mock
//         .set_esdt_balance(&user_addr, USDC_TOKEN_ID, &rust_biguint!(2_000 * BP));

//     lending_setup
//         .b_mock
//         .execute_esdt_transfer(
//             &user_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             USDC_TOKEN_ID,
//             0,
//             &rust_biguint!(2_000 * BP),
//             |sc| {
//                 sc.deposit_asset(managed_address!(&user_addr));
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &user_addr,
//         LEND_USDC_TOKEN_ID,
//         1,
//         &rust_biguint!(2_000 * BP),
//         Option::<&Empty>::None,
//     );

//     lending_setup.b_mock.set_block_round(10);

//     lending_setup
//         .b_mock
//         .set_nft_balance(&user_addr, LEND_EGLD, 1, &rust_biguint!(10 * BP), &Empty);

//     lending_setup
//         .b_mock
//         .execute_esdt_transfer(
//             &user_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             LEND_EGLD,
//             1,
//             &rust_biguint!(10 * BP),
//             |sc| {
//                 sc.borrow(managed_address!(&user_addr), managed_biguint!(500_000_000));
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &user_addr,
//         BORROW_USDC_TOKEN_ID,
//         1,
//         &rust_biguint!(1_000 * BP),
//         Some(&Vec::<u8>::new()),
//     );

//     lending_setup.b_mock.check_nft_balance(
//         &user_addr,
//         USDC_TOKEN_ID,
//         0,
//         &rust_biguint!(1_000 * BP),
//         Some(&Vec::<u8>::new()),
//     );

//     lending_setup.b_mock.set_block_round(12);

//     lending_setup
//         .b_mock
//         .set_nft_balance(&user_addr, LEND_EGLD, 2, &rust_biguint!(10 * BP), &Empty);

//     let mut payments = Vec::with_capacity(2);

//     payments.push(TxInputESDT {
//         token_identifier: BORROW_USDC_TOKEN_ID.to_vec(),
//         nonce: 1,
//         value: rust_biguint!(1_000 * BP),
//     });

//     payments.push(TxInputESDT {
//         token_identifier: LEND_EGLD.to_vec(),
//         nonce: 2,
//         value: rust_biguint!(10 * BP),
//     });

//     lending_setup
//         .b_mock
//         .execute_esdt_multi_transfer(
//             &user_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             &payments,
//             |sc| {
//                 sc.add_collateral(managed_address!(&user_addr), managed_biguint!(500_000_000));
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &user_addr,
//         BORROW_USDC_TOKEN_ID,
//         2,
//         &rust_biguint!(2_000 * BP),
//         Some(&Vec::<u8>::new()),
//     );

//     lending_setup.b_mock.check_nft_balance(
//         &user_addr,
//         USDC_TOKEN_ID,
//         0,
//         &rust_biguint!(1_000 * BP),
//         Some(&Vec::<u8>::new()),
//     );

//     lending_setup.b_mock.check_nft_balance(
//         &user_addr,
//         LEND_EGLD,
//         1,
//         &rust_biguint!(0),
//         Some(&Vec::<u8>::new()),
//     );
// }

// #[test]
// fn partial_liquidate_without_bonus_test() {
//     let mut lending_setup = LendingSetup::deploy_lending(
//         lending_pool::contract_obj,
//         liquidity_pool::contract_obj,
//         aggregator_mock::contract_obj,
//     );
//     let borrower_addr = lending_setup.first_user_addr.clone();
//     let liquidator_addr = lending_setup.second_user_addr.clone();

//     lending_setup
//         .b_mock
//         .set_esdt_balance(&borrower_addr, USDC_TOKEN_ID, &rust_biguint!(200_000));

//     // Deposit USDC
//     lending_setup
//         .b_mock
//         .execute_esdt_transfer(
//             &borrower_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             USDC_TOKEN_ID,
//             0,
//             &rust_biguint!(200_000),
//             |sc| {
//                 sc.deposit_asset(managed_address!(&borrower_addr));
//             },
//         )
//         .assert_ok();

//     // Check LUSDC
//     lending_setup.b_mock.check_nft_balance(
//         &borrower_addr,
//         LEND_USDC_TOKEN_ID,
//         1,
//         &rust_biguint!(200_000),
//         Option::<&Empty>::None,
//     );

//     lending_setup
//         .b_mock
//         .set_esdt_balance(&borrower_addr, EGLD_TOKEN_ID, &rust_biguint!(1_000));

//     // Deposit EGLD
//     lending_setup
//         .b_mock
//         .execute_esdt_transfer(
//             &borrower_addr,
//             &lending_setup.liquidity_pool_egld_wrapper,
//             EGLD_TOKEN_ID,
//             0,
//             &rust_biguint!(1_000),
//             |sc| {
//                 sc.deposit_asset(managed_address!(&borrower_addr));
//             },
//         )
//         .assert_ok();

//     // Check LEGLD
//     lending_setup.b_mock.check_nft_balance(
//         &borrower_addr,
//         LEND_EGLD,
//         1,
//         &rust_biguint!(1_000),
//         Option::<&Empty>::None,
//     );

//     // Borrow USDC
//     lending_setup
//         .b_mock
//         .execute_esdt_transfer(
//             &borrower_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             LEND_EGLD,
//             1,
//             &rust_biguint!(1_000),
//             |sc| {
//                 sc.borrow(
//                     managed_address!(&borrower_addr),
//                     managed_biguint!(500_000_000),
//                 );
//             },
//         )
//         .assert_ok();

//     // Check received BUSDC
//     lending_setup.b_mock.check_nft_balance(
//         &borrower_addr,
//         BORROW_USDC_TOKEN_ID,
//         1,
//         &rust_biguint!(100_000),
//         Some(&Vec::<u8>::new()),
//     );

//     // Check received USDC
//     lending_setup.b_mock.check_nft_balance(
//         &borrower_addr,
//         USDC_TOKEN_ID,
//         0,
//         &rust_biguint!(100_000),
//         Some(&Vec::<u8>::new()),
//     );

//     lending_setup
//         .b_mock
//         .execute_tx(
//             &liquidator_addr,
//             &lending_setup.price_aggregator_wrapper,
//             &rust_biguint!(0),
//             |sc| {
//                 sc.set_latest_price_feed(
//                     managed_buffer!(EGLD_TICKER),
//                     managed_buffer!(DOLLAR_TICKER),
//                     managed_biguint!(EGLD_PRICE_DROPPED_IN_DOLLARS),
//                 );
//             },
//         )
//         .assert_ok();

//     lending_setup
//         .b_mock
//         .set_esdt_balance(&liquidator_addr, USDC_TOKEN_ID, &rust_biguint!(100_000));

//     lending_setup
//         .b_mock
//         .execute_esdt_transfer(
//             &liquidator_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             USDC_TOKEN_ID,
//             0,
//             &rust_biguint!(100_000),
//             |sc| {
//                 sc.liquidate(managed_address!(&liquidator_addr), 1, managed_biguint!(0));
//             },
//         )
//         .assert_ok();

//     // Check Liquidator Balance - LEND_EGLD tokens
//     lending_setup.b_mock.check_nft_balance(
//         &liquidator_addr,
//         LEND_EGLD,
//         1,
//         &rust_biguint!(714),
//         Option::<&Empty>::None,
//     );
// }

// #[test]
// fn partial_liquidate_with_bonus_test() {
//     let mut lending_setup = LendingSetup::deploy_lending(
//         lending_pool::contract_obj,
//         liquidity_pool::contract_obj,
//         aggregator_mock::contract_obj,
//     );
//     let borrower_addr = lending_setup.first_user_addr.clone();
//     let liquidator_addr = lending_setup.second_user_addr.clone();

//     lending_setup
//         .b_mock
//         .set_esdt_balance(&borrower_addr, USDC_TOKEN_ID, &rust_biguint!(200_000));

//     // Deposit USDC
//     lending_setup
//         .b_mock
//         .execute_esdt_transfer(
//             &borrower_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             USDC_TOKEN_ID,
//             0,
//             &rust_biguint!(200_000),
//             |sc| {
//                 sc.deposit_asset(managed_address!(&borrower_addr));
//             },
//         )
//         .assert_ok();

//     // Check LUSDC
//     lending_setup.b_mock.check_nft_balance(
//         &borrower_addr,
//         LEND_USDC_TOKEN_ID,
//         1,
//         &rust_biguint!(200_000),
//         Option::<&Empty>::None,
//     );

//     lending_setup
//         .b_mock
//         .set_esdt_balance(&borrower_addr, EGLD_TOKEN_ID, &rust_biguint!(1_000));

//     // Deposit EGLD
//     lending_setup
//         .b_mock
//         .execute_esdt_transfer(
//             &borrower_addr,
//             &lending_setup.liquidity_pool_egld_wrapper,
//             EGLD_TOKEN_ID,
//             0,
//             &rust_biguint!(1_000),
//             |sc| {
//                 sc.deposit_asset(managed_address!(&borrower_addr));
//             },
//         )
//         .assert_ok();

//     // Check LEGLD
//     lending_setup.b_mock.check_nft_balance(
//         &borrower_addr,
//         LEND_EGLD,
//         1,
//         &rust_biguint!(1_000),
//         Option::<&Empty>::None,
//     );

//     // Borrow USDC
//     lending_setup
//         .b_mock
//         .execute_esdt_transfer(
//             &borrower_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             LEND_EGLD,
//             1,
//             &rust_biguint!(1_000),
//             |sc| {
//                 sc.borrow(
//                     managed_address!(&borrower_addr),
//                     managed_biguint!(500_000_000),
//                 );
//             },
//         )
//         .assert_ok();

//     // Check received BUSDC
//     lending_setup.b_mock.check_nft_balance(
//         &borrower_addr,
//         BORROW_USDC_TOKEN_ID,
//         1,
//         &rust_biguint!(100_000),
//         Some(&Vec::<u8>::new()),
//     );

//     // Check received USDC
//     lending_setup.b_mock.check_nft_balance(
//         &borrower_addr,
//         USDC_TOKEN_ID,
//         0,
//         &rust_biguint!(100_000),
//         Some(&Vec::<u8>::new()),
//     );

//     lending_setup
//         .b_mock
//         .execute_tx(
//             &liquidator_addr,
//             &lending_setup.price_aggregator_wrapper,
//             &rust_biguint!(0),
//             |sc| {
//                 sc.set_latest_price_feed(
//                     managed_buffer!(EGLD_TICKER),
//                     managed_buffer!(DOLLAR_TICKER),
//                     managed_biguint!(EGLD_PRICE_DROPPED_IN_DOLLARS),
//                 );
//             },
//         )
//         .assert_ok();

//     lending_setup
//         .b_mock
//         .set_esdt_balance(&liquidator_addr, USDC_TOKEN_ID, &rust_biguint!(100_000));

//     lending_setup
//         .b_mock
//         .execute_esdt_transfer(
//             &liquidator_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             USDC_TOKEN_ID,
//             0,
//             &rust_biguint!(100_000),
//             |sc| {
//                 sc.liquidate(
//                     managed_address!(&liquidator_addr),
//                     1,
//                     managed_biguint!(50_000_000),
//                 );
//             },
//         )
//         .assert_ok();

//     // Check Liquidator Balance - LEND_EGLD tokens
//     lending_setup.b_mock.check_nft_balance(
//         &liquidator_addr,
//         LEND_EGLD,
//         1,
//         &rust_biguint!(750),
//         Option::<&Empty>::None,
//     );
// }

// #[test]
// fn scenario1() {
//     let mut lending_setup = LendingSetup::deploy_lending(
//         lending_pool::contract_obj,
//         liquidity_pool::contract_obj,
//         aggregator_mock::contract_obj,
//     );
//     let supplier_addr = lending_setup.first_user_addr.clone();
//     let borrower_addr = lending_setup.second_user_addr.clone();

//     lending_setup.b_mock.set_block_round(3);

//     // Supply/Deposit
//     lending_setup.b_mock.set_esdt_balance(
//         &supplier_addr,
//         USDC_TOKEN_ID,
//         &rust_biguint!(2_000 * BP),
//     );

//     lending_setup
//         .b_mock
//         .execute_esdt_transfer(
//             &supplier_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             USDC_TOKEN_ID,
//             0,
//             &rust_biguint!(2_000 * BP),
//             |sc| {
//                 sc.deposit_asset(managed_address!(&supplier_addr));
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &supplier_addr,
//         LEND_USDC_TOKEN_ID,
//         1,
//         &rust_biguint!(2_000 * BP),
//         Option::<&Empty>::None,
//     );

//     lending_setup
//         .b_mock
//         .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
//             let borrow_index = sc.borrow_index().get();
//             assert_eq!(borrow_index, 1000000000);
//         })
//         .assert_ok();

//     lending_setup.b_mock.set_block_round(5);

//     // Borrow
//     lending_setup.b_mock.set_nft_balance(
//         &borrower_addr,
//         LEND_EGLD,
//         1,
//         &rust_biguint!(10 * BP),
//         &Empty,
//     );

//     lending_setup
//         .b_mock
//         .execute_esdt_transfer(
//             &borrower_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             LEND_EGLD,
//             1,
//             &rust_biguint!(10 * BP),
//             |sc| {
//                 sc.borrow(
//                     managed_address!(&borrower_addr),
//                     managed_biguint!(500_000_000),
//                 );
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &borrower_addr,
//         BORROW_USDC_TOKEN_ID,
//         1,
//         &rust_biguint!(1_000 * BP),
//         Some(&Vec::<u8>::new()),
//     );

//     lending_setup.b_mock.check_nft_balance(
//         &borrower_addr,
//         USDC_TOKEN_ID,
//         0,
//         &rust_biguint!(1_000 * BP),
//         Some(&Vec::<u8>::new()),
//     );

//     lending_setup
//         .b_mock
//         .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
//             let borrow_index = sc.borrow_index().get();
//             assert_eq!(borrow_index, 1000000000);
//         })
//         .assert_ok();

//     lending_setup.b_mock.set_block_round(7);

//     // Repay
//     lending_setup.b_mock.set_esdt_balance(
//         &borrower_addr,
//         USDC_TOKEN_ID,
//         &rust_biguint!(1_050 * BP),
//     );

//     let mut payments = Vec::with_capacity(2);

//     payments.push(TxInputESDT {
//         token_identifier: BORROW_USDC_TOKEN_ID.to_vec(),
//         nonce: 1,
//         value: rust_biguint!(1_000 * BP),
//     });

//     payments.push(TxInputESDT {
//         token_identifier: USDC_TOKEN_ID.to_vec(),
//         nonce: 0,
//         value: rust_biguint!(1_050 * BP),
//     });

//     lending_setup
//         .b_mock
//         .execute_esdt_multi_transfer(
//             &borrower_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             &payments,
//             |sc| {
//                 sc.repay(managed_address!(&borrower_addr));
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &borrower_addr,
//         BORROW_USDC_TOKEN_ID,
//         1,
//         &rust_biguint!(0),
//         Some(&Vec::<u8>::new()),
//     );

//     lending_setup
//         .b_mock
//         .check_esdt_balance(&borrower_addr, USDC_TOKEN_ID, &rust_biguint!(0));

//     lending_setup.b_mock.check_nft_balance(
//         &borrower_addr,
//         LEND_EGLD,
//         1,
//         &rust_biguint!(10 * BP),
//         Some(&Vec::<u8>::new()),
//     );

//     lending_setup
//         .b_mock
//         .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
//             let borrow_index = sc.borrow_index().get();
//             assert_eq!(borrow_index, 1050000000);
//         })
//         .assert_ok();

//     lending_setup.b_mock.set_block_round(11);

//     // Withdraw
//     lending_setup
//         .b_mock
//         .execute_esdt_transfer(
//             &supplier_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             LEND_USDC_TOKEN_ID,
//             1,
//             &rust_biguint!(2_000 * BP),
//             |sc| {
//                 sc.withdraw(managed_address!(&supplier_addr));
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &supplier_addr,
//         LEND_USDC_TOKEN_ID,
//         1,
//         &rust_biguint!(0),
//         Option::<&Empty>::None,
//     );

//     lending_setup.b_mock.check_nft_balance(
//         &supplier_addr,
//         USDC_TOKEN_ID,
//         0,
//         &rust_biguint!(2_050 * BP),
//         Some(&Empty),
//     );
// }

// #[test]
// fn scenario2() {
//     let mut lending_setup = LendingSetup::deploy_lending(
//         lending_pool::contract_obj,
//         liquidity_pool::contract_obj,
//         aggregator_mock::contract_obj,
//     );
//     let supplier_addr = lending_setup.first_user_addr.clone();
//     let borrower_addr = lending_setup.second_user_addr.clone();

//     lending_setup.b_mock.set_block_round(3);

//     // Supply/Deposit
//     lending_setup.b_mock.set_esdt_balance(
//         &supplier_addr,
//         USDC_TOKEN_ID,
//         &rust_biguint!(2_000 * BP),
//     );

//     lending_setup
//         .b_mock
//         .execute_esdt_transfer(
//             &supplier_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             USDC_TOKEN_ID,
//             0,
//             &rust_biguint!(2_000 * BP),
//             |sc| {
//                 sc.deposit_asset(managed_address!(&supplier_addr));
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &supplier_addr,
//         LEND_USDC_TOKEN_ID,
//         1,
//         &rust_biguint!(2_000 * BP),
//         Option::<&Empty>::None,
//     );

//     lending_setup
//         .b_mock
//         .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
//             let borrow_index = sc.borrow_index().get();
//             assert_eq!(borrow_index, 1000000000);
//         })
//         .assert_ok();

//     lending_setup.b_mock.set_block_round(5);

//     // Borrow
//     lending_setup.b_mock.set_nft_balance(
//         &borrower_addr,
//         LEND_EGLD,
//         1,
//         &rust_biguint!(10 * BP),
//         &Empty,
//     );

//     lending_setup
//         .b_mock
//         .execute_esdt_transfer(
//             &borrower_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             LEND_EGLD,
//             1,
//             &rust_biguint!(10 * BP),
//             |sc| {
//                 sc.borrow(
//                     managed_address!(&borrower_addr),
//                     managed_biguint!(500_000_000),
//                 );
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &borrower_addr,
//         BORROW_USDC_TOKEN_ID,
//         1,
//         &rust_biguint!(1_000 * BP),
//         Some(&Vec::<u8>::new()),
//     );

//     lending_setup.b_mock.check_nft_balance(
//         &borrower_addr,
//         USDC_TOKEN_ID,
//         0,
//         &rust_biguint!(1_000 * BP),
//         Some(&Vec::<u8>::new()),
//     );

//     lending_setup
//         .b_mock
//         .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
//             let borrow_index = sc.borrow_index().get();
//             assert_eq!(borrow_index, 1000000000);
//         })
//         .assert_ok();

//     lending_setup.b_mock.set_block_round(7);

//     // Repay 600 USDC + interest (30 USDC)
//     lending_setup
//         .b_mock
//         .set_esdt_balance(&borrower_addr, USDC_TOKEN_ID, &rust_biguint!(630 * BP));

//     let mut payments = Vec::with_capacity(2);

//     payments.push(TxInputESDT {
//         token_identifier: BORROW_USDC_TOKEN_ID.to_vec(),
//         nonce: 1,
//         value: rust_biguint!(600 * BP),
//     });

//     payments.push(TxInputESDT {
//         token_identifier: USDC_TOKEN_ID.to_vec(),
//         nonce: 0,
//         value: rust_biguint!(630 * BP),
//     });

//     lending_setup
//         .b_mock
//         .execute_esdt_multi_transfer(
//             &borrower_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             &payments,
//             |sc| {
//                 sc.repay(managed_address!(&borrower_addr));
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &borrower_addr,
//         BORROW_USDC_TOKEN_ID,
//         1,
//         &rust_biguint!(400 * BP),
//         Some(&Vec::<u8>::new()),
//     );

//     lending_setup
//         .b_mock
//         .check_esdt_balance(&borrower_addr, USDC_TOKEN_ID, &rust_biguint!(0));

//     lending_setup.b_mock.check_nft_balance(
//         &borrower_addr,
//         LEND_EGLD,
//         1,
//         &rust_biguint!(6 * BP),
//         Some(&Vec::<u8>::new()),
//     );

//     lending_setup
//         .b_mock
//         .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
//             let borrow_index = sc.borrow_index().get();
//             assert_eq!(borrow_index, 1050000000);
//         })
//         .assert_ok();

//     lending_setup.b_mock.set_block_round(9);

//     // repay the rest of 400 USDC
//     lending_setup.b_mock.set_esdt_balance(
//         &borrower_addr,
//         USDC_TOKEN_ID,
//         &rust_biguint!(427_881_772_800),
//     );

//     let mut payments = Vec::with_capacity(2);

//     payments.push(TxInputESDT {
//         token_identifier: BORROW_USDC_TOKEN_ID.to_vec(),
//         nonce: 1,
//         value: rust_biguint!(400 * BP),
//     });

//     payments.push(TxInputESDT {
//         token_identifier: USDC_TOKEN_ID.to_vec(),
//         nonce: 0,
//         value: rust_biguint!(427_881_772_800),
//     });

//     lending_setup
//         .b_mock
//         .execute_esdt_multi_transfer(
//             &borrower_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             &payments,
//             |sc| {
//                 sc.repay(managed_address!(&borrower_addr));
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &borrower_addr,
//         BORROW_USDC_TOKEN_ID,
//         1,
//         &rust_biguint!(0),
//         Some(&Vec::<u8>::new()),
//     );

//     lending_setup
//         .b_mock
//         .check_esdt_balance(&borrower_addr, USDC_TOKEN_ID, &rust_biguint!(0));

//     lending_setup.b_mock.check_nft_balance(
//         &borrower_addr,
//         LEND_EGLD,
//         1,
//         &rust_biguint!(10 * BP),
//         Some(&Vec::<u8>::new()),
//     );
//     lending_setup
//         .b_mock
//         .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
//             let borrow_index = sc.borrow_index().get();
//             assert_eq!(borrow_index, 1069704432);
//         })
//         .assert_ok();

//     lending_setup.b_mock.set_block_round(11);

//     // Withdraw
//     lending_setup
//         .b_mock
//         .execute_esdt_transfer(
//             &supplier_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             LEND_USDC_TOKEN_ID,
//             1,
//             &rust_biguint!(2_000 * BP),
//             |sc| {
//                 sc.withdraw(managed_address!(&supplier_addr));
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &supplier_addr,
//         LEND_USDC_TOKEN_ID,
//         1,
//         &rust_biguint!(0),
//         Option::<&Empty>::None,
//     );

//     lending_setup.b_mock.check_nft_balance(
//         &supplier_addr,
//         USDC_TOKEN_ID,
//         0,
//         &rust_biguint!(2_057_765_292_000),
//         Some(&Empty),
//     );

//     lending_setup
//         .b_mock
//         .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
//             let borrow_index = sc.borrow_index().get();
//             assert_eq!(borrow_index, 1069704432);
//         })
//         .assert_ok();
// }

// #[test]
// fn scenario3() {
//     let mut lending_setup = LendingSetup::deploy_lending(
//         lending_pool::contract_obj,
//         liquidity_pool::contract_obj,
//         aggregator_mock::contract_obj,
//     );
//     let supplier_addr = lending_setup.first_user_addr.clone();
//     let borrower_addr = lending_setup.second_user_addr.clone();

//     lending_setup.b_mock.set_block_round(3);

//     // Supply/Deposit
//     lending_setup.b_mock.set_esdt_balance(
//         &supplier_addr,
//         USDC_TOKEN_ID,
//         &rust_biguint!(2_000 * BP),
//     );

//     lending_setup
//         .b_mock
//         .execute_esdt_transfer(
//             &supplier_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             USDC_TOKEN_ID,
//             0,
//             &rust_biguint!(2_000 * BP),
//             |sc| {
//                 sc.deposit_asset(managed_address!(&supplier_addr));
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &supplier_addr,
//         LEND_USDC_TOKEN_ID,
//         1,
//         &rust_biguint!(2_000 * BP),
//         Option::<&Empty>::None,
//     );

//     lending_setup
//         .b_mock
//         .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
//             let borrow_index = sc.borrow_index().get();
//             assert_eq!(borrow_index, 1000000000);
//         })
//         .assert_ok();

//     lending_setup.b_mock.set_block_round(5);

//     // Borrow
//     lending_setup.b_mock.set_nft_balance(
//         &borrower_addr,
//         LEND_EGLD,
//         1,
//         &rust_biguint!(10 * BP),
//         &Empty,
//     );

//     lending_setup
//         .b_mock
//         .execute_esdt_transfer(
//             &borrower_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             LEND_EGLD,
//             1,
//             &rust_biguint!(10 * BP),
//             |sc| {
//                 sc.borrow(
//                     managed_address!(&borrower_addr),
//                     managed_biguint!(500_000_000),
//                 );
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &borrower_addr,
//         BORROW_USDC_TOKEN_ID,
//         1,
//         &rust_biguint!(1_000 * BP),
//         Some(&Vec::<u8>::new()),
//     );

//     lending_setup.b_mock.check_nft_balance(
//         &borrower_addr,
//         USDC_TOKEN_ID,
//         0,
//         &rust_biguint!(1_000 * BP),
//         Some(&Vec::<u8>::new()),
//     );

//     lending_setup
//         .b_mock
//         .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
//             let borrow_index = sc.borrow_index().get();
//             assert_eq!(borrow_index, 1000000000);
//         })
//         .assert_ok();

//     lending_setup.b_mock.set_block_round(7);

//     // Repay
//     lending_setup.b_mock.set_esdt_balance(
//         &borrower_addr,
//         USDC_TOKEN_ID,
//         &rust_biguint!(1_050 * BP),
//     );

//     let mut payments = Vec::with_capacity(2);

//     payments.push(TxInputESDT {
//         token_identifier: BORROW_USDC_TOKEN_ID.to_vec(),
//         nonce: 1,
//         value: rust_biguint!(1_000 * BP),
//     });

//     payments.push(TxInputESDT {
//         token_identifier: USDC_TOKEN_ID.to_vec(),
//         nonce: 0,
//         value: rust_biguint!(1_050 * BP),
//     });

//     lending_setup
//         .b_mock
//         .execute_esdt_multi_transfer(
//             &borrower_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             &payments,
//             |sc| {
//                 sc.repay(managed_address!(&borrower_addr));
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &borrower_addr,
//         BORROW_USDC_TOKEN_ID,
//         1,
//         &rust_biguint!(0),
//         Some(&Vec::<u8>::new()),
//     );

//     lending_setup
//         .b_mock
//         .check_esdt_balance(&borrower_addr, USDC_TOKEN_ID, &rust_biguint!(0));

//     lending_setup.b_mock.check_nft_balance(
//         &borrower_addr,
//         LEND_EGLD,
//         1,
//         &rust_biguint!(10 * BP),
//         Some(&Vec::<u8>::new()),
//     );

//     lending_setup
//         .b_mock
//         .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
//             let borrow_index = sc.borrow_index().get();
//             assert_eq!(borrow_index, 1050000000);
//         })
//         .assert_ok();

//     lending_setup.b_mock.set_block_round(11);

//     // Withdraw (400 USDC) - round 1
//     lending_setup
//         .b_mock
//         .execute_esdt_transfer(
//             &supplier_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             LEND_USDC_TOKEN_ID,
//             1,
//             &rust_biguint!(400 * BP),
//             |sc| {
//                 sc.withdraw(managed_address!(&supplier_addr));
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &supplier_addr,
//         LEND_USDC_TOKEN_ID,
//         1,
//         &rust_biguint!(1600 * BP),
//         Option::<&Empty>::None,
//     );

//     lending_setup.b_mock.check_nft_balance(
//         &supplier_addr,
//         USDC_TOKEN_ID,
//         0,
//         &rust_biguint!(410 * BP),
//         Some(&Empty),
//     );

//     lending_setup
//         .b_mock
//         .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
//             let borrow_index = sc.borrow_index().get();
//             assert_eq!(borrow_index, 1050000000);
//         })
//         .assert_ok();

//     lending_setup.b_mock.set_block_round(15);

//     // Withdraw (400 USDC) - round 2
//     lending_setup
//         .b_mock
//         .execute_esdt_transfer(
//             &supplier_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             LEND_USDC_TOKEN_ID,
//             1,
//             &rust_biguint!(400 * BP),
//             |sc| {
//                 sc.withdraw(managed_address!(&supplier_addr));
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &supplier_addr,
//         LEND_USDC_TOKEN_ID,
//         1,
//         &rust_biguint!(1200 * BP),
//         Option::<&Empty>::None,
//     );

//     lending_setup.b_mock.check_nft_balance(
//         &supplier_addr,
//         USDC_TOKEN_ID,
//         0,
//         &rust_biguint!(820 * BP),
//         Some(&Empty),
//     );

//     lending_setup
//         .b_mock
//         .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
//             let borrow_index = sc.borrow_index().get();
//             assert_eq!(borrow_index, 1050000000);
//         })
//         .assert_ok();

//     lending_setup.b_mock.set_block_round(23);

//     // Withdraw (400 USDC) - round 3
//     lending_setup
//         .b_mock
//         .execute_esdt_transfer(
//             &supplier_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             LEND_USDC_TOKEN_ID,
//             1,
//             &rust_biguint!(400 * BP),
//             |sc| {
//                 sc.withdraw(managed_address!(&supplier_addr));
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &supplier_addr,
//         LEND_USDC_TOKEN_ID,
//         1,
//         &rust_biguint!(800 * BP),
//         Option::<&Empty>::None,
//     );

//     lending_setup.b_mock.check_nft_balance(
//         &supplier_addr,
//         USDC_TOKEN_ID,
//         0,
//         &rust_biguint!(1230 * BP),
//         Some(&Empty),
//     );

//     lending_setup
//         .b_mock
//         .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
//             let borrow_index = sc.borrow_index().get();
//             assert_eq!(borrow_index, 1050000000);
//         })
//         .assert_ok();

//     lending_setup.b_mock.set_block_round(28);

//     // Withdraw (400 USDC) - round 4
//     lending_setup
//         .b_mock
//         .execute_esdt_transfer(
//             &supplier_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             LEND_USDC_TOKEN_ID,
//             1,
//             &rust_biguint!(400 * BP),
//             |sc| {
//                 sc.withdraw(managed_address!(&supplier_addr));
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &supplier_addr,
//         LEND_USDC_TOKEN_ID,
//         1,
//         &rust_biguint!(400 * BP),
//         Option::<&Empty>::None,
//     );

//     lending_setup.b_mock.check_nft_balance(
//         &supplier_addr,
//         USDC_TOKEN_ID,
//         0,
//         &rust_biguint!(1640 * BP),
//         Some(&Empty),
//     );

//     lending_setup
//         .b_mock
//         .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
//             let borrow_index = sc.borrow_index().get();
//             assert_eq!(borrow_index, 1050000000);
//         })
//         .assert_ok();

//     lending_setup.b_mock.set_block_round(32);

//     // Withdraw (400 USDC) - round 5
//     lending_setup
//         .b_mock
//         .execute_esdt_transfer(
//             &supplier_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             LEND_USDC_TOKEN_ID,
//             1,
//             &rust_biguint!(400 * BP),
//             |sc| {
//                 sc.withdraw(managed_address!(&supplier_addr));
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &supplier_addr,
//         LEND_USDC_TOKEN_ID,
//         1,
//         &rust_biguint!(0),
//         Option::<&Empty>::None,
//     );

//     lending_setup.b_mock.check_nft_balance(
//         &supplier_addr,
//         USDC_TOKEN_ID,
//         0,
//         &rust_biguint!(2050 * BP),
//         Some(&Empty),
//     );

//     lending_setup
//         .b_mock
//         .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
//             let borrow_index = sc.borrow_index().get();
//             assert_eq!(borrow_index, 1050000000);
//         })
//         .assert_ok();
// }

// #[test]
// fn scenario4() {
//     let mut lending_setup = LendingSetup::deploy_lending(
//         lending_pool::contract_obj,
//         liquidity_pool::contract_obj,
//         aggregator_mock::contract_obj,
//     );
//     let supplier_addr = lending_setup.first_user_addr.clone();
//     let supplier2_addr = lending_setup.second_user_addr.clone();
//     let borrower_addr = lending_setup.third_user_addr.clone();

//     lending_setup.b_mock.set_block_round(3);

//     // Supply/Deposit
//     lending_setup.b_mock.set_esdt_balance(
//         &supplier_addr,
//         USDC_TOKEN_ID,
//         &rust_biguint!(1_000 * BP),
//     );

//     lending_setup
//         .b_mock
//         .execute_esdt_transfer(
//             &supplier_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             USDC_TOKEN_ID,
//             0,
//             &rust_biguint!(1_000 * BP),
//             |sc| {
//                 sc.deposit_asset(managed_address!(&supplier_addr));
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &supplier_addr,
//         LEND_USDC_TOKEN_ID,
//         1,
//         &rust_biguint!(1_000 * BP),
//         Option::<&Empty>::None,
//     );

//     lending_setup
//         .b_mock
//         .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
//             let borrow_index = sc.borrow_index().get();
//             assert_eq!(borrow_index, 1000000000);
//         })
//         .assert_ok();

//     lending_setup.b_mock.set_block_round(5);

//     // Supply/Deposit
//     lending_setup.b_mock.set_esdt_balance(
//         &supplier2_addr,
//         USDC_TOKEN_ID,
//         &rust_biguint!(1_000 * BP),
//     );

//     lending_setup
//         .b_mock
//         .execute_esdt_transfer(
//             &supplier2_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             USDC_TOKEN_ID,
//             0,
//             &rust_biguint!(1_000 * BP),
//             |sc| {
//                 sc.deposit_asset(managed_address!(&supplier2_addr));
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &supplier2_addr,
//         LEND_USDC_TOKEN_ID,
//         2,
//         &rust_biguint!(1_000 * BP),
//         Option::<&Empty>::None,
//     );

//     lending_setup
//         .b_mock
//         .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
//             let borrow_index = sc.borrow_index().get();
//             assert_eq!(borrow_index, 1000000000);
//         })
//         .assert_ok();

//     lending_setup.b_mock.set_block_round(7);

//     // Borrow
//     lending_setup.b_mock.set_nft_balance(
//         &borrower_addr,
//         LEND_EGLD,
//         1,
//         &rust_biguint!(10 * BP),
//         &Empty,
//     );

//     lending_setup
//         .b_mock
//         .execute_esdt_transfer(
//             &borrower_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             LEND_EGLD,
//             1,
//             &rust_biguint!(10 * BP),
//             |sc| {
//                 sc.borrow(
//                     managed_address!(&borrower_addr),
//                     managed_biguint!(500_000_000),
//                 );
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &borrower_addr,
//         BORROW_USDC_TOKEN_ID,
//         1,
//         &rust_biguint!(1_000 * BP),
//         Some(&Vec::<u8>::new()),
//     );

//     lending_setup.b_mock.check_nft_balance(
//         &borrower_addr,
//         USDC_TOKEN_ID,
//         0,
//         &rust_biguint!(1_000 * BP),
//         Some(&Vec::<u8>::new()),
//     );

//     lending_setup
//         .b_mock
//         .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
//             let borrow_index = sc.borrow_index().get();
//             assert_eq!(borrow_index, 1000000000);
//         })
//         .assert_ok();

//     lending_setup.b_mock.set_block_round(9);

//     // Repay
//     lending_setup.b_mock.set_esdt_balance(
//         &borrower_addr,
//         USDC_TOKEN_ID,
//         &rust_biguint!(1_050 * BP),
//     );

//     let mut payments = Vec::with_capacity(2);

//     payments.push(TxInputESDT {
//         token_identifier: BORROW_USDC_TOKEN_ID.to_vec(),
//         nonce: 1,
//         value: rust_biguint!(1_000 * BP),
//     });

//     payments.push(TxInputESDT {
//         token_identifier: USDC_TOKEN_ID.to_vec(),
//         nonce: 0,
//         value: rust_biguint!(1_050 * BP),
//     });

//     lending_setup
//         .b_mock
//         .execute_esdt_multi_transfer(
//             &borrower_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             &payments,
//             |sc| {
//                 sc.repay(managed_address!(&borrower_addr));
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &borrower_addr,
//         BORROW_USDC_TOKEN_ID,
//         1,
//         &rust_biguint!(0),
//         Some(&Vec::<u8>::new()),
//     );

//     lending_setup
//         .b_mock
//         .check_esdt_balance(&borrower_addr, USDC_TOKEN_ID, &rust_biguint!(0));

//     lending_setup.b_mock.check_nft_balance(
//         &borrower_addr,
//         LEND_EGLD,
//         1,
//         &rust_biguint!(10 * BP),
//         Some(&Vec::<u8>::new()),
//     );

//     lending_setup
//         .b_mock
//         .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
//             let borrow_index = sc.borrow_index().get();
//             assert_eq!(borrow_index, 1050000000);
//         })
//         .assert_ok();

//     lending_setup.b_mock.set_block_round(11);

//     // Withdraw - Supplier1
//     lending_setup
//         .b_mock
//         .execute_esdt_transfer(
//             &supplier_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             LEND_USDC_TOKEN_ID,
//             1,
//             &rust_biguint!(1_000 * BP),
//             |sc| {
//                 sc.withdraw(managed_address!(&supplier_addr));
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &supplier_addr,
//         LEND_USDC_TOKEN_ID,
//         1,
//         &rust_biguint!(0),
//         Option::<&Empty>::None,
//     );

//     lending_setup.b_mock.check_nft_balance(
//         &supplier_addr,
//         USDC_TOKEN_ID,
//         0,
//         &rust_biguint!(1_025 * BP),
//         Some(&Empty),
//     );

//     lending_setup
//         .b_mock
//         .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
//             let borrow_index = sc.borrow_index().get();
//             assert_eq!(borrow_index, 1050000000);
//         })
//         .assert_ok();

//     lending_setup.b_mock.set_block_round(13);

//     // Withdraw - Supplier 2
//     lending_setup
//         .b_mock
//         .execute_esdt_transfer(
//             &supplier2_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             LEND_USDC_TOKEN_ID,
//             2,
//             &rust_biguint!(1_000 * BP),
//             |sc| {
//                 sc.withdraw(managed_address!(&supplier2_addr));
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &supplier2_addr,
//         LEND_USDC_TOKEN_ID,
//         1,
//         &rust_biguint!(0),
//         Option::<&Empty>::None,
//     );

//     lending_setup.b_mock.check_nft_balance(
//         &supplier2_addr,
//         USDC_TOKEN_ID,
//         0,
//         &rust_biguint!(1_025 * BP),
//         Some(&Empty),
//     );

//     lending_setup
//         .b_mock
//         .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
//             let borrow_index = sc.borrow_index().get();
//             assert_eq!(borrow_index, 1050000000);
//         })
//         .assert_ok();
// }

// #[test]
// fn scenario5() {
//     let mut lending_setup = LendingSetup::deploy_lending(
//         lending_pool::contract_obj,
//         liquidity_pool::contract_obj,
//         aggregator_mock::contract_obj,
//     );
//     let supplier_addr = lending_setup.first_user_addr.clone();
//     let supplier2_addr = lending_setup.second_user_addr.clone();
//     let borrower_addr = lending_setup.third_user_addr.clone();

//     lending_setup.b_mock.set_block_round(3);
//     lending_setup
//         .b_mock
//         .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
//             let borrow_index = sc.borrow_index().get();
//             assert_eq!(borrow_index, 1000000000);
//         })
//         .assert_ok();

//     // Supply/Deposit
//     lending_setup.b_mock.set_esdt_balance(
//         &supplier_addr,
//         USDC_TOKEN_ID,
//         &rust_biguint!(2_000 * BP),
//     );

//     lending_setup
//         .b_mock
//         .execute_esdt_transfer(
//             &supplier_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             USDC_TOKEN_ID,
//             0,
//             &rust_biguint!(2_000 * BP),
//             |sc| {
//                 sc.deposit_asset(managed_address!(&supplier_addr));
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &supplier_addr,
//         LEND_USDC_TOKEN_ID,
//         1,
//         &rust_biguint!(2_000 * BP),
//         Option::<&Empty>::None,
//     );

//     lending_setup
//         .b_mock
//         .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
//             let borrow_index = sc.borrow_index().get();
//             assert_eq!(borrow_index, 1000000000);
//         })
//         .assert_ok();

//     lending_setup.b_mock.set_block_round(5);

//     // Borrow
//     lending_setup.b_mock.set_nft_balance(
//         &borrower_addr,
//         LEND_EGLD,
//         1,
//         &rust_biguint!(4 * BP),
//         &Empty,
//     );

//     lending_setup
//         .b_mock
//         .execute_esdt_transfer(
//             &borrower_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             LEND_EGLD,
//             1,
//             &rust_biguint!(4 * BP),
//             |sc| {
//                 sc.borrow(
//                     managed_address!(&borrower_addr),
//                     managed_biguint!(500_000_000),
//                 );
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &borrower_addr,
//         BORROW_USDC_TOKEN_ID,
//         1,
//         &rust_biguint!(400 * BP),
//         Some(&Vec::<u8>::new()),
//     );

//     lending_setup.b_mock.check_nft_balance(
//         &borrower_addr,
//         USDC_TOKEN_ID,
//         0,
//         &rust_biguint!(400 * BP),
//         Some(&Vec::<u8>::new()),
//     );

//     lending_setup
//         .b_mock
//         .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
//             let borrow_index = sc.borrow_index().get();
//             assert_eq!(borrow_index, 1000000000);
//         })
//         .assert_ok();

//     lending_setup.b_mock.set_block_round(7);

//     // Supply/Deposit
//     lending_setup.b_mock.set_esdt_balance(
//         &supplier2_addr,
//         USDC_TOKEN_ID,
//         &rust_biguint!(2_000 * BP),
//     );

//     lending_setup
//         .b_mock
//         .execute_esdt_transfer(
//             &supplier2_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             USDC_TOKEN_ID,
//             0,
//             &rust_biguint!(2_000 * BP),
//             |sc| {
//                 sc.deposit_asset(managed_address!(&supplier2_addr));
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &supplier2_addr,
//         LEND_USDC_TOKEN_ID,
//         2,
//         &rust_biguint!(2_000 * BP),
//         Option::<&Empty>::None,
//     );

//     lending_setup
//         .b_mock
//         .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
//             let borrow_index = sc.borrow_index().get();
//             assert_eq!(borrow_index, 1020000000);
//         })
//         .assert_ok();

//     lending_setup.b_mock.set_block_round(9);

//     // Repay
//     lending_setup
//         .b_mock
//         .set_esdt_balance(&borrower_addr, USDC_TOKEN_ID, &rust_biguint!(412 * BP));

//     let mut payments = Vec::with_capacity(2);

//     payments.push(TxInputESDT {
//         token_identifier: BORROW_USDC_TOKEN_ID.to_vec(),
//         nonce: 1,
//         value: rust_biguint!(400 * BP),
//     });

//     payments.push(TxInputESDT {
//         token_identifier: USDC_TOKEN_ID.to_vec(),
//         nonce: 0,
//         value: rust_biguint!(412 * BP),
//     });

//     lending_setup
//         .b_mock
//         .execute_esdt_multi_transfer(
//             &borrower_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             &payments,
//             |sc| {
//                 sc.repay(managed_address!(&borrower_addr));
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &borrower_addr,
//         BORROW_USDC_TOKEN_ID,
//         1,
//         &rust_biguint!(0),
//         Some(&Vec::<u8>::new()),
//     );

//     lending_setup
//         .b_mock
//         .check_esdt_balance(&borrower_addr, USDC_TOKEN_ID, &rust_biguint!(0));

//     lending_setup.b_mock.check_nft_balance(
//         &borrower_addr,
//         LEND_EGLD,
//         1,
//         &rust_biguint!(4 * BP),
//         Some(&Vec::<u8>::new()),
//     );

//     lending_setup
//         .b_mock
//         .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
//             let borrow_index = sc.borrow_index().get();
//             assert_eq!(borrow_index, 1030000000);
//         })
//         .assert_ok();

//     lending_setup.b_mock.set_block_round(11);

//     // Withdraw - Supplier1
//     lending_setup
//         .b_mock
//         .execute_esdt_transfer(
//             &supplier_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             LEND_USDC_TOKEN_ID,
//             1,
//             &rust_biguint!(2_000 * BP),
//             |sc| {
//                 sc.withdraw(managed_address!(&supplier_addr));
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &supplier_addr,
//         LEND_USDC_TOKEN_ID,
//         1,
//         &rust_biguint!(0),
//         Option::<&Empty>::None,
//     );

//     lending_setup.b_mock.check_nft_balance(
//         &supplier_addr,
//         USDC_TOKEN_ID,
//         0,
//         &rust_biguint!(2_010 * BP),
//         Some(&Empty),
//     );

//     lending_setup
//         .b_mock
//         .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
//             let borrow_index = sc.borrow_index().get();
//             assert_eq!(borrow_index, 1030000000);
//         })
//         .assert_ok();

//     lending_setup.b_mock.set_block_round(13);

//     // Withdraw - Supplier 2
//     lending_setup
//         .b_mock
//         .execute_esdt_transfer(
//             &supplier2_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             LEND_USDC_TOKEN_ID,
//             2,
//             &rust_biguint!(2_000 * BP),
//             |sc| {
//                 sc.withdraw(managed_address!(&supplier2_addr));
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &supplier2_addr,
//         LEND_USDC_TOKEN_ID,
//         1,
//         &rust_biguint!(0),
//         Option::<&Empty>::None,
//     );

//     lending_setup.b_mock.check_nft_balance(
//         &supplier2_addr,
//         USDC_TOKEN_ID,
//         0,
//         &rust_biguint!(2_002 * BP),
//         Some(&Empty),
//     );

//     lending_setup
//         .b_mock
//         .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
//             let borrow_index = sc.borrow_index().get();
//             assert_eq!(borrow_index, 1030000000);
//         })
//         .assert_ok();
// }

// #[test]
// fn scenario6() {
//     let mut lending_setup = LendingSetup::deploy_lending(
//         lending_pool::contract_obj,
//         liquidity_pool::contract_obj,
//         aggregator_mock::contract_obj,
//     );
//     let alice_addr = lending_setup.first_user_addr.clone();
//     let bob_addr = lending_setup.second_user_addr.clone();
//     let charlie_addr = lending_setup.third_user_addr.clone();
//     let dave_addr = lending_setup.fourth_user_addr.clone();
//     let eve_addr = lending_setup.fifth_user_addr.clone();

//     lending_setup.b_mock.set_block_round(3);
//     lending_setup
//         .b_mock
//         .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
//             let borrow_index = sc.borrow_index().get();
//             assert_eq!(borrow_index, 1000000000);
//         })
//         .assert_ok();

//     // Supply/Deposit - Alice
//     lending_setup
//         .b_mock
//         .set_esdt_balance(&alice_addr, USDC_TOKEN_ID, &rust_biguint!(40_000 * BP));

//     lending_setup
//         .b_mock
//         .execute_esdt_transfer(
//             &alice_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             USDC_TOKEN_ID,
//             0,
//             &rust_biguint!(40_000 * BP),
//             |sc| {
//                 sc.deposit_asset(managed_address!(&alice_addr));
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &alice_addr,
//         LEND_USDC_TOKEN_ID,
//         1,
//         &rust_biguint!(40_000 * BP),
//         Option::<&Empty>::None,
//     );

//     lending_setup
//         .b_mock
//         .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
//             let borrow_index = sc.borrow_index().get();
//             assert_eq!(borrow_index, 1000000000);
//         })
//         .assert_ok();

//     lending_setup.b_mock.set_block_round(5);

//     // Supply/Deposit - Bob
//     lending_setup
//         .b_mock
//         .set_esdt_balance(&bob_addr, USDC_TOKEN_ID, &rust_biguint!(20_000 * BP));

//     lending_setup
//         .b_mock
//         .execute_esdt_transfer(
//             &bob_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             USDC_TOKEN_ID,
//             0,
//             &rust_biguint!(20_000 * BP),
//             |sc| {
//                 sc.deposit_asset(managed_address!(&bob_addr));
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &bob_addr,
//         LEND_USDC_TOKEN_ID,
//         2,
//         &rust_biguint!(20_000 * BP),
//         Option::<&Empty>::None,
//     );

//     lending_setup
//         .b_mock
//         .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
//             let borrow_index = sc.borrow_index().get();
//             assert_eq!(borrow_index, 1000000000);
//         })
//         .assert_ok();

//     lending_setup.b_mock.set_block_round(7);

//     // Borrow - Charlie
//     lending_setup.b_mock.set_nft_balance(
//         &charlie_addr,
//         LEND_EGLD,
//         3,
//         &rust_biguint!(150 * BP),
//         &Empty,
//     );

//     lending_setup
//         .b_mock
//         .execute_esdt_transfer(
//             &charlie_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             LEND_EGLD,
//             3,
//             &rust_biguint!(150 * BP),
//             |sc| {
//                 sc.borrow(
//                     managed_address!(&charlie_addr),
//                     managed_biguint!(500_000_000),
//                 );
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &charlie_addr,
//         BORROW_USDC_TOKEN_ID,
//         3,
//         &rust_biguint!(15_000 * BP),
//         Some(&Vec::<u8>::new()),
//     );

//     lending_setup.b_mock.check_nft_balance(
//         &charlie_addr,
//         USDC_TOKEN_ID,
//         0,
//         &rust_biguint!(15_000 * BP),
//         Some(&Vec::<u8>::new()),
//     );

//     lending_setup
//         .b_mock
//         .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
//             let borrow_index = sc.borrow_index().get();
//             assert_eq!(borrow_index, 1000000000);
//         })
//         .assert_ok();

//     lending_setup.b_mock.set_block_round(9);

//     // Borrow - Dave
//     lending_setup
//         .b_mock
//         .set_nft_balance(&dave_addr, LEND_EGLD, 4, &rust_biguint!(60 * BP), &Empty);

//     lending_setup
//         .b_mock
//         .execute_esdt_transfer(
//             &dave_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             LEND_EGLD,
//             4,
//             &rust_biguint!(60 * BP),
//             |sc| {
//                 sc.borrow(managed_address!(&dave_addr), managed_biguint!(500_000_000));
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &dave_addr,
//         BORROW_USDC_TOKEN_ID,
//         4,
//         &rust_biguint!(6_000 * BP),
//         Some(&Vec::<u8>::new()),
//     );

//     lending_setup.b_mock.check_nft_balance(
//         &dave_addr,
//         USDC_TOKEN_ID,
//         0,
//         &rust_biguint!(6_000 * BP),
//         Some(&Vec::<u8>::new()),
//     );

//     lending_setup
//         .b_mock
//         .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
//             let borrow_index = sc.borrow_index().get();
//             assert_eq!(borrow_index, 1025000000);
//         })
//         .assert_ok();

//     lending_setup.b_mock.set_block_round(11);

//     // Borrow - Eve
//     lending_setup
//         .b_mock
//         .set_nft_balance(&eve_addr, LEND_EGLD, 5, &rust_biguint!(120 * BP), &Empty);

//     lending_setup
//         .b_mock
//         .execute_esdt_transfer(
//             &eve_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             LEND_EGLD,
//             5,
//             &rust_biguint!(120 * BP),
//             |sc| {
//                 sc.borrow(managed_address!(&eve_addr), managed_biguint!(500_000_000));
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &eve_addr,
//         BORROW_USDC_TOKEN_ID,
//         5,
//         &rust_biguint!(12_000 * BP),
//         Some(&Vec::<u8>::new()),
//     );

//     lending_setup.b_mock.check_nft_balance(
//         &eve_addr,
//         USDC_TOKEN_ID,
//         0,
//         &rust_biguint!(12_000 * BP),
//         Some(&Vec::<u8>::new()),
//     );

//     lending_setup
//         .b_mock
//         .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
//             let borrow_index = sc.borrow_index().get();
//             assert_eq!(borrow_index, 1060000000);
//         })
//         .assert_ok();

//     lending_setup.b_mock.set_block_round(13);

//     // Repay - Eve
//     lending_setup
//         .b_mock
//         .set_esdt_balance(&eve_addr, USDC_TOKEN_ID, &rust_biguint!(12_660 * BP));

//     let mut payments = Vec::with_capacity(2);

//     payments.push(TxInputESDT {
//         token_identifier: BORROW_USDC_TOKEN_ID.to_vec(),
//         nonce: 3,
//         value: rust_biguint!(12_000 * BP),
//     });

//     payments.push(TxInputESDT {
//         token_identifier: USDC_TOKEN_ID.to_vec(),
//         nonce: 0,
//         value: rust_biguint!(12_660 * BP),
//     });

//     lending_setup
//         .b_mock
//         .execute_esdt_multi_transfer(
//             &eve_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             &payments,
//             |sc| {
//                 sc.repay(managed_address!(&eve_addr));
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &eve_addr,
//         BORROW_USDC_TOKEN_ID,
//         3,
//         &rust_biguint!(0),
//         Some(&Vec::<u8>::new()),
//     );

//     lending_setup
//         .b_mock
//         .check_esdt_balance(&eve_addr, USDC_TOKEN_ID, &rust_biguint!(0));

//     lending_setup.b_mock.check_nft_balance(
//         &eve_addr,
//         LEND_EGLD,
//         5,
//         &rust_biguint!(120 * BP),
//         Some(&Vec::<u8>::new()),
//     );

//     lending_setup
//         .b_mock
//         .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
//             let borrow_index = sc.borrow_index().get();
//             assert_eq!(borrow_index, 1115000000);
//         })
//         .assert_ok();

//     lending_setup.b_mock.set_block_round(15);

//     // Repay - Dave (1)
//     lending_setup.b_mock.set_esdt_balance(
//         &dave_addr,
//         USDC_TOKEN_ID,
//         &rust_biguint!(3_373_857_564_000),
//     );

//     let mut payments = Vec::with_capacity(2);

//     payments.push(TxInputESDT {
//         token_identifier: BORROW_USDC_TOKEN_ID.to_vec(),
//         nonce: 2,
//         value: rust_biguint!(3_000 * BP),
//     });

//     payments.push(TxInputESDT {
//         token_identifier: USDC_TOKEN_ID.to_vec(),
//         nonce: 0,
//         value: rust_biguint!(3_373_857_564_000),
//     });

//     lending_setup
//         .b_mock
//         .execute_esdt_multi_transfer(
//             &dave_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             &payments,
//             |sc| {
//                 sc.repay(managed_address!(&dave_addr));
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &dave_addr,
//         BORROW_USDC_TOKEN_ID,
//         2,
//         &rust_biguint!(3_000 * BP),
//         Some(&Vec::<u8>::new()),
//     );

//     lending_setup
//         .b_mock
//         .check_esdt_balance(&dave_addr, USDC_TOKEN_ID, &rust_biguint!(0));

//     lending_setup.b_mock.check_nft_balance(
//         &dave_addr,
//         LEND_EGLD,
//         4,
//         &rust_biguint!(30 * BP),
//         Some(&Vec::<u8>::new()),
//     );

//     lending_setup
//         .b_mock
//         .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
//             let borrow_index = sc.borrow_index().get();
//             assert_eq!(borrow_index, 1149619188);
//         })
//         .assert_ok();

//     lending_setup.b_mock.set_block_round(17);

//     // Repay - Dave (2)
//     lending_setup.b_mock.set_esdt_balance(
//         &dave_addr,
//         USDC_TOKEN_ID,
//         &rust_biguint!(3_462_333_042_000),
//     );

//     let mut payments = Vec::with_capacity(2);

//     payments.push(TxInputESDT {
//         token_identifier: BORROW_USDC_TOKEN_ID.to_vec(),
//         nonce: 2,
//         value: rust_biguint!(3_000 * BP),
//     });

//     payments.push(TxInputESDT {
//         token_identifier: USDC_TOKEN_ID.to_vec(),
//         nonce: 0,
//         value: rust_biguint!(3_462_333_042_000),
//     });

//     lending_setup
//         .b_mock
//         .execute_esdt_multi_transfer(
//             &dave_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             &payments,
//             |sc| {
//                 sc.repay(managed_address!(&dave_addr));
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &dave_addr,
//         BORROW_USDC_TOKEN_ID,
//         2,
//         &rust_biguint!(0),
//         Some(&Vec::<u8>::new()),
//     );

//     lending_setup
//         .b_mock
//         .check_esdt_balance(&dave_addr, USDC_TOKEN_ID, &rust_biguint!(0));

//     lending_setup.b_mock.check_nft_balance(
//         &dave_addr,
//         LEND_EGLD,
//         4,
//         &rust_biguint!(60 * BP),
//         Some(&Vec::<u8>::new()),
//     );

//     lending_setup
//         .b_mock
//         .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
//             let borrow_index = sc.borrow_index().get();
//             assert_eq!(borrow_index, 1179111014);
//         })
//         .assert_ok();

//     lending_setup.b_mock.set_block_round(19);

//     // Repay - Charlie (1)
//     lending_setup.b_mock.set_esdt_balance(
//         &charlie_addr,
//         USDC_TOKEN_ID,
//         &rust_biguint!(3_610_508_304_000),
//     );

//     let mut payments = Vec::with_capacity(2);

//     payments.push(TxInputESDT {
//         token_identifier: BORROW_USDC_TOKEN_ID.to_vec(),
//         nonce: 1,
//         value: rust_biguint!(3_000 * BP),
//     });

//     payments.push(TxInputESDT {
//         token_identifier: USDC_TOKEN_ID.to_vec(),
//         nonce: 0,
//         value: rust_biguint!(3_610_508_304_000),
//     });

//     lending_setup
//         .b_mock
//         .execute_esdt_multi_transfer(
//             &charlie_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             &payments,
//             |sc| {
//                 sc.repay(managed_address!(&charlie_addr));
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &charlie_addr,
//         BORROW_USDC_TOKEN_ID,
//         1,
//         &rust_biguint!(12_000 * BP),
//         Some(&Vec::<u8>::new()),
//     );

//     lending_setup
//         .b_mock
//         .check_esdt_balance(&charlie_addr, USDC_TOKEN_ID, &rust_biguint!(0));

//     lending_setup.b_mock.check_nft_balance(
//         &charlie_addr,
//         LEND_EGLD,
//         3,
//         &rust_biguint!(30 * BP),
//         Some(&Vec::<u8>::new()),
//     );

//     lending_setup
//         .b_mock
//         .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
//             let borrow_index = sc.borrow_index().get();
//             assert_eq!(borrow_index, 1203502768);
//         })
//         .assert_ok();

//     lending_setup.b_mock.set_block_round(21);

//     // Repay - Charlie (2)
//     lending_setup.b_mock.set_esdt_balance(
//         &charlie_addr,
//         USDC_TOKEN_ID,
//         &rust_biguint!(14_673_892_248_000),
//     );

//     let mut payments = Vec::with_capacity(2);

//     payments.push(TxInputESDT {
//         token_identifier: BORROW_USDC_TOKEN_ID.to_vec(),
//         nonce: 1,
//         value: rust_biguint!(12_000 * BP),
//     });

//     payments.push(TxInputESDT {
//         token_identifier: USDC_TOKEN_ID.to_vec(),
//         nonce: 0,
//         value: rust_biguint!(14_673_892_248_000),
//     });

//     lending_setup
//         .b_mock
//         .execute_esdt_multi_transfer(
//             &charlie_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             &payments,
//             |sc| {
//                 sc.repay(managed_address!(&charlie_addr));
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &charlie_addr,
//         BORROW_USDC_TOKEN_ID,
//         1,
//         &rust_biguint!(0),
//         Some(&Vec::<u8>::new()),
//     );

//     lending_setup
//         .b_mock
//         .check_esdt_balance(&charlie_addr, USDC_TOKEN_ID, &rust_biguint!(0));

//     lending_setup.b_mock.check_nft_balance(
//         &charlie_addr,
//         LEND_EGLD,
//         3,
//         &rust_biguint!(150 * BP),
//         Some(&Vec::<u8>::new()),
//     );

//     lending_setup
//         .b_mock
//         .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
//             let borrow_index = sc.borrow_index().get();
//             assert_eq!(borrow_index, 1222824354);
//         })
//         .assert_ok();

//     lending_setup.b_mock.set_block_round(23);

//     // Withdraw - Supplier1
//     lending_setup
//         .b_mock
//         .execute_esdt_transfer(
//             &bob_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             LEND_USDC_TOKEN_ID,
//             2,
//             &rust_biguint!(20_000 * BP),
//             |sc| {
//                 sc.withdraw(managed_address!(&bob_addr));
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &bob_addr,
//         LEND_USDC_TOKEN_ID,
//         2,
//         &rust_biguint!(0),
//         Option::<&Empty>::None,
//     );

//     lending_setup.b_mock.check_nft_balance(
//         &bob_addr,
//         USDC_TOKEN_ID,
//         0,
//         &rust_biguint!(21_582_307_460_000),
//         Some(&Empty),
//     );

//     lending_setup
//         .b_mock
//         .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
//             let borrow_index = sc.borrow_index().get();
//             assert_eq!(borrow_index, 1222824354);
//         })
//         .assert_ok();

//     lending_setup.b_mock.set_block_round(25);

//     // Withdraw - Supplier 2
//     lending_setup
//         .b_mock
//         .execute_esdt_transfer(
//             &alice_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             LEND_USDC_TOKEN_ID,
//             1,
//             &rust_biguint!(40_000 * BP),
//             |sc| {
//                 sc.withdraw(managed_address!(&alice_addr));
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &alice_addr,
//         LEND_USDC_TOKEN_ID,
//         1,
//         &rust_biguint!(0),
//         Option::<&Empty>::None,
//     );

//     lending_setup.b_mock.check_nft_balance(
//         &alice_addr,
//         USDC_TOKEN_ID,
//         0,
//         &rust_biguint!(43_164_614_920_000),
//         Some(&Empty),
//     );

//     lending_setup
//         .b_mock
//         .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
//             let borrow_index = sc.borrow_index().get();
//             assert_eq!(borrow_index, 1222824354);
//         })
//         .assert_ok();
// }

// #[test]
// fn scenario7() {
//     let mut lending_setup = LendingSetup::deploy_lending(
//         lending_pool::contract_obj,
//         liquidity_pool::contract_obj,
//         aggregator_mock::contract_obj,
//     );
//     let supplier_addr = lending_setup.first_user_addr.clone();
//     let supplier2_addr = lending_setup.second_user_addr.clone();
//     let borrower_addr = lending_setup.third_user_addr.clone();
//     let borrower2_addr = lending_setup.fourth_user_addr.clone();

//     // Supply/Deposit
//     lending_setup.b_mock.set_esdt_balance(
//         &supplier_addr,
//         USDC_TOKEN_ID,
//         &rust_biguint!(4_000 * BP),
//     );

//     lending_setup
//         .b_mock
//         .execute_esdt_transfer(
//             &supplier_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             USDC_TOKEN_ID,
//             0,
//             &rust_biguint!(4_000 * BP),
//             |sc| {
//                 sc.deposit_asset(managed_address!(&supplier_addr));
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &supplier_addr,
//         LEND_USDC_TOKEN_ID,
//         1,
//         &rust_biguint!(4_000 * BP),
//         Option::<&Empty>::None,
//     );

//     lending_setup
//         .b_mock
//         .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
//             let borrow_index = sc.borrow_index().get();
//             assert_eq!(borrow_index, 1000000000);
//         })
//         .assert_ok();

//     lending_setup.b_mock.set_block_round(5);

//     // Supply/Deposit
//     lending_setup.b_mock.set_esdt_balance(
//         &supplier2_addr,
//         USDC_TOKEN_ID,
//         &rust_biguint!(4_000 * BP),
//     );

//     lending_setup
//         .b_mock
//         .execute_esdt_transfer(
//             &supplier2_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             USDC_TOKEN_ID,
//             0,
//             &rust_biguint!(4_000 * BP),
//             |sc| {
//                 sc.deposit_asset(managed_address!(&supplier2_addr));
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &supplier2_addr,
//         LEND_USDC_TOKEN_ID,
//         2,
//         &rust_biguint!(4_000 * BP),
//         Option::<&Empty>::None,
//     );

//     lending_setup
//         .b_mock
//         .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
//             let borrow_index = sc.borrow_index().get();
//             assert_eq!(borrow_index, 1000000000);
//         })
//         .assert_ok();

//     lending_setup.b_mock.set_block_round(7);

//     // Borrow
//     lending_setup.b_mock.set_nft_balance(
//         &borrower_addr,
//         LEND_EGLD,
//         3,
//         &rust_biguint!(12 * BP),
//         &Empty,
//     );

//     lending_setup
//         .b_mock
//         .execute_esdt_transfer(
//             &borrower_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             LEND_EGLD,
//             3,
//             &rust_biguint!(12 * BP),
//             |sc| {
//                 sc.borrow(
//                     managed_address!(&borrower_addr),
//                     managed_biguint!(500_000_000),
//                 );
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &borrower_addr,
//         BORROW_USDC_TOKEN_ID,
//         1,
//         &rust_biguint!(1_200 * BP),
//         Some(&Vec::<u8>::new()),
//     );

//     lending_setup.b_mock.check_nft_balance(
//         &borrower_addr,
//         USDC_TOKEN_ID,
//         0,
//         &rust_biguint!(1_200 * BP),
//         Some(&Vec::<u8>::new()),
//     );

//     lending_setup
//         .b_mock
//         .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
//             let borrow_index = sc.borrow_index().get();
//             assert_eq!(borrow_index, 1000000000);
//         })
//         .assert_ok();

//     lending_setup.b_mock.set_block_round(8);

//     // Borrow
//     lending_setup.b_mock.set_nft_balance(
//         &borrower2_addr,
//         LEND_EGLD,
//         4,
//         &rust_biguint!(24 * BP),
//         &Empty,
//     );

//     lending_setup
//         .b_mock
//         .execute_esdt_transfer(
//             &borrower2_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             LEND_EGLD,
//             4,
//             &rust_biguint!(24 * BP),
//             |sc| {
//                 sc.borrow(
//                     managed_address!(&borrower2_addr),
//                     managed_biguint!(500_000_000),
//                 );
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &borrower2_addr,
//         BORROW_USDC_TOKEN_ID,
//         2,
//         &rust_biguint!(2_400 * BP),
//         Some(&Vec::<u8>::new()),
//     );

//     lending_setup.b_mock.check_nft_balance(
//         &borrower2_addr,
//         USDC_TOKEN_ID,
//         0,
//         &rust_biguint!(2_400 * BP),
//         Some(&Vec::<u8>::new()),
//     );

//     lending_setup
//         .b_mock
//         .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
//             let borrow_index = sc.borrow_index().get();
//             assert_eq!(borrow_index, 1007500000);
//         })
//         .assert_ok();

//     // Repay - Charlie (1000 USD)
//     lending_setup.b_mock.set_esdt_balance(
//         &borrower_addr,
//         USDC_TOKEN_ID,
//         &rust_biguint!(1_209 * BP),
//     );

//     let mut payments = Vec::with_capacity(2);

//     payments.push(TxInputESDT {
//         token_identifier: BORROW_USDC_TOKEN_ID.to_vec(),
//         nonce: 1,
//         value: rust_biguint!(1_200 * BP),
//     });

//     payments.push(TxInputESDT {
//         token_identifier: USDC_TOKEN_ID.to_vec(),
//         nonce: 0,
//         value: rust_biguint!(1_209 * BP),
//     });

//     lending_setup
//         .b_mock
//         .execute_esdt_multi_transfer(
//             &borrower_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             &payments,
//             |sc| {
//                 sc.repay(managed_address!(&borrower_addr));
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &borrower_addr,
//         BORROW_USDC_TOKEN_ID,
//         1,
//         &rust_biguint!(0),
//         Some(&Vec::<u8>::new()),
//     );

//     lending_setup
//         .b_mock
//         .check_esdt_balance(&borrower_addr, USDC_TOKEN_ID, &rust_biguint!(0));

//     lending_setup.b_mock.check_nft_balance(
//         &borrower_addr,
//         LEND_EGLD,
//         3,
//         &rust_biguint!(12 * BP),
//         Some(&Vec::<u8>::new()),
//     );

//     lending_setup
//         .b_mock
//         .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
//             let borrow_index = sc.borrow_index().get();
//             assert_eq!(borrow_index, 1007500000);
//         })
//         .assert_ok();

//     lending_setup.b_mock.set_block_round(11);

//     // Repay - Dave (2400 USD)
//     lending_setup.b_mock.set_esdt_balance(
//         &borrower2_addr,
//         USDC_TOKEN_ID,
//         &rust_biguint!(2507878629600),
//     );

//     let mut payments = Vec::with_capacity(2);

//     payments.push(TxInputESDT {
//         token_identifier: BORROW_USDC_TOKEN_ID.to_vec(),
//         nonce: 2,
//         value: rust_biguint!(2_400 * BP),
//     });

//     payments.push(TxInputESDT {
//         token_identifier: USDC_TOKEN_ID.to_vec(),
//         nonce: 0,
//         value: rust_biguint!(2507878629600),
//     });

//     lending_setup
//         .b_mock
//         .execute_esdt_multi_transfer(
//             &borrower2_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             &payments,
//             |sc| {
//                 sc.repay(managed_address!(&borrower2_addr));
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &borrower2_addr,
//         BORROW_USDC_TOKEN_ID,
//         2,
//         &rust_biguint!(0),
//         Some(&Vec::<u8>::new()),
//     );

//     lending_setup
//         .b_mock
//         .check_esdt_balance(&borrower2_addr, USDC_TOKEN_ID, &rust_biguint!(0));

//     lending_setup.b_mock.check_nft_balance(
//         &borrower2_addr,
//         LEND_EGLD,
//         4,
//         &rust_biguint!(24 * BP),
//         Some(&Vec::<u8>::new()),
//     );

//     lending_setup
//         .b_mock
//         .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
//             let borrow_index = sc.borrow_index().get();
//             assert_eq!(borrow_index, 1052449429);
//         })
//         .assert_ok();

//     lending_setup.b_mock.set_block_round(13);

//     // Withdraw - Supplier1
//     lending_setup
//         .b_mock
//         .execute_esdt_transfer(
//             &supplier_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             LEND_USDC_TOKEN_ID,
//             1,
//             &rust_biguint!(4_000 * BP),
//             |sc| {
//                 sc.withdraw(managed_address!(&supplier_addr));
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &supplier_addr,
//         LEND_USDC_TOKEN_ID,
//         1,
//         &rust_biguint!(0),
//         Option::<&Empty>::None,
//     );

//     lending_setup.b_mock.check_nft_balance(
//         &supplier_addr,
//         USDC_TOKEN_ID,
//         0,
//         &rust_biguint!(4058378700000),
//         Some(&Empty),
//     );

//     lending_setup
//         .b_mock
//         .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
//             let borrow_index = sc.borrow_index().get();
//             assert_eq!(borrow_index, 1052449429);
//         })
//         .assert_ok();

//     lending_setup.b_mock.set_block_round(15);

//     // Withdraw - Supplier 2
//     lending_setup
//         .b_mock
//         .execute_esdt_transfer(
//             &supplier2_addr,
//             &lending_setup.liquidity_pool_usdc_wrapper,
//             LEND_USDC_TOKEN_ID,
//             2,
//             &rust_biguint!(4_000 * BP),
//             |sc| {
//                 sc.withdraw(managed_address!(&supplier2_addr));
//             },
//         )
//         .assert_ok();

//     lending_setup.b_mock.check_nft_balance(
//         &supplier2_addr,
//         LEND_USDC_TOKEN_ID,
//         2,
//         &rust_biguint!(0),
//         Option::<&Empty>::None,
//     );

//     lending_setup.b_mock.check_nft_balance(
//         &supplier2_addr,
//         USDC_TOKEN_ID,
//         0,
//         &rust_biguint!(4058378700000),
//         Some(&Empty),
//     );

//     lending_setup
//         .b_mock
//         .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
//             let borrow_index = sc.borrow_index().get();
//             assert_eq!(borrow_index, 1052449429);
//         })
//         .assert_ok();
// }
