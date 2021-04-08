ALICE="../wallets/users/alice.pem"
ALICE_ADDRESS=0x0139472eff6886771a982f3083da5d421f24c29181e63888228dc81ca60d69e1

BOB="../wallets/users/bob.pem"
BOB_ADDRESS=0x8049d639e5a6980d1cd2392abcce41029cda74a1563523a202f09641cc2618f8

BUSD_POOL="../wallets/users/carol.pem"
BUSD_POOL_ADDRESS=0xb2a11555ce521e4944e09ab17549d85b487dcd26c84b5017a39e31a3670889ba


ADDRESS=$(erdpy data load --key=address-testnet)

DEPLOY_TRANSACTION=""

PROJECT="../../safety_module"

PROXY=https://testnet-gateway.elrond.com
CHAIN_ID=T

# issue an esdt token first
ESDT_TICKER=0x5745474c442d393933393334 # WEGLD-993934
ESDT_NAME=0x5745474c44 # WEGLD

ESDT_BUSD_TICKER=0x425553442d663333663764 #BUSD-f33f7d
ESDT_BUSD_NAME=42555344 #BUSD

FUND_ENDPOINT = 0x66756e64

issueWEGLD(){
    erdpy contract call erd1qqqqqqqqqqqqqqqpqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzllls8a5w6u --value=5000000000000000000 --pem=${BOB} --recall-nonce --gas-limit=1000000000 --proxy=${PROXY} --chain=${CHAIN_ID} --function=issue --outfile="issueWEGLD.json" --arguments 0X5745474c44 0X5745474c44 100000 10 --send
}

issueBUSD(){
    erdpy contract call erd1qqqqqqqqqqqqqqqpqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzllls8a5w6u --value=5000000000000000000 --pem=${BUSD_POOL} --recall-nonce --gas-limit=1000000000 --proxy=${PROXY} --chain=${CHAIN_ID} --function=issue --outfile="issueBUSD.json" --arguments 0X42555344 0X42555344 100000 10 --send
}

deploy(){
    erdpy contract deploy --project=${PROJECT} --pem=${ALICE} --arguments ${ESDT_TICKER} 3000000000 --proxy=${PROXY} --outfile="deploy.json" --recall-nonce --send --gas-limit="600000000" --chain ${CHAIN_ID} --send || return

    TRANSACTION=$(erdpy data parse --file="deploy.json" --expression="data['emitted_tx']['hash']")
    ADDRESS=$(erdpy data parse --file="deploy.json" --expression="data['emitted_tx']['address']")

    erdpy data store --key=address-testnet --value=${ADDRESS}
    erdpy data store --key=deployTransaction-testnet --value=${TRANSACTION}

    echo ""
    echo "Smart contract address: ${ADDRESS}"

}

upgrade(){
    erdpy contract upgrade ${ADDRESS} --project=${PROJECT} --pem=${ALICE} --arguments ${ESDT_TICKER} 3000000000 --proxy=${PROXY} --outfile="upgrade.json" --recall-nonce --send --gas-limit="600000000" --chain ${CHAIN_ID}
    echo ""
    echo "Smart contract address: ${ADDRESS}"
}

addPool(){
    echo "Smart contract address: ${ADDRESS}"
    erdpy contract call ${ADDRESS} --value=0 --pem=${ALICE} --recall-nonce --gas-limit=100000000 --proxy=${PROXY} --chain=${CHAIN_ID} --function=addPool --outfile="addPool.json" --arguments ${ESDT_BUSD_TICKER} ${BUSD_POOL_ADDRESS} --send
}

fundFromPool(){
    erdpy contract call ${ADDRESS} --pem=${BUSD} --recall-nonce --gas-limit=1000000000 --proxy=${PROXY} --chain=${CHAIN_ID} --function=ESDTTransfer --arguments 0x424f422d393863303365 20 0x66756e6446726f6d506f6f6c --send
}

NFTIssue(){
    erdpy contract call ${ADDRESS} --function=nftIssue --pem=${ALICE} --value=5000000000000000000 --arguments 0x4e4654 0x4e4654 --proxy=${PROXY} --outfile="NFTIssue.json" --recall-nonce --send --gas-limit="600000000" --chain ${CHAIN_ID} --send || return
}

getNFTTokenInfo(){
    erdpy contract query ${ADDRESS} --proxy=${PROXY} --function=nftToken
}

getWEGLTokenInfo(){
    erdpy contract query ${ADDRESS} --proxy=${PROXY} --function=wegld_token
}

fund(){
    erdpy contract call ${ADDRESS} --pem=${BOB} --recall-nonce --gas-limit=1000000000 --proxy=${PROXY} --chain=T --function=ESDTTransfer --arguments 0x5745474c442d393933393334 10 0x66756e64 --send
}

