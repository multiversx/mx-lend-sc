import path from "path";
import { AddressValue, BigIntValue, CodeMetadata, IAddress, Interaction, ResultsParser, ReturnCode, SmartContract, SmartContractAbi, Struct, TokenIdentifierValue, TokenPayment, TransactionWatcher } from "@elrondnetwork/erdjs";
import { INetworkProvider, ITestSession, ITestUser, loadAbiRegistry, loadCode } from "@elrondnetwork/erdjs-snippets";
import { NetworkConfig } from "@elrondnetwork/erdjs-network-providers";

const PathToWasm = path.resolve(__dirname, "..", "..", "lending_pool", "output", "lending-pool.wasm");
const PathToAbi = path.resolve(__dirname, "..", "..", "lending_pool", "output", "lending-pool.abi.json");
const PathToWasmDummyLiquidityPool = path.resolve(__dirname, "..", "..", "liquidity_pool", "output", "liquidity-pool.wasm");

export async function createInteractor(session: ITestSession, contractAddress?: IAddress): Promise<LendingPoolInteractor> {
    let registry = await loadAbiRegistry(PathToAbi);
    let abi = new SmartContractAbi(registry, ["LendingPool"]);
    let contract = new SmartContract({ address: contractAddress, abi: abi });
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
            gasLimit: 40000000,
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
            gasLimit: 60000000,
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

        console.log(`LendingPoolInteractor.deploy(): contract = ${address}`);
        return { address, returnCode };
    }

    async deposit(user: ITestUser, amount: TokenPayment): Promise<ReturnCode> {
        console.log(`LendingPoolInteractor.deposit(): address = ${user.address}, amount = ${amount.toPrettyString()}`);

        // Prepare the interaction
        let interaction = <Interaction>this.contract.methods
            .deposit([])
            .withGasLimit(30000000)
            .withSingleESDTTransfer(amount)
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
