ALICE="/home/boop/elrondsdk/sandbox/testnet/wallets/users/alice.pem"
ALICE_ADDRESS=0x0139472eff6886771a982f3083da5d421f24c29181e63888228dc81ca60d69e1

ADDRESS=$(erdpy data load --key=address-testnet)
DEPLOY_TRANSACTION=$(erdpy data load --key=deployTransaction-testnet)

PROXY=http://localhost:7950
CHAIN_ID=local-testnet

PROJECT="../../liquidity_pool"

AMOUNT=1000000000000000000 # 1 token (18 decimals)

ESDT_TICKER=0x57555344 # WUSD
ESDT_TICKER_FULL=0x # add after ESDT issue
ESDT_NAME=0x57726170706564555344 # WrappedUSD

deploy() {
    erdpy contract deploy --project=${PROJECT} --recall-nonce --pem=${ALICE} --gas-limit=75000000 --outfile="deploy.json" --proxy=${PROXY} --chain=${CHAIN_ID} --send || return

    TRANSACTION=$(erdpy data parse --file="deploy.json" --expression="data['emitted_tx']['hash']")
    ADDRESS=$(erdpy data parse --file="deploy.json" --expression="data['emitted_tx']['address']")

    erdpy data store --key=address-testnet --value=${ADDRESS}
    erdpy data store --key=deployTransaction-testnet --value=${TRANSACTION}

    echo ""
    echo "Smart contract address: ${ADDRESS}"
}

upgrade() {
    erdpy contract upgrade ${ADDRESS} --project==${PROJECT} --recall-nonce --pem=${ALICE} --gas-limit=75000000 
    --outfile="upgrade.json" --proxy=${PROXY} --chain=${CHAIN_ID} --send || return
}

# SC calls

issue_lend() {
    erdpy contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=80000000 --function="issue" --arguments ${ESDT_TICKER_FULL} ${ESDT_TICKER} ${LIQ_DEPOSIT_ENDPOINT} 0x4c 4294967295000000000000000000 18 --proxy=${PROXY} --chain=${CHAIN_ID} --send
}

# Queries

getPoolAddress() {
    erdpy contract query ${ADDRESS} --function="lastIssuedToken" --arguments ${ESDT_TICKER} --proxy=${PROXY}
}
