////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

#![no_std]

elrond_wasm_node::wasm_endpoints! {
    lending_pool
    (
        callBack
        addCollateral
        borrow
        createLiquidityPool
        enterMarket
        exitMarket
        getAccountPositions
        getAccountToken
        getAggregatorAddress
        getAssetLiquidationBonus
        getAssetLoanToValue
        getBorrowPositions
        getCollateralAmountForToken
        getDepositPositions
        getLiqPoolTemplateAddress
        getPoolAddress
        getPoolAllowed
        getTotalBorrowInDollars
        getTotalCollateralAvailable
        liquidate
        registerAccountToken
        removeCollateral
        repay
        setAggregator
        setAssetLiquidationBonus
        setAssetLoanToValue
        setPriceAggregatorAddress
        updateBorrowsWithDebt
        updateCollateralWithInterest
        upgradeLiquidityPool
    )
}
