ALICE="/home/boop/Elrond/wallet/testnet/wallet_key.pem"
ALICE_ADDRESS=0x9bc31161f8c0cad0af2beafe08168fc57108fc054338e8016ca5a173e0a0e3df

ADDRESS=$(erdpy data load --key=address-testnet)
DEPLOY_TRANSACTION=$(erdpy data load --key=deployTransaction-testnet)

PROXY=https://devnet-api.elrond.com
CHAIN_ID=D

PROJECT="../../lending_pool"

AMOUNT=1000000000000000000 # 1 token (18 decimals)
LIQ_DEPOSIT_ENDPOINT=0x6465706f736974 # deposit_asset
RECEIVER=${ALICE}

# update manually
LIQ_POOL_ADDRESS=0x

# issue an esdt token first
ESDT_TICKER=0x4543432d316237343533 # ECC-1b7453
ESDT_NAME=0x4575726f7065616e436f6d6974746565 # EuropeanComittee

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
    erdpy contract upgrade ${ADDRESS} --project==${PROJECT} --recall-nonce --pem=${ALICE} --gas-limit=75000000 --outfile="upgrade.json" --proxy=${PROXY} --chain=${CHAIN_ID} --send || return
}

# SC calls

deposit() {
    erdpy contract call ${ADDRESS} --recall-nonce --pem=${ALICE} -gas-limit=80000000 --function="ESDTTransfer" --arguments ${ESDT_TICKER} ${AMOUNT} ${LIQ_DEPOSIT_ENDPOINT} ${RECEIVER} --proxy=${PROXY} --chain=${CHAIN_ID} --send
}

setPoolAddress() {
    erdpy contract call ${ADDRESS} --recall-nonce --pem=${ALICE} -gas-limit=80000000 --function="setPoolAddress" --arguments ${ESDT_TICKER} ${LIQ_POOL_ADDRESS} --proxy=${PROXY} --chain=${CHAIN_ID} --send
}

# Queries

getPoolAddress() {
    erdpy contract query ${ADDRESS} --function="getPoolAddress" --arguments ${ESDT_TICKER} --proxy=${PROXY}
}

