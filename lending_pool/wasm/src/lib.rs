////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

#![no_std]

elrond_wasm_node::wasm_endpoints! {
    lending_pool
    (
        borrow
        createLiquidityPool
        deposit
        getAssetLiquidationBonus
        getAssetLoanToValue
        getLiqPoolTemplateAddress
        getPoolAddress
        getPoolAllowed
        issueBorrowToken
        issueLendToken
        liquidate
        repay
        setAggregator
        setAssetLiquidationBonus
        setAssetLoanToValue
        setBorrowRoles
        setLendRoles
        setTokenIdAfterIssue
        upgradeLiquidityPool
        withdraw
    )
}

elrond_wasm_node::wasm_empty_callback! {}
