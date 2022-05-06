import path from "path";
import { AddressValue, BigIntValue, CodeMetadata, IAddress, Interaction, ResultsParser, ReturnCode, SmartContract, SmartContractAbi, TokenIdentifierValue, TokenPayment, TransactionWatcher } from "@elrondnetwork/erdjs";
import { INetworkProvider, ITestSession, ITestUser, loadAbiRegistry, loadCode } from "@elrondnetwork/erdjs-snippets";
import { NetworkConfig } from "@elrondnetwork/erdjs-network-providers";

const PathToWasm = path.resolve(__dirname, "..", "..", "lending_pool", "output", "lending-pool.wasm");
const PathToAbi = path.resolve(__dirname, "..", "..", "lending_pool", "output", "lending-pool.abi.json");
const PathToWasmDummyLiquidityPool = path.resolve(__dirname, "..", "..", "liquidity_pool", "output", "liquidity-pool.wasm");

export async function createLendingInteractor(session: ITestSession, contractAddress?: IAddress): Promise<LendingPoolInteractor> {
    let registry = await loadAbiRegistry(PathToAbi);
    let abi = new SmartContractAbi(registry, ["LendingPool"]);
    let contract = new SmartContract({ address: contractAddress, abi: abi});
    let networkProvider = session.networkProvider;
    let networkConfig = session.getNetworkConfig();

    let interactor = new LendingPoolInteractor(contract, networkProvider, networkConfig);
    return interactor;
}

export class LendingPoolInteractor {
    private readonly contract: SmartContract;
    private readonly networkProvider: INetworkProvider;
    private readonly networkConfig: NetworkConfig;
    private readonly transactionWatcher: TransactionWatcher;
    private readonly resultsParser: ResultsParser;

    constructor(contract: SmartContract, networkProvider: INetworkProvider, networkConfig: NetworkConfig) {
        this.contract = contract;
        this.networkProvider = networkProvider;
        this.networkConfig = networkConfig;
        this.transactionWatcher = new TransactionWatcher(networkProvider);
        this.resultsParser = new ResultsParser();
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
        let address = SmartContract.computeAddress(transaction.getSender(), transaction.getNonce());

        // Let's broadcast the transaction and await its completion:
        await this.networkProvider.sendTransaction(transaction);
        let transactionOnNetwork = await this.transactionWatcher.awaitCompleted(transaction);

        // In the end, parse the results:
        let { returnCode } = this.resultsParser.parseUntypedOutcome(transactionOnNetwork);

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
        console.log(`LendingPoolInteractor.issueBorrow(): address = ${user.address}`);

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
        console.log(`LendingPoolInteractor.issueBorrow(): address = ${user.address}`);

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
        console.log(`LendingPoolInteractor.setAssetLoanToValue(): address = ${user.address}`);

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
        console.log(`LendingPoolInteractor.setAssetLiquidationBonus(): address = ${user.address}`);

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
        console.log(`LendingPoolInteractor.deposit(): address = ${user.address}, amount = ${tokenPayment.toPrettyString()}`);

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

        console.log(`LendingPoolInteractor.deposit(): contract = ${this.contract.getAddress()} Received SDT with nonce = ${returnCode} ${firstValue}`);
        let depositNonce = firstValue!.valueOf();

        return { returnCode, depositNonce };
    }


    async withdraw(user: ITestUser, tokenPayment: TokenPayment): Promise<ReturnCode> {
        console.log(`LendingPoolInteractor.deposit(): address = ${user.address}, amount = ${tokenPayment.toPrettyString()}`);

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
        let { returnCode, firstValue } = this.resultsParser.parseOutcome(transactionOnNetwork, interaction.getEndpoint());

        console.log(`LendingPoolInteractor.withdraw(): contract = ${this.contract.getAddress()}  returnCode = ${returnCode} Received metaESDT with nonce = ${firstValue}`);

        return returnCode;
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

}
