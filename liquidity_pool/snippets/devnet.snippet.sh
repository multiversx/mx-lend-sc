PEM="$HOME/pems/dev.pem"

ADDRESS=$(erdpy data load --key=address-testnet)
DEPLOY_TRANSACTION=$(erdpy data load --key=deployTransaction-testnet)

PROXY=https://devnet-gateway.elrond.com
CHAIN_ID=D

PROJECT="../../liquidity_pool"

# init params
ASSET=0x544553542d333663616365
R_BASE=0
R_SLOPE1=40000000
R_SLOPE2=1000000000
U_OPTIMAL=800000000
RESERVE_FACTOR=100000000
LIQ_THRESOLD=700000000

PLAIN_TICKER=0x54455354
LEND_PREFIX=0x4c
BORROW_PREFIX=0x42

ISSUE_COST=50000000000000000

GAS_LIMIT=250000000

deploy() {
    erdpy contract deploy --project=${PROJECT} \
    --recall-nonce --pem=${PEM} --gas-limit=${GAS_LIMIT} --outfile="deploy.json" \
    --arguments ${ASSET} ${R_BASE} ${R_SLOPE1} ${R_SLOPE2} ${U_OPTIMAL} ${RESERVE_FACTOR} ${LIQ_THRESOLD} \
    --proxy=${PROXY} --chain=${CHAIN_ID} --send || return

    TRANSACTION=$(erdpy data parse --file="deploy.json" --expression="data['emitted_tx']['hash']")
    ADDRESS=$(erdpy data parse --file="deploy.json" --expression="data['emitted_tx']['address']")

    erdpy data store --key=address-testnet --value=${ADDRESS}
    erdpy data store --key=deployTransaction-testnet --value=${TRANSACTION}

    echo ""
    echo "Smart contract address: ${ADDRESS}"
}

deploy_dummy() {
    erdpy contract deploy --project=${PROJECT} \
    --recall-nonce --pem=${PEM} --gas-limit=${GAS_LIMIT} --outfile="deploy.json" \
    --arguments 0x4142432d653233383030 10 10 10 80 5 50 \
    --proxy=${PROXY} --chain=${CHAIN_ID} --send || return

    TRANSACTION=$(erdpy data parse --file="deploy.json" --expression="data['emitted_tx']['hash']")
    ADDRESS=$(erdpy data parse --file="deploy.json" --expression="data['emitted_tx']['address']")

    erdpy data store --key=dummy_address --value=${ADDRESS}
    erdpy data store --key=deployDummy-testnet --value=${TRANSACTION}

    echo ""
    echo "Smart contract address: ${ADDRESS}"
}

upgrade() {
    erdpy contract upgrade ${ADDRESS} \
    --project=${PROJECT} --recall-nonce --pem=${PEM} \
    --gas-limit=${GAS_LIMIT} --outfile="upgrade.json" \
    --arguments ${ASSET} ${R_BASE} ${R_SLOPE1} ${R_SLOPE2} ${U_OPTIMAL} ${RESERVE_FACTOR} ${LIQ_THRESOLD} \
    --proxy=${PROXY} --chain=${CHAIN_ID} --send || return
}

# SC calls

issue_lend() {
    erdpy contract call ${ADDRESS} \
    --recall-nonce --pem=${PEM} --gas-limit=${GAS_LIMIT} \
    --function="issue" --arguments ${PLAIN_TICKER} ${ASSET} ${LEND_PREFIX} \
    --value=${ISSUE_COST} --proxy=${PROXY} --chain=${CHAIN_ID} --send
}

issue_borrow() {
    erdpy contract call ${ADDRESS} \
    --recall-nonce --pem=${PEM} --gas-limit=${GAS_LIMIT} \
    --function="issue" --arguments ${PLAIN_TICKER} ${ASSET} ${BORROW_PREFIX} \
    --value=${ISSUE_COST} --proxy=${PROXY} --chain=${CHAIN_ID} --send
}

# Queries

get_lend_token() {
    erdpy contract query ${ADDRESS} --function="lendToken" --proxy=${PROXY}
}

get_borrow_token() {
    erdpy contract query ${ADDRESS} --function="borrowToken" --proxy=${PROXY}
}
