ALICE="/home/boop/elrondsdk/sandbox/testnet/wallets/users/alice.pem"
ALICE_ADDRESS=0x0139472eff6886771a982f3083da5d421f24c29181e63888228dc81ca60d69e1
ALICE_BECH32=erd1qyu5wthldzr8wx5c9ucg8kjagg0jfs53s8nr3zpz3hypefsdd8ssycr6th

ADDRESS=$(erdpy data load --key=address-testnet)
DEPLOY_TRANSACTION=$(erdpy data load --key=deployTransaction-testnet)

PROXY=http://localhost:7950
CHAIN_ID=local-testnet

PROJECT="../../lending_pool"

AMOUNT=1000000000000000000 # 1 token (18 decimals)
DEPOSIT_ENDPOINT=0x6465706f736974 # deposit
RECEIVER=${ALICE}

ROUTER_ADDRESS=0x000000000000000005002218c5fd648b3d8eb68ff3282f162de38b296fe669e1

ESDT_TICKER=0x425553442d643338346436 

GAS_LIMIT=1200000000

deploy() {
    erdpy contract deploy --project=${PROJECT} --recall-nonce --pem=${ALICE} --gas-limit=${GAS_LIMIT} --outfile="deploy.json" --proxy=${PROXY} --chain=${CHAIN_ID} --send || return

    TRANSACTION=$(erdpy data parse --file="deploy.json" --expression="data['emitted_tx']['hash']")
    ADDRESS=$(erdpy data parse --file="deploy.json" --expression="data['emitted_tx']['address']")

    erdpy data store --key=address-testnet --value=${ADDRESS}
    erdpy data store --key=deployTransaction-testnet --value=${TRANSACTION}

    echo ""
    echo "Smart contract address: ${ADDRESS}"
}

upgrade() {
    erdpy contract upgrade ${ADDRESS} --project=${PROJECT} --recall-nonce --pem=${ALICE} --gas-limit=${GAS_LIMIT} --outfile="upgrade.json" --proxy=${PROXY} --chain=${CHAIN_ID} --send || return
}

# SC calls

deposit() {
    erdpy contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=${GAS_LIMIT} --function="ESDTTransfer" --arguments ${ESDT_TICKER} ${AMOUNT} ${DEPOSIT_ENDPOINT} --proxy=${PROXY} --chain=${CHAIN_ID} --send
}

withdraw() {
    erdpy contract call ${ADDRESS} --recal-nonce --pem=${ALICE} --gas-limit=${GAS_LIMIT} --function=
}

setRouter() {
    erdpy contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=${GAS_LIMIT} --function="setRouterAddress" --arguments ${ROUTER_ADDRESS} --proxy=${PROXY} --chain=${CHAIN_ID} --send
}
