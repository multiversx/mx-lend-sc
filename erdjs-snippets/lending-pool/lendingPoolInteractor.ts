import path from "path";
import { AddressValue, BigIntValue, CodeMetadata, IAddress, Interaction, ResultsParser, ReturnCode, SmartContract, SmartContractAbi, Struct, TokenIdentifierValue, TokenPayment, TransactionWatcher } from "@elrondnetwork/erdjs";
import { IAudit, INetworkConfig, INetworkProvider, ITestSession, ITestUser, loadAbiRegistry, loadCode } from "@elrondnetwork/erdjs-snippets";

const PathToWasm = path.resolve(__dirname, "..", "..", "lending_pool", "output", "lending-pool.wasm");
const PathToAbi = path.resolve(__dirname, "..", "..", "lending_pool", "output", "lending-pool.abi.json");
const PathToWasmDummyLiquidityPool = path.resolve(__dirname, "..", "..", "liquidity_pool", "output", "liquidity-pool.wasm");

export async function createLendingInteractor(session: ITestSession, contractAddress?: IAddress): Promise<LendingPoolInteractor> {
    const registry = await loadAbiRegistry(PathToAbi);
    const abi = new SmartContractAbi(registry);
    const contract = new SmartContract({ address: contractAddress, abi: abi});
    const networkProvider = session.networkProvider;
    const networkConfig = session.getNetworkConfig();
    const audit = session.audit;
    let interactor = new LendingPoolInteractor(contract, networkProvider, networkConfig, audit);
    return interactor;
}

export class LendingPoolInteractor {
    private readonly contract: SmartContract;
    private readonly networkProvider: INetworkProvider;
    private readonly networkConfig: INetworkConfig;
    private readonly transactionWatcher: TransactionWatcher;
    private readonly resultsParser: ResultsParser;
    private readonly audit: IAudit;

    constructor(contract: SmartContract, networkProvider: INetworkProvider, networkConfig: INetworkConfig, audit: IAudit) {
        this.contract = contract;
        this.networkProvider = networkProvider;
        this.networkConfig = networkConfig;
        this.transactionWatcher = new TransactionWatcher(networkProvider);
        this.resultsParser = new ResultsParser();
        this.audit = audit;
    }

    async deployDummyLiquidityPool(deployer: ITestUser, tokenIdentifier: string): Promise<{ address: IAddress, returnCode: ReturnCode }> {
        // Load the bytecode from a file.
        let code = await loadCode(PathToWasmDummyLiquidityPool);

        // Prepare the deploy transaction.
        let transaction = this.contract.deploy({
            code: code,
            codeMetadata: new CodeMetadata(),
            initArguments: [
                new TokenIdentifierValue(tokenIdentifier),
                new BigIntValue(0),
                new BigIntValue(40000000),
                new BigIntValue(1000000000),
                new BigIntValue(800000000),
                new BigIntValue(100000000),
                new BigIntValue(700000000)
            ],
            gasLimit: 100000000,
            chainID: this.networkConfig.ChainID
        });

        // Set the transaction nonce. The account nonce must be synchronized beforehand.
        // Also, locally increment the nonce of the deployer (optional).
        transaction.setNonce(deployer.account.getNonceThenIncrement());

        // Let's sign the transaction. For dApps, use a wallet provider instead.
        await deployer.signer.sign(transaction);

        // The contract address is deterministically computable:
        const address = SmartContract.computeAddress(transaction.getSender(), transaction.getNonce());

        const transactionHash = await this.networkProvider.sendTransaction(transaction);
        await this.audit.onContractDeploymentSent({ transactionHash: transactionHash, contractAddress: address });

        let transactionOnNetwork = await this.transactionWatcher.awaitCompleted(transaction);
        await this.audit.onTransactionCompleted({ transactionHash: transactionHash, transaction: transactionOnNetwork });

        // In the end, parse the results:
        const { returnCode } = this.resultsParser.parseUntypedOutcome(transactionOnNetwork);

        console.log(`LendingPoolInteractor.deployDummyLiquidityPool(): contract = ${address}`);
        return { address, returnCode };
    }

    async deploy(deployer: ITestUser, dummyAddress: IAddress): Promise<{ address: IAddress, returnCode: ReturnCode }> {
        // Load the bytecode from a file.
        let code = await loadCode(PathToWasm);

        // Prepare the deploy transaction.
        let transaction = this.contract.deploy({
            code: code,
            codeMetadata: new CodeMetadata(),
            initArguments: [
                new AddressValue(dummyAddress)
            ],
            gasLimit: 100000000,
            chainID: this.networkConfig.ChainID
        });

        // Set the transaction nonce. The account nonce must be synchronized beforehand.
        // Also, locally increment the nonce of the deployer (optional).
        transaction.setNonce(deployer.account.getNonceThenIncrement());

        // Let's sign the transaction. For dApps, use a wallet provider instead.
        await deployer.signer.sign(transaction);

        // The contract address is deterministically computable:
        let address = SmartContract.computeAddress(transaction.getSender(), transaction.getNonce());
        this.contract.setAddress(address);

        // Let's broadcast the transaction and await its completion:
        await this.networkProvider.sendTransaction(transaction);
        let transactionOnNetwork = await this.transactionWatcher.awaitCompleted(transaction);

        // In the end, parse the results:
        let { returnCode } = this.resultsParser.parseUntypedOutcome(transactionOnNetwork);

        console.log(`LendingPoolInteractor.deploy(): contract = ${address}`);
        return { address, returnCode };
    }

    async addLiquidityPool(user: ITestUser, tokenIdentifier: string, R_BASE: number, R_SLOPE1: number, R_SLOPE2: number, U_OPTIMAL: number, RESERVE_FACTOR: number, LIQ_THRESHOLD: number): Promise<ReturnCode> {
        console.log(`LendingPoolInteractor.addLiquidityPool(): address = ${user.address}`);

        // Prepare the interaction
        let interaction = <Interaction>this.contract.methods
            .createLiquidityPool([tokenIdentifier, R_BASE, R_SLOPE1, R_SLOPE2, U_OPTIMAL, RESERVE_FACTOR, LIQ_THRESHOLD])
            .withGasLimit(50000000)
            .withNonce(user.account.getNonceThenIncrement())
            .withChainID(this.networkConfig.ChainID);


        // Let's check the interaction, then build the transaction object.
        let transaction = interaction.check().buildTransaction();

        // Let's sign the transaction. For dApps, use a wallet provider instead.
        await user.signer.sign(transaction);

        // Let's broadcast the transaction and await its completion:
        await this.networkProvider.sendTransaction(transaction);
        let transactionOnNetwork = await this.transactionWatcher.awaitCompleted(transaction);

        // In the end, parse the results:
        let { returnCode } = this.resultsParser.parseOutcome(transactionOnNetwork, interaction.getEndpoint());
        console.log(`LendingPoolInteractor.addLiquidityPool(): contract = ${this.contract.getAddress()}`);

        return returnCode;
    }


    async issueLend(user: ITestUser, tokenIdentifier: string): Promise<ReturnCode> {
        console.log(`LendingPoolInteractor.issueLend(): address = ${user.address}`);

        // Prepare the interaction
        let interaction = <Interaction>this.contract.methods
            .issueLendToken([tokenIdentifier, tokenIdentifier.split("-")[0]])
            .withGasLimit(120000000)
            .withValue(TokenPayment.egldFromAmount(0.05))
            .withNonce(user.account.getNonceThenIncrement())
            .withChainID(this.networkConfig.ChainID);

        // Let's check the interaction, then build the transaction object.
        let transaction = interaction.check().buildTransaction();

        // Let's sign the transaction. For dApps, use a wallet provider instead.
        await user.signer.sign(transaction);

        // Let's broadcast the transaction and await its completion:
        await this.networkProvider.sendTransaction(transaction);
        let transactionOnNetwork = await this.transactionWatcher.awaitCompleted(transaction);

        // In the end, parse the results:
        let { returnCode } = this.resultsParser.parseOutcome(transactionOnNetwork, interaction.getEndpoint());
        return returnCode;
    }

    async issueBorrow(user: ITestUser, tokenIdentifier: string): Promise<ReturnCode> {
        console.log(`LendingPoolInteractor.issueBorrow(): address = ${user.address}`);

        // Prepare the interaction
        let interaction = <Interaction>this.contract.methods
            .issueBorrowToken([tokenIdentifier, tokenIdentifier.split("-")[0]])
            .withValue(TokenPayment.egldFromAmount(0.05))
            .withGasLimit(120000000)
            .withNonce(user.account.getNonceThenIncrement())
            .withChainID(this.networkConfig.ChainID);

        // Let's check the interaction, then build the transaction object.
        let transaction = interaction.check().buildTransaction();

        // Let's sign the transaction. For dApps, use a wallet provider instead.
        await user.signer.sign(transaction);

        // Let's broadcast the transaction and await its completion:
        await this.networkProvider.sendTransaction(transaction);
        let transactionOnNetwork = await this.transactionWatcher.awaitCompleted(transaction);

        // In the end, parse the results:
        let { returnCode } = this.resultsParser.parseOutcome(transactionOnNetwork, interaction.getEndpoint());
        return returnCode;
    }


    async setLendRoles(user: ITestUser, tokenIdentifier: string): Promise<ReturnCode> {
        // Prepare the interaction
        let interaction = <Interaction>this.contract.methods
            .setLendRoles([tokenIdentifier])
            .withGasLimit(100000000)
            .withNonce(user.account.getNonceThenIncrement())
            .withChainID(this.networkConfig.ChainID);

        // Let's check the interaction, then build the transaction object.
        let transaction = interaction.check().buildTransaction();

        // Let's sign the transaction. For dApps, use a wallet provider instead.
        await user.signer.sign(transaction);

        // Let's broadcast the transaction and await its completion:
        await this.networkProvider.sendTransaction(transaction);
        let transactionOnNetwork = await this.transactionWatcher.awaitCompleted(transaction);

        // In the end, parse the results:
        let { returnCode } = this.resultsParser.parseOutcome(transactionOnNetwork, interaction.getEndpoint());
        return returnCode;
    }


    async setBorrowRoles(user: ITestUser, tokenIdentifier: string): Promise<ReturnCode> {
        // Prepare the interaction
        let interaction = <Interaction>this.contract.methods
            .setBorrowRoles([tokenIdentifier])
            .withGasLimit(100000000)
            .withNonce(user.account.getNonceThenIncrement())
            .withChainID(this.networkConfig.ChainID);

        // Let's check the interaction, then build the transaction object.
        let transaction = interaction.check().buildTransaction();

        // Let's sign the transaction. For dApps, use a wallet provider instead.
        await user.signer.sign(transaction);

        // Let's broadcast the transaction and await its completion:
        await this.networkProvider.sendTransaction(transaction);
        let transactionOnNetwork = await this.transactionWatcher.awaitCompleted(transaction);

        // In the end, parse the results:
        let { returnCode } = this.resultsParser.parseOutcome(transactionOnNetwork, interaction.getEndpoint());
        return returnCode;
    }

    async setAssetLoanToValue(user: ITestUser, tokenIdentifier: string, ltv: number): Promise<ReturnCode> {
        console.log(`LendingPoolInteractor.setAssetLoanToValue(): tokenIdentifier = ${tokenIdentifier} to ltv = ${ltv}`);

        // Prepare the interaction
        let interaction = <Interaction>this.contract.methods
            .setAssetLoanToValue([tokenIdentifier, ltv])
            .withGasLimit(8000000)
            .withNonce(user.account.getNonceThenIncrement())
            .withChainID(this.networkConfig.ChainID);

        // Let's check the interaction, then build the transaction object.
        let transaction = interaction.check().buildTransaction();

        // Let's sign the transaction. For dApps, use a wallet provider instead.
        await user.signer.sign(transaction);

        // Let's broadcast the transaction and await its completion:
        await this.networkProvider.sendTransaction(transaction);
        let transactionOnNetwork = await this.transactionWatcher.awaitCompleted(transaction);

        // In the end, parse the results:
        let { returnCode } = this.resultsParser.parseOutcome(transactionOnNetwork, interaction.getEndpoint());
        return returnCode;
    }

    async setAssetLiquidationBonus(user: ITestUser, tokenIdentifier: string, liqBonus: number): Promise<ReturnCode> {
        console.log(`LendingPoolInteractor.setAssetLiquidationBonus(): tokenIdentifier = ${tokenIdentifier}, liqBonus = ${liqBonus}`);

        // Prepare the interaction
        let interaction = <Interaction>this.contract.methods
            .setAssetLiquidationBonus([tokenIdentifier, liqBonus])
            .withGasLimit(8000000)
            .withNonce(user.account.getNonceThenIncrement())
            .withChainID(this.networkConfig.ChainID);

        // Let's check the interaction, then build the transaction object.
        let transaction = interaction.check().buildTransaction();

        // Let's sign the transaction. For dApps, use a wallet provider instead.
        await user.signer.sign(transaction);

        // Let's broadcast the transaction and await its completion:
        await this.networkProvider.sendTransaction(transaction);
        let transactionOnNetwork = await this.transactionWatcher.awaitCompleted(transaction);

        // In the end, parse the results:
        let { returnCode } = this.resultsParser.parseOutcome(transactionOnNetwork, interaction.getEndpoint());
        return returnCode;
    }

    async deposit(user: ITestUser, tokenPayment: TokenPayment): Promise<{ returnCode: ReturnCode, depositNonce: number }> {
        console.log(`LendingPoolInteractor.deposit(): user = ${user.address}, Token = ${tokenPayment.toPrettyString()}`);

        // Prepare the interaction
        let interaction = <Interaction>this.contract.methods
            .deposit([])
            .withGasLimit(15000000)
            .withSingleESDTTransfer(tokenPayment)
            .withNonce(user.account.getNonceThenIncrement())
            .withChainID(this.networkConfig.ChainID);

        // Let's check the interaction, then build the transaction object.
        let transaction = interaction.check().buildTransaction();

        // Let's sign the transaction. For dApps, use a wallet provider instead.
        await user.signer.sign(transaction);

        // Let's broadcast the transaction and await its completion:
        await this.networkProvider.sendTransaction(transaction);
        let transactionOnNetwork = await this.transactionWatcher.awaitCompleted(transaction);

        // In the end, parse the results:
        let { returnCode, firstValue } = this.resultsParser.parseOutcome(transactionOnNetwork, interaction.getEndpoint());

        let depositNonce: number = +(<Struct>firstValue).getFieldValue("token_nonce")["c"][0];

        return { returnCode, depositNonce };
    }


    async withdraw(user: ITestUser, tokenPayment: TokenPayment): Promise<ReturnCode> {
        console.log(`LendingPoolInteractor.deposit(): user = ${user.address}, Token = ${tokenPayment.toPrettyString()}`);

        // Prepare the interaction
        let interaction = <Interaction>this.contract.methods
            .withdraw([])
            .withGasLimit(15000000)
            .withSingleESDTNFTTransfer(tokenPayment, user.address)
            .withNonce(user.account.getNonceThenIncrement())
            .withChainID(this.networkConfig.ChainID);

        // Let's check the interaction, then build the transaction object.
        let transaction = interaction.check().buildTransaction();

        // Let's sign the transaction. For dApps, use a wallet provider instead.
        await user.signer.sign(transaction);

        // Let's broadcast the transaction and await its completion:
        await this.networkProvider.sendTransaction(transaction);
        let transactionOnNetwork = await this.transactionWatcher.awaitCompleted(transaction);

        // In the end, parse the results:
        let { returnCode } = this.resultsParser.parseOutcome(transactionOnNetwork, interaction.getEndpoint());
        return returnCode;
    }

    async borrow(user: ITestUser, collateralPayment: TokenPayment, assetToBorrow: string): Promise<{ returnCode: ReturnCode, borrowNonce: number }> {
        console.log(`LendingPoolInteractor.borrow(): borrower = ${user.address}, collateralToken = ${collateralPayment.toPrettyString()}, 
                    nonce = ${collateralPayment.nonce} assetToBorrow = ${assetToBorrow}`);

        // Prepare the interaction
        let interaction = <Interaction>this.contract.methods
            .borrow([assetToBorrow])
            .withGasLimit(150000000)
            .withSingleESDTNFTTransfer(collateralPayment, user.address)
            .withNonce(user.account.getNonceThenIncrement())
            .withChainID(this.networkConfig.ChainID);

        // Let's check the interaction, then build the transaction object.
        let transaction = interaction.check().buildTransaction();

        // Let's sign the transaction. For dApps, use a wallet provider instead.
        await user.signer.sign(transaction);

        // Let's broadcast the transaction and await its completion:
        await this.networkProvider.sendTransaction(transaction);
        let transactionOnNetwork = await this.transactionWatcher.awaitCompleted(transaction);

        // In the end, parse the results:
        let { returnCode, firstValue } = this.resultsParser.parseOutcome(transactionOnNetwork, interaction.getEndpoint());

        let borrowNonce: number = +(<Struct>firstValue).getFieldValue("token_nonce")["c"][0];
        return { returnCode, borrowNonce };
    }

    async getLiquidityAddress(tokenIdentifier: string): Promise<IAddress> {
        // Prepare the interaction, check it, then build the query:
        let interaction = <Interaction>this.contract.methods.getPoolAddress([tokenIdentifier]);

        let query = interaction.check().buildQuery();

        // Let's run the query and parse the results:
        let queryResponse = await this.networkProvider.queryContract(query);
        let { firstValue } = this.resultsParser.parseQueryResponse(queryResponse, interaction.getEndpoint());

        // Now let's interpret the results.
        return firstValue!.valueOf();
    }

    async getAssetLoanToValue(tokenIdentifier: string): Promise<IAddress> {
        // Prepare the interaction, check it, then build the query:
        let interaction = <Interaction>this.contract.methods.getAssetLoanToValue([tokenIdentifier]);

        let query = interaction.check().buildQuery();

        // Let's run the query and parse the results:
        let queryResponse = await this.networkProvider.queryContract(query);
        let { firstValue } = this.resultsParser.parseQueryResponse(queryResponse, interaction.getEndpoint());

        console.log(`Asset ${tokenIdentifier} are LTV ${firstValue!.valueOf()}`)
        // Now let's interpret the results.
        return firstValue!.valueOf();
    }


    async setAggregator(user: ITestUser, poolAsset: string, priceAggregatorAddress: IAddress) {
        console.log(`LiquidityInteractor.setPriceAggregatorAddress(): priceAggregatorAddres = ${priceAggregatorAddress}, poolAsset = ${poolAsset}`);

        // Prepare the interaction
        let interaction = <Interaction>this.contract.methods
            .setAggregator([poolAsset, priceAggregatorAddress])
            .withGasLimit(10000000)
            .withNonce(user.account.getNonceThenIncrement())
            .withChainID(this.networkConfig.ChainID);

        // Let's check the interaction, then build the transaction object.
        let transaction = interaction.check().buildTransaction();

        // Let's sign the transaction. For dApps, use a wallet provider instead.
        await user.signer.sign(transaction);

        // Let's broadcast the transaction and await its completion:
        await this.networkProvider.sendTransaction(transaction);
        let transactionOnNetwork = await this.transactionWatcher.awaitCompleted(transaction);

        // In the end, parse the results:
        let { returnCode } = this.resultsParser.parseOutcome(transactionOnNetwork, interaction.getEndpoint());
        return returnCode;
    }

}