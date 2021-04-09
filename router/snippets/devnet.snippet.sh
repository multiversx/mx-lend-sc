ALICE="$HOME/pems/dev.pem"

ADDRESS=$(erdpy data load --key=address-testnet)

PROXY=https://devnet-api.elrond.com
CHAIN_ID=D

PROJECT="../../router"

ISSUE_VALUE=5000000000000000000 # 5 EGLD

LEND_TICKER=0x
BORROW_TICKER=0x

QUERY_TOKEN_ID=0x

deploy() {
  erdpy contract deploy --project=${PROJECT} --recall-nonce --pem="${ALICE}" --gas-limit=150000000 --outfile="deploy.json" --proxy=${PROXY} --chain=${CHAIN_ID} --send || return

  TRANSACTION=$(erdpy data parse --file="deploy.json" --expression="data['emitted_tx']['hash']")
  ADDRESS=$(erdpy data parse --file="deploy.json" --expression="data['emitted_tx']['address']")

  erdpy data store --key=address-testnet --value="${ADDRESS}"
  erdpy data store --key=deployTransaction-testnet --value="${TRANSACTION}"

  echo ""
  echo "Smart contract address: ${ADDRESS}"
}

upgrade() {
  erdpy contract upgrade "${ADDRESS}" --project=${PROJECT} --recall-nonce --pem="${ALICE}" --gas-limit=150000000 --outfile="upgrade.json" --proxy=${PROXY} --chain=${CHAIN_ID} --send || return
}

# SC calls

issueLendToken() {
  erdpy contract call "${ADDRESS}" --recall-nonce --pem="${ALICE}" --gas-limit=150000000 --value="${ISSUE_VALUE}" --function="issueLendToken" --arguments ${LEND_TICKER} --proxy=${PROXY} --chain=${CHAIN_ID} --send
}

issueBorrowToken() {
  erdpy contract call "${ADDRESS}" --recall-nonce --pem="${ALICE}" --gas-limit=150000000 --value="${ISSUE_VALUE}" --function="issueBorrowToken" --arguments ${BORROW_TICKER} --proxy=${PROXY} --chain=${CHAIN_ID} --send
}

# Queries

getPoolAddress() {
  erdpy contract query "${ADDRESS}" --function="getPoolAddress" --arguments ${QUERY_TOKEN_ID} --proxy="${PROXY}"
}
