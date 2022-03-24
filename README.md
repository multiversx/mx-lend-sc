# elrond-lend-rs

## Introduction

As the next stage in the evolution of DeFi, decentralized lending brings more opportunities.
In contrast to decentralized exchanges, lending involves two parties: the *borrower* and the *provider*.
Suppliers deposit assets into a pool to earn interest.
Borrowers can obtain an asset by placing other assets as collateral.
For the borrowed asset, the borrower pays interest *over time* to the suppliers.

Lending Protocol enhanches the opportunities for DeFi participants to further grow the ecosystem:

1. Instead of staking tokens into liquidity mining pools, users can now lend their tokens.
Therefore, users can now earn yield *without* the limitation of *impermanent loss* and not having to hold two tokens at a fixed ratio (as for liquidity pool).

2. Users can deposit their tokens as collateral to borrow other class of tokens.

These benefits are not risk-free.
Borrowers have the risk of *liquidation*, when the borrowing balance exceeds the total collateral value.
Supplieres have the risk of *short fall*, which happens when the liquidation does not recover sufficient obligation. 

The main idea behind the lending/borrowing collateral in a p2p way is that of utilisation ratios.
This means that contracts don’t need to hold state about rates, or enforce rates statically. 
It achieves this by computing the current borrow/deposit rate in a dynamic way, taking the relation between the particular pool’s reserves and the utilisation rate at that moment (amount of collateral borrowed).

## Protocol Architecture

We propose a lending strategy based on algorithmic pool-based liquidity.
It supports multiple assets, with a Liquidity Pool for each asset.
The addition of a new asset requires the deployment of another liquidity pool contract.
The Lending Pool deploys and manages the Liquidity Pools as an owner.
The protocol supports multiple Liquidity Pools, which have their own state and a Lending Pool which acts as Router and Proxy for routing and executing transactions based on the transaction input.

Supliers wanting to supply a token **TEST**, will deposit 

![Supply-Borrow Lending Pool](https://user-images.githubusercontent.com/3630188/159972815-8c7c746d-3f0e-444d-8bdc-81287ddc95c1.png)


### Interest Rate Model

We use an interest rate model which is based on the pool’s capital utilisation, which we can call *U*.
Liquidity risk materialises when utilisation is high, its becomes more problematic as *U* gets closer to 100%.
To tailor the model to this constraint, the interest rate curve is split in two parts around an optimal utilisation rate *Uoptimal*.
Before *Uoptimal* the slope is small, after it starts rising sharply. 

For *U >= Uoptimal*, the interest rates rise, because the capital is scarce.
For *U < Uoptimal*, the interest rates decrease, because the capital is plentiful.

The interest rate *Rt* formulas depending on the capital utilisation of the pool, where:

![image](https://user-images.githubusercontent.com/3630188/160089036-63f00d49-4a4c-4de0-8a5a-d4be220d9004.png)

*R0*, *Rslope1* and *Rslope2* are predefined values

#### Simulations

![image](https://user-images.githubusercontent.com/3630188/160086654-8cfb9201-abb6-4b56-a57d-5bb72a0273e9.png)


In this case we have a pool where the asset is a Stablecoin, which is not volatile, so the borrow rate increases slightly with utilisation.
An optimal Utilisation Rate of 80% has been chosen, from which the rate increases sharply, as it provides enough safety in case of market downside.

![image](https://user-images.githubusercontent.com/3630188/160086719-ed30925f-f9f4-43d3-a07a-a5b7f2436477.png)


For a more volatile asset the borrow rate follows the same pattern as in the case of BUSD, but a much lower optimal utilisation rate has been chosen, with a steeper slope, as this assets are more likely to become undercollateralized and put the protocol solvency at risk.
Using a steeper slope once the Borrowed Amounts exceed the Optimal Rate ensures that loans are being repaid in time or, if not, liquidated before it becomes unhealthy for the pool’s reserves.

The simulations were run in a python environment using multiple timeframes and multiple borrows.
**In a real scenario, the curves are not going to be perfectly smooth as pictured above, as a pool’s reserves won’t follow a linear action.**


### Risk Parameters

**Loan to Value (LTV)** ratio defines the maximum amount of currency that can be borrowed with a specific collateral.
It’s expressed in percentage: at LTV=75%, for every 1 EGLD worth of collateral, borrowers will be able to borrow 0.75 EGLD worth of the corresponding (different) currency.
Once a borrow is taken, the LTV evolves with market conditions.

**Liquidation Threshold** is the percentage at which a position is defined as undercollateralised.
For example, a Liquidation threshold of 80% means that if the value rises above 80% of the collateral, the position is undercollateralised and could be liquidated.

The delta between the Loan-To-Value and the Liquidation Threshold is a safety cushion for borrowers.



## The LendingPool Contract

### Deposit

The deposit action is the simplest one and does not have any state check.

![Deposit Position (2)](https://user-images.githubusercontent.com/3630188/160102524-defb6b6f-870a-45b2-a037-ae210da2aae8.png)


The flow is as follows:
1. The user calls `deposit` function from the *Lending Pool*, which calls `deposit_asset` from the *Liquidity Pool*;
2. Mints metaESDT tokens with an unique nonce;
3. Creates a `DepositPosition` which contains the timestamp;
4. Deposit in the *Deposit Storage Mapper* an entry with the nonce (obtained from the 2nd step) and `DepositPosition` struct (obtained from the 3rd step);
5. Updates the *Reserves Storage Mapper*.


### Widthraw

### Borrow

### Repay

Borrowers can repay tokens to the market up to the **borrow balance**.
If partial repayment is made, the **borrow balance** may be non-zero, and continue to accrue interest.
Repayment is a transfer of tokens from the borrower back to the token market.

### Liquidate

If a borrower’s **borrowing balance exceeds** their total collateral value, it means that the protocol is at risk of suffering a loss if the borrower defaults on repayment.
We call this a shortfall event.
In order to reduce this risk, the protocol provides a public liquidate function which can be called by any user


