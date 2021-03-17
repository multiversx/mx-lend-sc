ALICE="/home/boop/elrondsdk/sandbox/testnet/wallets/users/alice.pem"
ALICE_ADDRESS=0x0139472eff6886771a982f3083da5d421f24c29181e63888228dc81ca60d69e1

ADDRESS=$(erdpy data load --key=address-testnet)
DEPLOY_TRANSACTION=$(erdpy data load --key=deployTransaction-testnet)

PROXY=http://localhost:7950
CHAIN_ID=local-testnet

PROJECT="../../lending_pool"

AMOUNT=1000000000000000000 # 1 token (18 decimals)
LIQ_DEPOSIT_ENDPOINT=0x6465706f736974 # deposit_asset
RECEIVER=${ALICE}

# update manually
LIQ_POOL_ADDRESS=erd1qqqqqqqqqqqqqpgqfzydqmdw7m2vazsp6u5p95yxz76t2p9rd8ss0zp9ts

ESDT_TICKER=0x57555344 # WUSD
ESDT_NAME=0x57726170706564555344 # WrappedUSD

deploy() {
    erdpy contract deploy --project=${PROJECT} --recall-nonce --pem=${ALICE} --gas-limit=75000000
    --outfile="deploy.json" --proxy=${PROXY} --chain=${CHAIN_ID} --send || return

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

deposit() {
    erdpy contract call ${ADDRESS} --recall-nonce --pem=${ALICE} -gas-limit=80000000 --function="ESDTTransfer" 
    --arguments ${ESDT_TICKER} ${AMOUNT} ${LIQ_DEPOSIT_ENDPOINT} ${RECEIVER} --proxy=${PROXY} --chain=${CHAIN_ID} --send
}

setPoolAddress() {
    erdpy contract call ${ADDRESS} --recall-nonce --pem=${ALICE} -gas-limit=80000000 --function=setPoolAddress 
    --arguments ${ESDT_TICKER} ${LIQ_POOL_ADDRESS} --proxy=${PROXY} --chain=${CHAIN_ID} --send
}

# Queries

getPoolAddress() {
    erdpy contract query ${ADDRESS} --function="lastIssuedToken" --arguments ${ESDT_TICKER} --proxy=${PROXY}
}
