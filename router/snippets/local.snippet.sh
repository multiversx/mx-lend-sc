ALICE="$HOME/pems/local.pem"

ADDRESS=$(erdpy data load --key=address-testnet)

PROXY=http://localhost:7950
CHAIN_ID=local-testnet

PROJECT="../../router"

ISSUE_VALUE=5000000000000000000 # 5 EGLD

TICKER=0x425553442d393163643331
TICKER_PLAIN=0x42555344

QUERY_TOKEN_ID=0x4c425553442d653462336362

GAS_LIMIT=250000000

forceStoreAddr() {
  erdpy data store --key=address-testnet --value="$1" 
}

deploy() {
  erdpy contract deploy --project=${PROJECT} --recall-nonce --pem="${ALICE}" --gas-limit=${GAS_LIMIT} --outfile="deploy.json" --proxy=${PROXY} --chain=${CHAIN_ID} --send || return

  TRANSACTION=$(erdpy data parse --file="deploy.json" --expression="data['emitted_tx']['hash']")
  ADDRESS=$(erdpy data parse --file="deploy.json" --expression="data['emitted_tx']['address']")

  erdpy data store --key=address-testnet --value="${ADDRESS}"
  erdpy data store --key=deployTransaction-testnet --value="${TRANSACTION}"

  echo ""
  echo "Smart contract address: ${ADDRESS}"
}

upgrade() {
  erdpy contract upgrade "${ADDRESS}" --project=${PROJECT} --recall-nonce --pem="${ALICE}" --gas-limit=${GAS_LIMIT} --outfile="upgrade.json" --proxy=${PROXY} --chain=${CHAIN_ID} --send || return
}

# SC calls

issueLendToken() {
  erdpy contract call "${ADDRESS}" --recall-nonce --pem="${ALICE}" --gas-limit=${GAS_LIMIT} --value="${ISSUE_VALUE}" --function="issueLendToken" --arguments ${TICKER_PLAIN} ${TICKER} --proxy=${PROXY} --chain=${CHAIN_ID} --send
}

issueBorrowToken() {
  erdpy contract call "${ADDRESS}" --recall-nonce --pem="${ALICE}" --gas-limit=${GAS_LIMIT} --value="${ISSUE_VALUE}" --function="issueBorrowToken" --arguments ${TICKER_PLAIN} ${TICKER} --proxy=${PROXY} --chain=${CHAIN_ID} --send
}

setLendRoles() {
  erdpy contract call "${ADDRESS}" --recall-nonce --pem="${ALICE}" --gas-limit=${GAS_LIMIT} --function="setLendRoles" --arguments 0x03 0x04 0x05 --proxy=${PROXY} --chain=${CHAIN_ID} --send
}

setBorrowRoles() {
  erdpy contract call "${ADDRESS}" --recall-nonce --pem="${ALICE}" --gas-limit=${GAS_LIMIT} --function="setBorrowRoles" --arguments 0x03 0x04 0x05 --proxy=${PROXY} --chain=${CHAIN_ID} --send
}

# Queries

getPoolAddress() {
  erdpy contract query "${ADDRESS}" --function="getPoolAddress" --arguments ${QUERY_TOKEN_ID} --proxy="${PROXY}"
}
