ALICE="/home/boop/Elrond/wallet/testnet/wallet_key.pem"
ALICE_ADDRESS=0x9bc31161f8c0cad0af2beafe08168fc57108fc054338e8016ca5a173e0a0e3df

ADDRESS=$(erdpy data load --key=address-testnet)
DEPLOY_TRANSACTION=$(erdpy data load --key=deployTransaction-testnet)

PROXY=https://devnet-api.elrond.com
CHAIN_ID=D

ISSUE_VALUE=5000000000000000000 # 5 EGLD

PROJECT="../../liquidity_pool"

AMOUNT=1000000000000000000 # 1 token (18 decimals)

ESDT_TICKER=0x454343 # ECC
ESDT_TICKER_FULL=0x4543432d316237343533 # add after ESDT issue
LEND_TICKER=0x4c4543432d616565356431


deploy() {
    erdpy contract deploy --project=${PROJECT} --recall-nonce --pem=${ALICE} --gas-limit=150000000 --outfile="deploy.json" --proxy=${PROXY} --chain=${CHAIN_ID} --arguments ${ESDT_TICKER_FULL} --send || return

    TRANSACTION=$(erdpy data parse --file="deploy.json" --expression="data['emitted_tx']['hash']")
    ADDRESS=$(erdpy data parse --file="deploy.json" --expression="data['emitted_tx']['address']")

    erdpy data store --key=address-testnet --value=${ADDRESS}
    erdpy data store --key=deployTransaction-testnet --value=${TRANSACTION}

    echo ""
    echo "Smart contract address: ${ADDRESS}"
}

upgrade() {
    erdpy contract upgrade ${ADDRESS} --project=${PROJECT} --recall-nonce --pem=${ALICE} --gas-limit=150000000 --outfile="upgrade.json" --proxy=${PROXY} --chain=${CHAIN_ID} --arguments ${ESDT_TICKER_FULL} --send || return
}

# SC calls

issue_lend() {
    erdpy contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=80000000 --value=${ISSUE_VALUE} --function="issue" --arguments ${ESDT_TICKER_FULL} ${ESDT_TICKER} ${LIQ_DEPOSIT_ENDPOINT} 0x4c 4294967295000000000000000000 18 --proxy=${PROXY} --chain=${CHAIN_ID} --send
}

deposit_asset() {
    erdpy contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=80000000 --function="ESDTTransfer" --arguments ${ESDT_TICKER_FULL} ${AMOUNT} 0x6465706f7369745f6173736574 ${ALICE_ADDRESS} --proxy=${PROXY} --chain=${CHAIN_ID} --send
}

# Queries

getPoolAsset() {
    erdpy contract query ${ADDRESS} --function="poolAsset" --proxy=${PROXY}
}

getAssetReserve() {
    erdpy contract query ${ADDRESS} --function="getReserve" --arguments ${ESDT_TICKER_FULL} --proxy=${PROXY}
}

getLendReserve() {
    erdpy contract query ${ADDRESS} --function="getReserve" --arguments ${LEND_TICKER} --proxy=${PROXY}
}
