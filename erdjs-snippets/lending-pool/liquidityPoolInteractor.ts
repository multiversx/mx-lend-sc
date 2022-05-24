import path from "path";
import { IAddress, Interaction, ResultsParser, SmartContract, SmartContractAbi, TransactionWatcher } from "@elrondnetwork/erdjs";
<<<<<<< HEAD
import { IAudit, INetworkConfig, INetworkProvider, ITestSession, ITestUser, loadAbiRegistry, loadCode } from "@elrondnetwork/erdjs-snippets";
=======
import { INetworkProvider, ITestSession, ITestUser, loadAbiRegistry } from "@elrondnetwork/erdjs-snippets";
>>>>>>> Add helper functions to Interactors
import { NetworkConfig } from "@elrondnetwork/erdjs-network-providers";

const PathToLiquidityAbi = path.resolve(__dirname, "..", "..", "liquidity_pool", "output", "liquidity-pool.abi.json");

export async function createLiquidityInteractor(session: ITestSession, contractAddress?: IAddress): Promise<LiquidityPoolInteractor> {
    const registry = await loadAbiRegistry(PathToLiquidityAbi);
    const abi = new SmartContractAbi(registry, ["LiquidityPool"]);
    const contract = new SmartContract({ address: contractAddress, abi: abi });
    const networkProvider = session.networkProvider;
    const networkConfig = session.getNetworkConfig();
    const audit = session.audit;
    const interactor = new LiquidityPoolInteractor(contract, networkProvider, networkConfig, audit);
    return interactor;
}

export class LiquidityPoolInteractor {
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

    async getPoolAsset(): Promise<string> {
        // Prepare the interaction, check it, then build the query:
        let interaction = <Interaction>this.contract.methods.getPoolAsset();
        let query = interaction.check().buildQuery();

        // Let's run the query and parse the results:
        await this.networkProvider.queryContract(query);
        let queryResponse = await this.networkProvider.queryContract(query);
        let { firstValue } = this.resultsParser.parseQueryResponse(queryResponse, interaction.getEndpoint());

        // Now let's interpret the results.
        return firstValue!.valueOf().toString();
    }

    async getPoolParams(): Promise<string> {
        // Prepare the interaction, check it, then build the query:
        let interaction = <Interaction>this.contract.methods.getPoolParams();
        let query = interaction.check().buildQuery();

        // Let's run the query and parse the results:
        await this.networkProvider.queryContract(query);
        let queryResponse = await this.networkProvider.queryContract(query);
        let { firstValue } = this.resultsParser.parseQueryResponse(queryResponse, interaction.getEndpoint());

        // Now let's interpret the results.
        return firstValue!.valueOf().toString();
    }


    async getAggregatorAddress(): Promise<string> {
        // Prepare the interaction, check it, then build the query:
        let interaction = <Interaction>this.contract.methods.getAggregatorAddress();
        let query = interaction.check().buildQuery();

        // Let's run the query and parse the results:
        await this.networkProvider.queryContract(query);
        let queryResponse = await this.networkProvider.queryContract(query);
        let { firstValue } = this.resultsParser.parseQueryResponse(queryResponse, interaction.getEndpoint());

        console.log(`getAggregatorAddress = ${firstValue!.valueOf().toString()}`)
        // Now let's interpret the results.
        return firstValue!.valueOf().toString();
    }

    async getReserves(): Promise<string> {
        // Prepare the interaction, check it, then build the query:
        let interaction = <Interaction>this.contract.methods.getReserves();
        let query = interaction.check().buildQuery();

        // Let's run the query and parse the results:
        await this.networkProvider.queryContract(query);
        let queryResponse = await this.networkProvider.queryContract(query);
        let { firstValue } = this.resultsParser.parseQueryResponse(queryResponse, interaction.getEndpoint());

        console.log(`Reserves are = ${firstValue!.valueOf().toString()}`)
        // Now let's interpret the results.
        return firstValue!.valueOf().toString();
    }
    
}
