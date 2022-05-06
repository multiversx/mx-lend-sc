import path from "path";
import { IAddress, Interaction, ResultsParser, SmartContract, SmartContractAbi, TransactionWatcher } from "@elrondnetwork/erdjs";
import { INetworkProvider, ITestSession, loadAbiRegistry } from "@elrondnetwork/erdjs-snippets";
import { NetworkConfig } from "@elrondnetwork/erdjs-network-providers";

const PathToLiquidityAbi = path.resolve(__dirname, "..", "..", "liquidity_pool", "output", "liquidity-pool.abi.json");

export async function createLiquidityInteractor(session: ITestSession, contractAddress?: IAddress): Promise<LiquidityPoolInteractor> {
    let registry = await loadAbiRegistry(PathToLiquidityAbi);
    let abi = new SmartContractAbi(registry, ["LiquidityPool"]);
    let contract = new SmartContract({ address: contractAddress, abi: abi });
    let networkProvider = session.networkProvider;
    let networkConfig = session.getNetworkConfig();

    let interactor = new LiquidityPoolInteractor(contract, networkProvider, networkConfig);
    return interactor;
}

export class LiquidityPoolInteractor {
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

    async getLendingToken(): Promise<string> {
        // Prepare the interaction, check it, then build the query:
        let interaction = <Interaction>this.contract.methods.getLendToken();
        let query = interaction.check().buildQuery();

        // Let's run the query and parse the results:
        await this.networkProvider.queryContract(query);
        let queryResponse = await this.networkProvider.queryContract(query);
        let { firstValue } = this.resultsParser.parseQueryResponse(queryResponse, interaction.getEndpoint());

        // Now let's interpret the results.
        return firstValue!.valueOf().toString();
    }

}
