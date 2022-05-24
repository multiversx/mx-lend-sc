import { BigIntValue, TokenPayment } from "@elrondnetwork/erdjs";
import { createAirdropService, FiveMinutesInMilliseconds, createESDTInteractor, INetworkProvider, ITestSession, ITestUser, TestSession } from "@elrondnetwork/erdjs-snippets";
import { assert } from "chai";
import { helperAddLiquidityPool, helperAirdropTokens, helperIssueBorrowToken, helperIssueLendToken, helperIssueToken, helperSetAggregatorForLP, helperSetAssetLiquidationBonus, helperSetAssetLoanToValue, helperSetBorrowRoles, helperSetLendRoles } from "./lendingPoolHelper";
import { createLendingInteractor } from "./lendingPoolInteractor";
import { createLiquidityInteractor } from "./liquidityPoolInteractor";
import { createPriceAggregatorInteractor } from "./priceAggregatorPoolInteractor";

describe("lending snippet", async function () {
    this.bail(true);

    let suite = this;
    let session: ITestSession;
    let provider: INetworkProvider;
    let whale: ITestUser;
    let firstUser: ITestUser;
    let secondUser: ITestUser;

    this.beforeAll(async function () {
        session = await TestSession.load("devnet", __dirname);
        provider = session.networkProvider;
        whale = session.users.getUser("whale");
        firstUser = session.users.getUser("firstUser");
        secondUser = session.users.getUser("secondUser");
        await session.syncNetworkConfig();
    });

    this.beforeEach(async function () {
        session.correlation.step = this.currentTest?.fullTitle() || "";
    });

    it("Airdrop EGLD", async function () {
        this.timeout(FiveMinutesInMilliseconds);
        
        let payment = TokenPayment.egldFromAmount(0.1);
        await session.syncUsers([whale]);
        await createAirdropService(session).sendToEachUser(whale, [firstUser, secondUser], [payment]);
    });

    it("Issue Pool Token", async function () {
        this.timeout(FiveMinutesInMilliseconds);
        this.retries(5);

        let interactor = await createESDTInteractor(session);
        await session.syncUsers([whale]);
        let token = await interactor.issueFungibleToken(whale, { name: "ABC", ticker: "ABC", decimals: 18, supply: "1000000000000000000000" })
        await session.saveToken({ name: "tokenABC", token: token });
    });


    it("airdrop pool_tokens to users", async function () {
        this.timeout(FiveMinutesInMilliseconds);
        await helperAirdropTokens(session, whale, firstUser, secondUser, "tokenABC");
        await helperAirdropTokens(session, whale, firstUser, secondUser, "tokenXYZ");

    });


    it("Deploy", async function () {
        this.timeout(FiveMinutesInMilliseconds);
        this.retries(5);

        await session.syncUsers([whale]);

        let token = await session.loadToken("tokenABC");
        let interactor = await createLendingInteractor(session);

        // Deploy dummy liquidity pool
        let { address: dummyAddress, returnCode: dummyReturnCode } = await interactor.deployDummyLiquidityPool(whale, token.identifier);
        assert.isTrue(dummyReturnCode.isSuccess());

        // Deploy lending pool
        let { address, returnCode } = await interactor.deploy(whale, dummyAddress);
        assert.isTrue(returnCode.isSuccess());
        await session.saveAddress({name: "lendingAddr", address: address});
    });

    it("Set price aggregator for Liquidity Pools", async function () {
        this.timeout(FiveMinutesInMilliseconds);
        await session.syncUsers([whale, firstUser, secondUser]);



        let priceAggregatorInteractor = await createPriceAggregatorInteractor(session);
        let { address: priceAggregatorAddress, returnCode: returnCode } = await priceAggregatorInteractor.deployAggregator(whale);

        await priceAggregatorInteractor.unpausePoolAggregator(whale);
        await priceAggregatorInteractor.submitPriceAggregator(whale, "ABC", "USD", 7000000000000000000);
        await priceAggregatorInteractor.submitPriceAggregator(whale, "XYZ", "USD", 9000000000000000000);
        await session.saveAddress({name: "priceAggregatorAddress", address: priceAggregatorAddress});
    });


    it("Create Liquidity Pool", async function () {
        this.timeout(FiveMinutesInMilliseconds);
        this.retries(5);

        await session.syncUsers([whale]);

        let isSuccess = await helperAddLiquidityPool(session, whale, "tokenABC");
        assert.isTrue(isSuccess);

        isSuccess = await helperAddLiquidityPool(session, whale, "tokenXYZ");
        assert.isTrue(isSuccess);
    });

    it("Issue Lend Tokens", async function () {
        this.timeout(FiveMinutesInMilliseconds);
        await session.syncUsers([whale]);

        let isSuccess = await helperIssueLendToken(session, whale, "tokenABC");
        assert.isTrue(isSuccess);

        isSuccess = await helperIssueLendToken(session, whale, "tokenXYZ");
        assert.isTrue(isSuccess);
    });

    it("Issue Borrow Tokens ", async function () {
        this.timeout(FiveMinutesInMilliseconds);
        await session.syncUsers([whale]);

        let isSuccess = await helperIssueBorrowToken(session, whale, "tokenABC");
        assert.isTrue(isSuccess);

        isSuccess = await helperIssueBorrowToken(session, whale, "tokenXYZ");
        assert.isTrue(isSuccess);
    });

    it("Setup Lending Pool", async function () {
        this.timeout(FiveMinutesInMilliseconds);
        this.retries(5);

        await session.syncUsers([whale]);

        let isSuccess = await helperSetLendRoles(session, whale, "tokenABC");
        assert.isTrue(isSuccess);

        isSuccess = await helperSetLendRoles(session, whale, "tokenXYZ");
        assert.isTrue( isSuccess);

        isSuccess = await helperSetBorrowRoles(session, whale, "tokenABC");
        assert.isTrue(isSuccess);

        isSuccess = await helperSetBorrowRoles(session, whale, "tokenXYZ");
        assert.isTrue(isSuccess);

        isSuccess = await helperSetAssetLoanToValue(session, whale, "tokenABC");
        assert.isTrue(isSuccess);

        isSuccess = await helperSetAssetLoanToValue(session, whale, "tokenXYZ");
        assert.isTrue(isSuccess);

        isSuccess = await helperSetAssetLiquidationBonus(session, whale, "tokenABC");
        assert.isTrue(isSuccess);

        isSuccess = await helperSetAssetLiquidationBonus(session, whale, "tokenXYZ");
        assert.isTrue(isSuccess);

        isSuccess = await helperSetAggregatorForLP(session, whale, "tokenABC");
        assert.isTrue(isSuccess);

        isSuccess = await helperSetAggregatorForLP(session, whale, "tokenXYZ");
        assert.isTrue(isSuccess);
    });


<<<<<<< HEAD
    it("deposit token", async function () {
        this.timeout(FiveMinutesInMilliseconds);
=======

    it("deposit token ABC", async function () {
        session.expectLongInteraction(this);
        await session.syncUsers([whale, firstUser]);
>>>>>>> Fix Borrow, Withdraw, Deposit scenarios

        let token = await session.loadToken("tokenABC");
        let address = await session.loadAddress("lendingAddr");
        let interactor = await createLendingInteractor(session, address);
        let paymentABC = TokenPayment.fungibleFromAmount(token.identifier, "20", token.decimals);
        let { returnCode: returnCodeDeposit, depositNonce: depositNonceABC } = await interactor.deposit(firstUser, paymentABC);
        assert.isTrue(returnCodeDeposit.isSuccess());

        session.saveBreadcrumb("depositNonceABC", depositNonceABC)
    });


    it("deposit token XYZ", async function () {
        session.expectLongInteraction(this);
        await session.syncUsers([whale, firstUser]);

<<<<<<< HEAD
        session.saveBreadcrumb({ name: "depositNonceOne", value: depositNonceOne})
        session.saveBreadcrumb({name: "depositNonceTwo", value: depositNonceTwo})
    });
=======
        let token = await session.loadToken("tokenXYZ");
        let address = await session.loadAddress("contractAddress");
        let interactor = await createLendingInteractor(session, address);
        let paymentXYZ = TokenPayment.fungibleFromAmount(token.identifier, "20", token.decimals);
        let { returnCode: returnCodeDeposit, depositNonce: depositNonceXYZ } = await interactor.deposit(firstUser, paymentXYZ);
        assert.isTrue(returnCodeDeposit.isSuccess());
>>>>>>> Fix Borrow, Withdraw, Deposit scenarios

        session.saveBreadcrumb("depositNonceXYZ", depositNonceXYZ)
    });

<<<<<<< HEAD
    it("withdraw token", async function () {
        this.timeout(FiveMinutesInMilliseconds);
        
        await session.syncUsers([firstUser, secondUser]);

        let depositNonceOne = await session.loadBreadcrumb("depositNonceOne");
        let depositNonceTwo = await session.loadBreadcrumb("depositNonceTwo");
        let lendingAddress = await session.loadAddress("lendingAddr");
=======
    it("withdraw token XYZ", async function () {
        session.expectLongInteraction(this);
        await session.syncUsers([firstUser, secondUser]);
        let depositNonceXYZ = await session.loadBreadcrumb("depositNonceXYZ");

        let lendingAddress = await session.loadAddress("contractAddress");
>>>>>>> Fix Borrow, Withdraw, Deposit scenarios
        let lendingInteractor = await createLendingInteractor(session, lendingAddress);

        let tokenXYZ = await session.loadToken("tokenXYZ");
        let liquidityAddress = await lendingInteractor.getLiquidityAddress(tokenXYZ.identifier);
        let liquidityInteractor = await createLiquidityInteractor(session, liquidityAddress)
        let lendToken = await liquidityInteractor.getLendingToken();

        let paymentXYZ = TokenPayment.metaEsdtFromAmount(lendToken, depositNonceXYZ, 7, tokenXYZ.decimals)

        let returnCode = await lendingInteractor.withdraw(firstUser, paymentXYZ);
        assert.isTrue(returnCode.isSuccess());

    });

    it("borrow ABC token - collateral XYZ", async function () {
        session.expectLongInteraction(this);
        await session.syncUsers([firstUser, secondUser]);
        let tokenABC = await session.loadToken("tokenABC");
        let tokenXYZ = await session.loadToken("tokenXYZ");
        let depositNonceXYZ = await session.loadBreadcrumb("depositNonceXYZ");

        let lendingAddress = await session.loadAddress("contractAddress");
        let lendingInteractor = await createLendingInteractor(session, lendingAddress);

        let liquidityAddressABC = await lendingInteractor.getLiquidityAddress(tokenABC.identifier);
        let liquidityInteractorABC = await createLiquidityInteractor(session, liquidityAddressABC);
        await liquidityInteractorABC.getAggregatorAddress();
        await liquidityInteractorABC.getReserves();

        let liquidityAddressXYZ = await lendingInteractor.getLiquidityAddress(tokenXYZ.identifier);
        let liquidityInteractorXYZ = await createLiquidityInteractor(session, liquidityAddressXYZ);
        await liquidityInteractorXYZ.getAggregatorAddress();
        await liquidityInteractorXYZ.getReserves();

        let lendToken = await liquidityInteractorXYZ.getLendingToken();
        let assetToBorrow = await liquidityInteractorABC.getPoolAsset();

        await lendingInteractor.getAssetLoanToValue(tokenABC.identifier);
        await lendingInteractor.getAssetLoanToValue(tokenXYZ.identifier);

        let collateralPayment = TokenPayment.metaEsdtFromAmount(lendToken, depositNonceXYZ, 5, tokenXYZ.decimals)

        let { returnCode: returnBorrowCode, borrowNonce: returnBorrowNonce } = await lendingInteractor.borrow(firstUser, collateralPayment, assetToBorrow);
        assert.isTrue(returnBorrowCode.isSuccess());

    });

    it("generate report", async function () {
        await session.generateReport();

    it("destroy session", async function () {
        await session.destroy();
    });
});
