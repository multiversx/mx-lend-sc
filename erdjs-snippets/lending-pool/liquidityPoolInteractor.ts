import path from "path";
import { AddressValue, BigIntValue, BytesValue, CodeMetadata, IAddress, Interaction, ResultsParser, ReturnCode, SmartContract, SmartContractAbi, Struct, TokenIdentifierValue, TokenPayment, TransactionWatcher } from "@elrondnetwork/erdjs";
import { INetworkProvider, ITestSession, ITestUser, loadAbiRegistry, loadCode } from "@elrondnetwork/erdjs-snippets";
import { NetworkConfig } from "@elrondnetwork/erdjs-network-providers";

const PathtoLiquidityAbi = path.resolve(__dirname, "..", "..", "liquidity_pool", "output", "liquidity-pool.abi.json");

export async function createLiquidityInteractor(session: ITestSession, contractAddress?: IAddress): Promise<LiquidityPoolInteractor> {
    let registry = await loadAbiRegistry(PathtoLiquidityAbi);
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
        // let contract = this.liquidityHashMap.get(liquidityPoolAddress.bech32());

        // Prepare the interaction, check it, then build the query:
        let interaction = <Interaction>this.contract.methods.getDepositRate();
        let query = interaction.check().buildQuery();

        // Let's run the query and parse the results:
        await this.networkProvider.queryContract(query);
        let queryResponse = await this.networkProvider.queryContract(query);
        let { firstValue } = this.resultsParser.parseQueryResponse(queryResponse, interaction.getEndpoint());

        // Now let's interpret the results.
        return firstValue!.valueOf().toString();
    }

}
