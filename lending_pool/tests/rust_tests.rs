use aggregator_mock::PriceAggregatorMock;
use constants::*;
use elrond_wasm::elrond_codec::Empty;
use elrond_wasm_debug::{
    managed_address, managed_biguint, managed_buffer, managed_token_id, rust_biguint,
    tx_mock::TxInputESDT,
};
use lending_pool::BP;
use lending_pool_interaction::LendingSetup;
use liquidity_pool::{liquidity::LiquidityModule, storage::StorageModule};

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
fn deposit_test() {
    let mut lending_setup = LendingSetup::deploy_lending(
        lending_pool::contract_obj,
        liquidity_pool::contract_obj,
        aggregator_mock::contract_obj,
    );
    let user_addr = lending_setup.first_user_addr.clone();

    lending_setup
        .b_mock
        .set_esdt_balance(&user_addr, USDC_TOKEN_ID, &rust_biguint!(1_000));

    lending_setup
        .b_mock
        .execute_esdt_transfer(
            &user_addr,
            &lending_setup.liquidity_pool_usdc_wrapper,
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
        Option::<&Empty>::None,
    );

    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let actual_lend_token = sc.lend_token().get();
            let expected_lend_token = managed_token_id!(LEND_USDC_TOKEN_ID);
            assert_eq!(actual_lend_token, expected_lend_token, "Wrong token");
        })
        .assert_ok();
}

#[test]
fn withdraw_test() {
    let mut lending_setup = LendingSetup::deploy_lending(
        lending_pool::contract_obj,
        liquidity_pool::contract_obj,
        aggregator_mock::contract_obj,
    );
    let user_addr = lending_setup.first_user_addr.clone();

    lending_setup
        .b_mock
        .set_esdt_balance(&user_addr, USDC_TOKEN_ID, &rust_biguint!(1_000));

    lending_setup
        .b_mock
        .execute_esdt_transfer(
            &user_addr,
            &lending_setup.liquidity_pool_usdc_wrapper,
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
        Option::<&Empty>::None,
    );

    lending_setup
        .b_mock
        .execute_query(&lending_setup.liquidity_pool_usdc_wrapper, |sc| {
            let actual_lend_token = sc.lend_token().get();
            let expected_lend_token = managed_token_id!(LEND_USDC_TOKEN_ID);
            assert_eq!(actual_lend_token, expected_lend_token, "Wrong token");
        })
        .assert_ok();

    lending_setup
        .b_mock
        .execute_esdt_transfer(
            &user_addr,
            &lending_setup.liquidity_pool_usdc_wrapper,
            LEND_USDC_TOKEN_ID,
            1,
            &rust_biguint!(1_000),
            |sc| {
                sc.withdraw(managed_address!(&user_addr));
            },
        )
        .assert_ok();

    lending_setup.b_mock.check_nft_balance(
        &user_addr,
        LEND_USDC_TOKEN_ID,
        1,
        &rust_biguint!(0),
        Option::<&Empty>::None,
    );

    lending_setup.b_mock.check_nft_balance(
        &user_addr,
        USDC_TOKEN_ID,
        0,
        &rust_biguint!(1_000),
        Some(&Empty),
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

    lending_setup
        .b_mock
        .set_esdt_balance(&user_addr, USDC_TOKEN_ID, &rust_biguint!(1_000));

    lending_setup
        .b_mock
        .execute_esdt_transfer(
            &user_addr,
            &lending_setup.liquidity_pool_usdc_wrapper,
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
        Option::<&Empty>::None,
    );

    lending_setup
        .b_mock
        .set_nft_balance(&user_addr, LEND_EGLD, 1, &rust_biguint!(10), &Empty);

    lending_setup
        .b_mock
        .execute_esdt_transfer(
            &user_addr,
            &lending_setup.liquidity_pool_usdc_wrapper,
            LEND_EGLD,
            1,
            &rust_biguint!(10),
            |sc| {
                sc.borrow(managed_address!(&user_addr), managed_biguint!(500_000_000));
            },
        )
        .assert_ok();

    lending_setup.b_mock.check_nft_balance(
        &user_addr,
        BORROW_USDC_TOKEN_ID,
        1,
        &rust_biguint!(1_000),
        Some(&Vec::<u8>::new()),
    );

    lending_setup.b_mock.check_nft_balance(
        &user_addr,
        USDC_TOKEN_ID,
        0,
        &rust_biguint!(1_000),
        Some(&Vec::<u8>::new()),
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

    lending_setup
        .b_mock
        .set_esdt_balance(&user_addr, USDC_TOKEN_ID, &rust_biguint!(2_000 * BP));

    lending_setup
        .b_mock
        .execute_esdt_transfer(
            &user_addr,
            &lending_setup.liquidity_pool_usdc_wrapper,
            USDC_TOKEN_ID,
            0,
            &rust_biguint!(2_000 * BP),
            |sc| {
                sc.deposit_asset(managed_address!(&user_addr));
            },
        )
        .assert_ok();

    lending_setup.b_mock.check_nft_balance(
        &user_addr,
        LEND_USDC_TOKEN_ID,
        1,
        &rust_biguint!(2_000 * BP),
        Option::<&Empty>::None,
    );

    lending_setup.b_mock.set_block_round(10);

    lending_setup
        .b_mock
        .set_nft_balance(&user_addr, LEND_EGLD, 1, &rust_biguint!(10 * BP), &Empty);

    lending_setup
        .b_mock
        .execute_esdt_transfer(
            &user_addr,
            &lending_setup.liquidity_pool_usdc_wrapper,
            LEND_EGLD,
            1,
            &rust_biguint!(10 * BP),
            |sc| {
                sc.borrow(managed_address!(&user_addr), managed_biguint!(500_000_000));
            },
        )
        .assert_ok();

    lending_setup.b_mock.check_nft_balance(
        &user_addr,
        BORROW_USDC_TOKEN_ID,
        1,
        &rust_biguint!(1_000 * BP),
        Some(&Vec::<u8>::new()),
    );

    lending_setup.b_mock.check_nft_balance(
        &user_addr,
        USDC_TOKEN_ID,
        0,
        &rust_biguint!(1_000 * BP),
        Some(&Vec::<u8>::new()),
    );

    lending_setup.b_mock.set_block_round(12);

    lending_setup
        .b_mock
        .set_esdt_balance(&user_addr, USDC_TOKEN_ID, &rust_biguint!(1_025 * BP));

    let mut payments = Vec::with_capacity(2);

    payments.push(TxInputESDT {
        token_identifier: BORROW_USDC_TOKEN_ID.to_vec(),
        nonce: 1,
        value: rust_biguint!(1_000 * BP),
    });

    payments.push(TxInputESDT {
        token_identifier: USDC_TOKEN_ID.to_vec(),
        nonce: 0,
        value: rust_biguint!(1_025 * BP),
    });

    lending_setup
        .b_mock
        .execute_esdt_multi_transfer(
            &user_addr,
            &lending_setup.liquidity_pool_usdc_wrapper,
            &payments,
            |sc| {
                sc.repay(managed_address!(&user_addr));
            },
        )
        .assert_ok();

    lending_setup.b_mock.check_nft_balance(
        &user_addr,
        BORROW_USDC_TOKEN_ID,
        1,
        &rust_biguint!(0),
        Some(&Vec::<u8>::new()),
    );

    lending_setup
        .b_mock
        .check_esdt_balance(&user_addr, USDC_TOKEN_ID, &rust_biguint!(0));

    lending_setup.b_mock.check_nft_balance(
        &user_addr,
        LEND_EGLD,
        1,
        &rust_biguint!(10 * BP),
        Some(&Vec::<u8>::new()),
    );
}

#[test]
fn partial_liquidate_without_bonus_test() {
    let mut lending_setup = LendingSetup::deploy_lending(
        lending_pool::contract_obj,
        liquidity_pool::contract_obj,
        aggregator_mock::contract_obj,
    );
    let borrower_addr = lending_setup.first_user_addr.clone();
    let liquidator_addr = lending_setup.second_user_addr.clone();

    lending_setup
        .b_mock
        .set_esdt_balance(&borrower_addr, USDC_TOKEN_ID, &rust_biguint!(200_000));

    // Deposit USDC
    lending_setup
        .b_mock
        .execute_esdt_transfer(
            &borrower_addr,
            &lending_setup.liquidity_pool_usdc_wrapper,
            USDC_TOKEN_ID,
            0,
            &rust_biguint!(200_000),
            |sc| {
                sc.deposit_asset(managed_address!(&borrower_addr));
            },
        )
        .assert_ok();

    // Check LUSDC
    lending_setup.b_mock.check_nft_balance(
        &borrower_addr,
        LEND_USDC_TOKEN_ID,
        1,
        &rust_biguint!(200_000),
        Option::<&Empty>::None,
    );

    lending_setup
        .b_mock
        .set_esdt_balance(&borrower_addr, EGLD_TOKEN_ID, &rust_biguint!(1_000));

    // Deposit EGLD
    lending_setup
        .b_mock
        .execute_esdt_transfer(
            &borrower_addr,
            &lending_setup.liquidity_pool_egld_wrapper,
            EGLD_TOKEN_ID,
            0,
            &rust_biguint!(1_000),
            |sc| {
                sc.deposit_asset(managed_address!(&borrower_addr));
            },
        )
        .assert_ok();

    // Check LEGLD
    lending_setup.b_mock.check_nft_balance(
        &borrower_addr,
        LEND_EGLD,
        1,
        &rust_biguint!(1_000),
        Option::<&Empty>::None,
    );

    // Borrow USDC
    lending_setup
        .b_mock
        .execute_esdt_transfer(
            &borrower_addr,
            &lending_setup.liquidity_pool_usdc_wrapper,
            LEND_EGLD,
            1,
            &rust_biguint!(1_000),
            |sc| {
                sc.borrow(
                    managed_address!(&borrower_addr),
                    managed_biguint!(500_000_000),
                );
            },
        )
        .assert_ok();

    // lending_setup.b_mock.dump_state_for_account_hex_attributes(&borrower_addr);

    // Check received BUSDC
    lending_setup.b_mock.check_nft_balance(
        &borrower_addr,
        BORROW_USDC_TOKEN_ID,
        1,
        &rust_biguint!(100_000),
        Some(&Vec::<u8>::new()),
    );

    // Check received USDC
    lending_setup.b_mock.check_nft_balance(
        &borrower_addr,
        USDC_TOKEN_ID,
        0,
        &rust_biguint!(100_000),
        Some(&Vec::<u8>::new()),
    );

    lending_setup
        .b_mock
        .execute_tx(
            &liquidator_addr,
            &lending_setup.price_aggregator_wrapper,
            &rust_biguint!(0),
            |sc| {
                sc.set_latest_price_feed(
                    managed_buffer!(EGLD_TICKER),
                    managed_buffer!(DOLLAR_TICKER),
                    managed_biguint!(EGLD_PRICE_DROPPED_IN_DOLLARS),
                );
            },
        )
        .assert_ok();

    lending_setup
        .b_mock
        .set_esdt_balance(&liquidator_addr, USDC_TOKEN_ID, &rust_biguint!(100_000));

    lending_setup
        .b_mock
        .execute_esdt_transfer(
            &liquidator_addr,
            &lending_setup.liquidity_pool_usdc_wrapper,
            USDC_TOKEN_ID,
            0,
            &rust_biguint!(100_000),
            |sc| {
                sc.liquidate(managed_address!(&liquidator_addr), 1, managed_biguint!(0));
            },
        )
        .assert_ok();

    // Check Liquidator Balance - LEND_EGLD tokens
    lending_setup.b_mock.check_nft_balance(
        &liquidator_addr,
        LEND_EGLD,
        1,
        &rust_biguint!(714),
        Option::<&Empty>::None,
    );

    lending_setup
        .b_mock
        .execute_esdt_transfer(
            &liquidator_addr,
            &lending_setup.liquidity_pool_egld_wrapper,
            LEND_EGLD,
            1,
            &rust_biguint!(714),
            |sc| {
                sc.reduce_position_after_liquidation();
            },
        )
        .assert_ok();

    // Check Liquidity Pool - LEND_EGLD tokens
    lending_setup.b_mock.check_nft_balance(
        &lending_setup.liquidity_pool_usdc_wrapper.address_ref(),
        LEND_EGLD,
        1,
        &rust_biguint!(286),
        Option::<&Empty>::None,
    );
}

#[test]
fn partial_liquidate_with_bonus_test() {
    let mut lending_setup = LendingSetup::deploy_lending(
        lending_pool::contract_obj,
        liquidity_pool::contract_obj,
        aggregator_mock::contract_obj,
    );
    let borrower_addr = lending_setup.first_user_addr.clone();
    let liquidator_addr = lending_setup.second_user_addr.clone();

    lending_setup
        .b_mock
        .set_esdt_balance(&borrower_addr, USDC_TOKEN_ID, &rust_biguint!(200_000));

    // Deposit USDC
    lending_setup
        .b_mock
        .execute_esdt_transfer(
            &borrower_addr,
            &lending_setup.liquidity_pool_usdc_wrapper,
            USDC_TOKEN_ID,
            0,
            &rust_biguint!(200_000),
            |sc| {
                sc.deposit_asset(managed_address!(&borrower_addr));
            },
        )
        .assert_ok();

    // Check LUSDC
    lending_setup.b_mock.check_nft_balance(
        &borrower_addr,
        LEND_USDC_TOKEN_ID,
        1,
        &rust_biguint!(200_000),
        Option::<&Empty>::None,
    );

    lending_setup
        .b_mock
        .set_esdt_balance(&borrower_addr, EGLD_TOKEN_ID, &rust_biguint!(1_000));

    // Deposit EGLD
    lending_setup
        .b_mock
        .execute_esdt_transfer(
            &borrower_addr,
            &lending_setup.liquidity_pool_egld_wrapper,
            EGLD_TOKEN_ID,
            0,
            &rust_biguint!(1_000),
            |sc| {
                sc.deposit_asset(managed_address!(&borrower_addr));
            },
        )
        .assert_ok();

    // Check LEGLD
    lending_setup.b_mock.check_nft_balance(
        &borrower_addr,
        LEND_EGLD,
        1,
        &rust_biguint!(1_000),
        Option::<&Empty>::None,
    );

    // Borrow USDC
    lending_setup
        .b_mock
        .execute_esdt_transfer(
            &borrower_addr,
            &lending_setup.liquidity_pool_usdc_wrapper,
            LEND_EGLD,
            1,
            &rust_biguint!(1_000),
            |sc| {
                sc.borrow(
                    managed_address!(&borrower_addr),
                    managed_biguint!(500_000_000),
                );
            },
        )
        .assert_ok();

    // lending_setup.b_mock.dump_state_for_account_hex_attributes(&borrower_addr);

    // Check received BUSDC
    lending_setup.b_mock.check_nft_balance(
        &borrower_addr,
        BORROW_USDC_TOKEN_ID,
        1,
        &rust_biguint!(100_000),
        Some(&Vec::<u8>::new()),
    );

    // Check received USDC
    lending_setup.b_mock.check_nft_balance(
        &borrower_addr,
        USDC_TOKEN_ID,
        0,
        &rust_biguint!(100_000),
        Some(&Vec::<u8>::new()),
    );

    lending_setup
        .b_mock
        .execute_tx(
            &liquidator_addr,
            &lending_setup.price_aggregator_wrapper,
            &rust_biguint!(0),
            |sc| {
                sc.set_latest_price_feed(
                    managed_buffer!(EGLD_TICKER),
                    managed_buffer!(DOLLAR_TICKER),
                    managed_biguint!(EGLD_PRICE_DROPPED_IN_DOLLARS),
                );
            },
        )
        .assert_ok();

    lending_setup
        .b_mock
        .set_esdt_balance(&liquidator_addr, USDC_TOKEN_ID, &rust_biguint!(100_000));

    lending_setup
        .b_mock
        .execute_esdt_transfer(
            &liquidator_addr,
            &lending_setup.liquidity_pool_usdc_wrapper,
            USDC_TOKEN_ID,
            0,
            &rust_biguint!(100_000),
            |sc| {
                sc.liquidate(
                    managed_address!(&liquidator_addr),
                    1,
                    managed_biguint!(5_000_000),
                );
            },
        )
        .assert_ok();

    // Check Liquidator Balance - LEND_EGLD tokens
    lending_setup.b_mock.check_nft_balance(
        &liquidator_addr,
        LEND_EGLD,
        1,
        &rust_biguint!(717),
        Option::<&Empty>::None,
    );

    lending_setup
        .b_mock
        .execute_esdt_transfer(
            &liquidator_addr,
            &lending_setup.liquidity_pool_egld_wrapper,
            LEND_EGLD,
            1,
            &rust_biguint!(717),
            |sc| {
                sc.reduce_position_after_liquidation();
            },
        )
        .assert_ok();

    // Check Liquidity Pool - LEND_EGLD tokens
    lending_setup.b_mock.check_nft_balance(
        &lending_setup.liquidity_pool_usdc_wrapper.address_ref(),
        LEND_EGLD,
        1,
        &rust_biguint!(283),
        Option::<&Empty>::None,
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

    // Supply/Deposit
    lending_setup.b_mock.set_esdt_balance(
        &supplier_addr,
        USDC_TOKEN_ID,
        &rust_biguint!(2_000 * BP),
    );

    lending_setup
        .b_mock
        .execute_esdt_transfer(
            &supplier_addr,
            &lending_setup.liquidity_pool_usdc_wrapper,
            USDC_TOKEN_ID,
            0,
            &rust_biguint!(2_000 * BP),
            |sc| {
                sc.deposit_asset(managed_address!(&supplier_addr));
            },
        )
        .assert_ok();

    lending_setup.b_mock.check_nft_balance(
        &supplier_addr,
        LEND_USDC_TOKEN_ID,
        1,
        &rust_biguint!(2_000 * BP),
        Option::<&Empty>::None,
    );

    lending_setup.b_mock.set_block_round(10);

    // Borrow
    lending_setup.b_mock.set_nft_balance(
        &borrower_addr,
        LEND_EGLD,
        1,
        &rust_biguint!(10 * BP),
        &Empty,
    );

    lending_setup
        .b_mock
        .execute_esdt_transfer(
            &borrower_addr,
            &lending_setup.liquidity_pool_usdc_wrapper,
            LEND_EGLD,
            1,
            &rust_biguint!(10 * BP),
            |sc| {
                sc.borrow(
                    managed_address!(&borrower_addr),
                    managed_biguint!(500_000_000),
                );
            },
        )
        .assert_ok();

    lending_setup.b_mock.check_nft_balance(
        &borrower_addr,
        BORROW_USDC_TOKEN_ID,
        1,
        &rust_biguint!(1_000 * BP),
        Some(&Vec::<u8>::new()),
    );

    lending_setup.b_mock.check_nft_balance(
        &borrower_addr,
        USDC_TOKEN_ID,
        0,
        &rust_biguint!(1_000 * BP),
        Some(&Vec::<u8>::new()),
    );

    lending_setup.b_mock.set_block_round(12);

    // Repay
    lending_setup.b_mock.set_esdt_balance(
        &borrower_addr,
        USDC_TOKEN_ID,
        &rust_biguint!(1_050 * BP),
    );

    let mut payments = Vec::with_capacity(2);

    payments.push(TxInputESDT {
        token_identifier: BORROW_USDC_TOKEN_ID.to_vec(),
        nonce: 1,
        value: rust_biguint!(1_000 * BP),
    });

    payments.push(TxInputESDT {
        token_identifier: USDC_TOKEN_ID.to_vec(),
        nonce: 0,
        value: rust_biguint!(1_050 * BP),
    });

    lending_setup
        .b_mock
        .execute_esdt_multi_transfer(
            &borrower_addr,
            &lending_setup.liquidity_pool_usdc_wrapper,
            &payments,
            |sc| {
                sc.repay(managed_address!(&borrower_addr));
            },
        )
        .assert_ok();

    lending_setup.b_mock.check_nft_balance(
        &borrower_addr,
        BORROW_USDC_TOKEN_ID,
        1,
        &rust_biguint!(0),
        Some(&Vec::<u8>::new()),
    );

    lending_setup
        .b_mock
        .check_esdt_balance(&borrower_addr, USDC_TOKEN_ID, &rust_biguint!(0));

    lending_setup.b_mock.check_nft_balance(
        &borrower_addr,
        LEND_EGLD,
        1,
        &rust_biguint!(10 * BP),
        Some(&Vec::<u8>::new()),
    );

    // Withdraw
    lending_setup
        .b_mock
        .execute_esdt_transfer(
            &supplier_addr,
            &lending_setup.liquidity_pool_usdc_wrapper,
            LEND_USDC_TOKEN_ID,
            1,
            &rust_biguint!(2_000 * BP),
            |sc| {
                sc.withdraw(managed_address!(&supplier_addr));
            },
        )
        .assert_ok();

    lending_setup.b_mock.check_nft_balance(
        &supplier_addr,
        LEND_USDC_TOKEN_ID,
        1,
        &rust_biguint!(0),
        Option::<&Empty>::None,
    );

    lending_setup.b_mock.check_nft_balance(
        &supplier_addr,
        USDC_TOKEN_ID,
        0,
        &rust_biguint!(2_050 * BP),
        Some(&Empty),
    );
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

    // Supply/Deposit
    lending_setup.b_mock.set_esdt_balance(
        &supplier_addr,
        USDC_TOKEN_ID,
        &rust_biguint!(2_000 * BP),
    );

    lending_setup
        .b_mock
        .execute_esdt_transfer(
            &supplier_addr,
            &lending_setup.liquidity_pool_usdc_wrapper,
            USDC_TOKEN_ID,
            0,
            &rust_biguint!(2_000 * BP),
            |sc| {
                sc.deposit_asset(managed_address!(&supplier_addr));
            },
        )
        .assert_ok();

    lending_setup.b_mock.check_nft_balance(
        &supplier_addr,
        LEND_USDC_TOKEN_ID,
        1,
        &rust_biguint!(2_000 * BP),
        Option::<&Empty>::None,
    );

    lending_setup.b_mock.set_block_round(10);

    // Borrow
    lending_setup.b_mock.set_nft_balance(
        &borrower_addr,
        LEND_EGLD,
        1,
        &rust_biguint!(10 * BP),
        &Empty,
    );

    lending_setup
        .b_mock
        .execute_esdt_transfer(
            &borrower_addr,
            &lending_setup.liquidity_pool_usdc_wrapper,
            LEND_EGLD,
            1,
            &rust_biguint!(10 * BP),
            |sc| {
                sc.borrow(
                    managed_address!(&borrower_addr),
                    managed_biguint!(500_000_000),
                );
            },
        )
        .assert_ok();

    lending_setup.b_mock.check_nft_balance(
        &borrower_addr,
        BORROW_USDC_TOKEN_ID,
        1,
        &rust_biguint!(1_000 * BP),
        Some(&Vec::<u8>::new()),
    );

    lending_setup.b_mock.check_nft_balance(
        &borrower_addr,
        USDC_TOKEN_ID,
        0,
        &rust_biguint!(1_000 * BP),
        Some(&Vec::<u8>::new()),
    );

    lending_setup.b_mock.set_block_round(12);

    // Repay 600 USDC + interest (30 USDC)
    lending_setup
        .b_mock
        .set_esdt_balance(&borrower_addr, USDC_TOKEN_ID, &rust_biguint!(630 * BP));

    let mut payments = Vec::with_capacity(2);

    payments.push(TxInputESDT {
        token_identifier: BORROW_USDC_TOKEN_ID.to_vec(),
        nonce: 1,
        value: rust_biguint!(600 * BP),
    });

    payments.push(TxInputESDT {
        token_identifier: USDC_TOKEN_ID.to_vec(),
        nonce: 0,
        value: rust_biguint!(630 * BP),
    });

    lending_setup
        .b_mock
        .execute_esdt_multi_transfer(
            &borrower_addr,
            &lending_setup.liquidity_pool_usdc_wrapper,
            &payments,
            |sc| {
                sc.repay(managed_address!(&borrower_addr));
            },
        )
        .assert_ok();

    lending_setup.b_mock.check_nft_balance(
        &borrower_addr,
        BORROW_USDC_TOKEN_ID,
        1,
        &rust_biguint!(400 * BP),
        Some(&Vec::<u8>::new()),
    );

    lending_setup
        .b_mock
        .check_esdt_balance(&borrower_addr, USDC_TOKEN_ID, &rust_biguint!(0));

    lending_setup.b_mock.check_nft_balance(
        &borrower_addr,
        LEND_EGLD,
        1,
        &rust_biguint!(6 * BP),
        Some(&Vec::<u8>::new()),
    );

    // repay the rest of 400 USDC

    lending_setup.b_mock.set_block_round(14);

    lending_setup
        .b_mock
        .set_esdt_balance(&borrower_addr, USDC_TOKEN_ID, &rust_biguint!(428 * BP));

    let mut payments = Vec::with_capacity(2);

    payments.push(TxInputESDT {
        token_identifier: BORROW_USDC_TOKEN_ID.to_vec(),
        nonce: 1,
        value: rust_biguint!(400 * BP),
    });

    payments.push(TxInputESDT {
        token_identifier: USDC_TOKEN_ID.to_vec(),
        nonce: 0,
        value: rust_biguint!(428 * BP),
    });

    lending_setup
        .b_mock
        .execute_esdt_multi_transfer(
            &borrower_addr,
            &lending_setup.liquidity_pool_usdc_wrapper,
            &payments,
            |sc| {
                sc.repay(managed_address!(&borrower_addr));
            },
        )
        .assert_ok();

    lending_setup.b_mock.check_nft_balance(
        &borrower_addr,
        BORROW_USDC_TOKEN_ID,
        1,
        &rust_biguint!(0),
        Some(&Vec::<u8>::new()),
    );

    lending_setup
        .b_mock
        .check_esdt_balance(&borrower_addr, USDC_TOKEN_ID, &rust_biguint!(0));

    lending_setup.b_mock.check_nft_balance(
        &borrower_addr,
        LEND_EGLD,
        1,
        &rust_biguint!(10 * BP),
        Some(&Vec::<u8>::new()),
    );

    // Withdraw
    lending_setup
        .b_mock
        .execute_esdt_transfer(
            &supplier_addr,
            &lending_setup.liquidity_pool_usdc_wrapper,
            LEND_USDC_TOKEN_ID,
            1,
            &rust_biguint!(2_000 * BP),
            |sc| {
                sc.withdraw(managed_address!(&supplier_addr));
            },
        )
        .assert_ok();

    lending_setup.b_mock.check_nft_balance(
        &supplier_addr,
        LEND_USDC_TOKEN_ID,
        1,
        &rust_biguint!(0),
        Option::<&Empty>::None,
    );

    lending_setup.b_mock.check_nft_balance(
        &supplier_addr,
        USDC_TOKEN_ID,
        0,
        &rust_biguint!(2_058 * BP),
        Some(&Empty),
    );
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

    // Supply/Deposit
    lending_setup.b_mock.set_esdt_balance(
        &supplier_addr,
        USDC_TOKEN_ID,
        &rust_biguint!(2_000 * BP),
    );

    lending_setup
        .b_mock
        .execute_esdt_transfer(
            &supplier_addr,
            &lending_setup.liquidity_pool_usdc_wrapper,
            USDC_TOKEN_ID,
            0,
            &rust_biguint!(2_000 * BP),
            |sc| {
                sc.deposit_asset(managed_address!(&supplier_addr));
            },
        )
        .assert_ok();

    lending_setup.b_mock.check_nft_balance(
        &supplier_addr,
        LEND_USDC_TOKEN_ID,
        1,
        &rust_biguint!(2_000 * BP),
        Option::<&Empty>::None,
    );

    lending_setup.b_mock.set_block_round(10);

    // Borrow
    lending_setup.b_mock.set_nft_balance(
        &borrower_addr,
        LEND_EGLD,
        1,
        &rust_biguint!(10 * BP),
        &Empty,
    );

    lending_setup
        .b_mock
        .execute_esdt_transfer(
            &borrower_addr,
            &lending_setup.liquidity_pool_usdc_wrapper,
            LEND_EGLD,
            1,
            &rust_biguint!(10 * BP),
            |sc| {
                sc.borrow(
                    managed_address!(&borrower_addr),
                    managed_biguint!(500_000_000),
                );
            },
        )
        .assert_ok();

    lending_setup.b_mock.check_nft_balance(
        &borrower_addr,
        BORROW_USDC_TOKEN_ID,
        1,
        &rust_biguint!(1_000 * BP),
        Some(&Vec::<u8>::new()),
    );

    lending_setup.b_mock.check_nft_balance(
        &borrower_addr,
        USDC_TOKEN_ID,
        0,
        &rust_biguint!(1_000 * BP),
        Some(&Vec::<u8>::new()),
    );

    lending_setup.b_mock.set_block_round(12);

    // Repay
    lending_setup.b_mock.set_esdt_balance(
        &borrower_addr,
        USDC_TOKEN_ID,
        &rust_biguint!(1_050 * BP),
    );

    let mut payments = Vec::with_capacity(2);

    payments.push(TxInputESDT {
        token_identifier: BORROW_USDC_TOKEN_ID.to_vec(),
        nonce: 1,
        value: rust_biguint!(1_000 * BP),
    });

    payments.push(TxInputESDT {
        token_identifier: USDC_TOKEN_ID.to_vec(),
        nonce: 0,
        value: rust_biguint!(1_050 * BP),
    });

    lending_setup
        .b_mock
        .execute_esdt_multi_transfer(
            &borrower_addr,
            &lending_setup.liquidity_pool_usdc_wrapper,
            &payments,
            |sc| {
                sc.repay(managed_address!(&borrower_addr));
            },
        )
        .assert_ok();

    lending_setup.b_mock.check_nft_balance(
        &borrower_addr,
        BORROW_USDC_TOKEN_ID,
        1,
        &rust_biguint!(0),
        Some(&Vec::<u8>::new()),
    );

    lending_setup
        .b_mock
        .check_esdt_balance(&borrower_addr, USDC_TOKEN_ID, &rust_biguint!(0));

    lending_setup.b_mock.check_nft_balance(
        &borrower_addr,
        LEND_EGLD,
        1,
        &rust_biguint!(10 * BP),
        Some(&Vec::<u8>::new()),
    );

    // Withdraw (400 USDC) - round 1
    lending_setup
        .b_mock
        .execute_esdt_transfer(
            &supplier_addr,
            &lending_setup.liquidity_pool_usdc_wrapper,
            LEND_USDC_TOKEN_ID,
            1,
            &rust_biguint!(400 * BP),
            |sc| {
                sc.withdraw(managed_address!(&supplier_addr));
            },
        )
        .assert_ok();

    lending_setup.b_mock.check_nft_balance(
        &supplier_addr,
        LEND_USDC_TOKEN_ID,
        1,
        &rust_biguint!(1600 * BP),
        Option::<&Empty>::None,
    );

    lending_setup.b_mock.check_nft_balance(
        &supplier_addr,
        USDC_TOKEN_ID,
        0,
        &rust_biguint!(410 * BP),
        Some(&Empty),
    );

    // Withdraw (400 USDC) - round 2
    lending_setup
        .b_mock
        .execute_esdt_transfer(
            &supplier_addr,
            &lending_setup.liquidity_pool_usdc_wrapper,
            LEND_USDC_TOKEN_ID,
            1,
            &rust_biguint!(400 * BP),
            |sc| {
                sc.withdraw(managed_address!(&supplier_addr));
            },
        )
        .assert_ok();

    lending_setup.b_mock.check_nft_balance(
        &supplier_addr,
        LEND_USDC_TOKEN_ID,
        1,
        &rust_biguint!(1200 * BP),
        Option::<&Empty>::None,
    );

    lending_setup.b_mock.check_nft_balance(
        &supplier_addr,
        USDC_TOKEN_ID,
        0,
        &rust_biguint!(820 * BP),
        Some(&Empty),
    );

    // Withdraw (400 USDC) - round 3
    lending_setup
        .b_mock
        .execute_esdt_transfer(
            &supplier_addr,
            &lending_setup.liquidity_pool_usdc_wrapper,
            LEND_USDC_TOKEN_ID,
            1,
            &rust_biguint!(400 * BP),
            |sc| {
                sc.withdraw(managed_address!(&supplier_addr));
            },
        )
        .assert_ok();

    lending_setup.b_mock.check_nft_balance(
        &supplier_addr,
        LEND_USDC_TOKEN_ID,
        1,
        &rust_biguint!(800 * BP),
        Option::<&Empty>::None,
    );

    lending_setup.b_mock.check_nft_balance(
        &supplier_addr,
        USDC_TOKEN_ID,
        0,
        &rust_biguint!(1230 * BP),
        Some(&Empty),
    );

    // Withdraw (400 USDC) - round 4
    lending_setup
        .b_mock
        .execute_esdt_transfer(
            &supplier_addr,
            &lending_setup.liquidity_pool_usdc_wrapper,
            LEND_USDC_TOKEN_ID,
            1,
            &rust_biguint!(400 * BP),
            |sc| {
                sc.withdraw(managed_address!(&supplier_addr));
            },
        )
        .assert_ok();

    lending_setup.b_mock.check_nft_balance(
        &supplier_addr,
        LEND_USDC_TOKEN_ID,
        1,
        &rust_biguint!(400 * BP),
        Option::<&Empty>::None,
    );

    lending_setup.b_mock.check_nft_balance(
        &supplier_addr,
        USDC_TOKEN_ID,
        0,
        &rust_biguint!(1640 * BP),
        Some(&Empty),
    );

    // Withdraw (400 USDC) - round 5
    lending_setup
        .b_mock
        .execute_esdt_transfer(
            &supplier_addr,
            &lending_setup.liquidity_pool_usdc_wrapper,
            LEND_USDC_TOKEN_ID,
            1,
            &rust_biguint!(400 * BP),
            |sc| {
                sc.withdraw(managed_address!(&supplier_addr));
            },
        )
        .assert_ok();

    lending_setup.b_mock.check_nft_balance(
        &supplier_addr,
        LEND_USDC_TOKEN_ID,
        1,
        &rust_biguint!(0),
        Option::<&Empty>::None,
    );

    lending_setup.b_mock.check_nft_balance(
        &supplier_addr,
        USDC_TOKEN_ID,
        0,
        &rust_biguint!(2050 * BP),
        Some(&Empty),
    );
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

    // Supply/Deposit
    lending_setup.b_mock.set_esdt_balance(
        &supplier_addr,
        USDC_TOKEN_ID,
        &rust_biguint!(1_000 * BP),
    );

    lending_setup
        .b_mock
        .execute_esdt_transfer(
            &supplier_addr,
            &lending_setup.liquidity_pool_usdc_wrapper,
            USDC_TOKEN_ID,
            0,
            &rust_biguint!(1_000 * BP),
            |sc| {
                sc.deposit_asset(managed_address!(&supplier_addr));
            },
        )
        .assert_ok();

    lending_setup.b_mock.check_nft_balance(
        &supplier_addr,
        LEND_USDC_TOKEN_ID,
        1,
        &rust_biguint!(1_000 * BP),
        Option::<&Empty>::None,
    );

    // Supply/Deposit
    lending_setup.b_mock.set_esdt_balance(
        &supplier2_addr,
        USDC_TOKEN_ID,
        &rust_biguint!(1_000 * BP),
    );

    lending_setup
        .b_mock
        .execute_esdt_transfer(
            &supplier2_addr,
            &lending_setup.liquidity_pool_usdc_wrapper,
            USDC_TOKEN_ID,
            0,
            &rust_biguint!(1_000 * BP),
            |sc| {
                sc.deposit_asset(managed_address!(&supplier2_addr));
            },
        )
        .assert_ok();

    lending_setup.b_mock.check_nft_balance(
        &supplier2_addr,
        LEND_USDC_TOKEN_ID,
        2,
        &rust_biguint!(1_000 * BP),
        Option::<&Empty>::None,
    );

    lending_setup.b_mock.set_block_round(10);

    // Borrow
    lending_setup.b_mock.set_nft_balance(
        &borrower_addr,
        LEND_EGLD,
        1,
        &rust_biguint!(10 * BP),
        &Empty,
    );

    lending_setup
        .b_mock
        .execute_esdt_transfer(
            &borrower_addr,
            &lending_setup.liquidity_pool_usdc_wrapper,
            LEND_EGLD,
            1,
            &rust_biguint!(10 * BP),
            |sc| {
                sc.borrow(
                    managed_address!(&borrower_addr),
                    managed_biguint!(500_000_000),
                );
            },
        )
        .assert_ok();

    lending_setup.b_mock.check_nft_balance(
        &borrower_addr,
        BORROW_USDC_TOKEN_ID,
        1,
        &rust_biguint!(1_000 * BP),
        Some(&Vec::<u8>::new()),
    );

    lending_setup.b_mock.check_nft_balance(
        &borrower_addr,
        USDC_TOKEN_ID,
        0,
        &rust_biguint!(1_000 * BP),
        Some(&Vec::<u8>::new()),
    );

    lending_setup.b_mock.set_block_round(12);

    // Repay
    lending_setup.b_mock.set_esdt_balance(
        &borrower_addr,
        USDC_TOKEN_ID,
        &rust_biguint!(1_050 * BP),
    );

    let mut payments = Vec::with_capacity(2);

    payments.push(TxInputESDT {
        token_identifier: BORROW_USDC_TOKEN_ID.to_vec(),
        nonce: 1,
        value: rust_biguint!(1_000 * BP),
    });

    payments.push(TxInputESDT {
        token_identifier: USDC_TOKEN_ID.to_vec(),
        nonce: 0,
        value: rust_biguint!(1_050 * BP),
    });

    lending_setup
        .b_mock
        .execute_esdt_multi_transfer(
            &borrower_addr,
            &lending_setup.liquidity_pool_usdc_wrapper,
            &payments,
            |sc| {
                sc.repay(managed_address!(&borrower_addr));
            },
        )
        .assert_ok();

    lending_setup.b_mock.check_nft_balance(
        &borrower_addr,
        BORROW_USDC_TOKEN_ID,
        1,
        &rust_biguint!(0),
        Some(&Vec::<u8>::new()),
    );

    lending_setup
        .b_mock
        .check_esdt_balance(&borrower_addr, USDC_TOKEN_ID, &rust_biguint!(0));

    lending_setup.b_mock.check_nft_balance(
        &borrower_addr,
        LEND_EGLD,
        1,
        &rust_biguint!(10 * BP),
        Some(&Vec::<u8>::new()),
    );

    // Withdraw - Supplier1
    lending_setup
        .b_mock
        .execute_esdt_transfer(
            &supplier_addr,
            &lending_setup.liquidity_pool_usdc_wrapper,
            LEND_USDC_TOKEN_ID,
            1,
            &rust_biguint!(1_000 * BP),
            |sc| {
                sc.withdraw(managed_address!(&supplier_addr));
            },
        )
        .assert_ok();

    lending_setup.b_mock.check_nft_balance(
        &supplier_addr,
        LEND_USDC_TOKEN_ID,
        1,
        &rust_biguint!(0),
        Option::<&Empty>::None,
    );

    lending_setup.b_mock.check_nft_balance(
        &supplier_addr,
        USDC_TOKEN_ID,
        0,
        &rust_biguint!(1_025 * BP),
        Some(&Empty),
    );

    // Withdraw - Supplier 2
    lending_setup
        .b_mock
        .execute_esdt_transfer(
            &supplier2_addr,
            &lending_setup.liquidity_pool_usdc_wrapper,
            LEND_USDC_TOKEN_ID,
            2,
            &rust_biguint!(1_000 * BP),
            |sc| {
                sc.withdraw(managed_address!(&supplier2_addr));
            },
        )
        .assert_ok();

    lending_setup.b_mock.check_nft_balance(
        &supplier2_addr,
        LEND_USDC_TOKEN_ID,
        1,
        &rust_biguint!(0),
        Option::<&Empty>::None,
    );

    lending_setup.b_mock.check_nft_balance(
        &supplier2_addr,
        USDC_TOKEN_ID,
        0,
        &rust_biguint!(1_025 * BP),
        Some(&Empty),
    );
}
