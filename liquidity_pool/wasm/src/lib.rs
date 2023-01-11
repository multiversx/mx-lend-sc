////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

#![no_std]

elrond_wasm_node::wasm_endpoints! {
    liquidity_pool
    (
        addCollateral
        borrow
        borrowIndexLastUpdateRound
        borrowToken
        getAccountPositions
        getAccountToken
        getAggregatorAddress
        getBorrowIndex
        getBorrowRate
        getCapitalUtilisation
        getDebtInterest
        getDepositRate
        getLendToken
        getLiquidationThreshold
        getPoolAsset
        getPoolParams
        getReserves
        getRewardsReserves
        getSuppliedAmount
        getSupplyIndex
        getTotalBorrow
        getTotalCapital
        remove_collateral
        repay
        sendTokens
        setPriceAggregatorAddress
        updateBorrowsWithDebt
        updateCollateralWithInterest
    )
}

elrond_wasm_node::wasm_empty_callback! {}
