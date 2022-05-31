import path from "path";
import { AddressValue, BigIntValue, CodeMetadata, IAddress, Interaction, ResultsParser, ReturnCode, SmartContract, SmartContractAbi, TransactionWatcher } from "@elrondnetwork/erdjs";
import { INetworkProvider, ITestSession, ITestUser, loadAbiRegistry, loadCode } from "@elrondnetwork/erdjs-snippets";
import { NetworkConfig } from "@elrondnetwork/erdjs-network-providers";

const PathToPriceAggregatorAbi = path.resolve(__dirname, "..", "..", "price_aggregator", "output", "price-aggregator.abi.json");
const PathToPriceAggregator = path.resolve(__dirname, "..", "..", "price_aggregator", "output", "price-aggregator.wasm");

export async function createPriceAggregatorInteractor(session: ITestSession, contractAddress?: IAddress): Promise<PriceAggregatorInteractor> {
    let registry = await loadAbiRegistry(PathToPriceAggregatorAbi);
    let abi = new SmartContractAbi(registry);
    let contract = new SmartContract({ address: contractAddress, abi: abi });
    let networkProvider = session.networkProvider;
    let networkConfig = session.getNetworkConfig();

    let interactor = new PriceAggregatorInteractor(contract, networkProvider, networkConfig);
    return interactor;
}

export class PriceAggregatorInteractor {
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

    async deployAggregator(deployer: ITestUser): Promise<{ address: IAddress, returnCode: ReturnCode }> {
        // Load the bytecode from a file.
        let code = await loadCode(PathToPriceAggregator);

        // Prepare the deploy transaction.
        let transaction = this.contract.deploy({
            code: code,
            codeMetadata: new CodeMetadata(),
            initArguments: [
                new BigIntValue(1),
                new BigIntValue(0),
                new AddressValue(deployer.address)
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

        console.log(`PriceAggregatorInteractor.deployAggregator(): contract = ${address}`);
        return { address, returnCode };
    }

    async unpausePoolAggregator(user: ITestUser) {
        // Prepare the interaction
        let interaction = <Interaction>this.contract.methods
            .unpause([])
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

    async submitPriceAggregator(user: ITestUser, from: string, to: string, price: number) {
        console.log(`PriceAggregatorInteractor.submitPriceAggregator(): address = ${user.address}, from = ${from}, to = ${to}, price = ${price}`);

        // Prepare the interaction
        let interaction = <Interaction>this.contract.methods
            .submit([from, to, price])
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
    }

    async latestPriceFeed(from: string, to: string) {
        // Prepare the interaction, check it, then build the query:
        let interaction = <Interaction>this.contract.methods.latestPriceFeed([from, to]);
        let query = interaction.check().buildQuery();

        // Let's run the query and parse the results:
        await this.networkProvider.queryContract(query);
        let queryResponse = await this.networkProvider.queryContract(query);
        let { values } = this.resultsParser.parseQueryResponse(queryResponse, interaction.getEndpoint());

        console.log(`New price for ${from}, is ${values}`);
    }
}
