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

    it("Issue Pool Token USDC", async function () {
        this.timeout(FiveMinutesInMilliseconds);
        this.retries(5);

        let interactor = await createESDTInteractor(session);
        await session.syncUsers([whale]);
        let token = await interactor.issueFungibleToken(whale, { name: "USDC", ticker: "USD", decimals: 18, supply: "1000000000000000000000" })
        await session.saveToken({ name: "tokenUSD", token: token });
    });

    it("Issue Pool Token EGLD", async function () {
        this.timeout(FiveMinutesInMilliseconds);
        this.retries(5);

        let interactor = await createESDTInteractor(session);
        await session.syncUsers([whale]);
        let token = await interactor.issueFungibleToken(whale, { name: "EGLD", ticker: "EGLD", decimals: 18, supply: "1000000000000000000000" })
        await session.saveToken({ name: "tokenEGLD", token: token });
    });

    it("airdrop pool_tokens to users", async function () {
        this.timeout(FiveMinutesInMilliseconds);
        await helperAirdropTokens(session, whale, firstUser, secondUser, "tokenUSD");
        await helperAirdropTokens(session, whale, firstUser, secondUser, "tokenEGLD");

    });


    it("Deploy", async function () {
        this.timeout(FiveMinutesInMilliseconds);
        this.retries(5);

        await session.syncUsers([whale]);

        let token = await session.loadToken("tokenUSD");
        let interactor = await createLendingInteractor(session);

        // Deploy dummy liquidity pool
        let { address: dummyAddress, returnCode: dummyReturnCode } = await interactor.deployDummyLiquidityPool(whale, token.identifier);
        assert.isTrue(dummyReturnCode.isSuccess());

        // Deploy lending pool
        let { address, returnCode } = await interactor.deploy(whale, dummyAddress);
        assert.isTrue(returnCode.isSuccess());
        await session.saveAddress({name: "lendingAddr", address: address});
    });

    it("Issue Account Token", async function () {
        this.timeout(FiveMinutesInMilliseconds);
        this.retries(5);

        await session.syncUsers([whale]);

        let address = await session.loadAddress("lendingAddr");
        let interactor = await createLendingInteractor(session, address);

        // Deploy dummy liquidity pool
        let returnCode = await interactor.registerAccountToken(whale, "LAccount", "LACC");
        assert.isTrue(returnCode.isSuccess());
        
    });

    it("Set price aggregator for Liquidity Pools", async function () {
        this.timeout(FiveMinutesInMilliseconds);
        await session.syncUsers([whale, firstUser, secondUser]);

        let priceAggregatorInteractor = await createPriceAggregatorInteractor(session);
        let { address: priceAggregatorAddress, returnCode: returnCode } = await priceAggregatorInteractor.deployAggregator(whale);

        await priceAggregatorInteractor.unpausePoolAggregator(whale);
        await priceAggregatorInteractor.submitPriceAggregator(whale, "USDC", "USD", 1000000000000000000);
        await priceAggregatorInteractor.submitPriceAggregator(whale, "EGLD", "EGLD", 50000000000000000000);
        await session.saveAddress({name: "priceAggregatorAddress", address: priceAggregatorAddress});
    });


    it("Create Liquidity Pool", async function () {
        this.timeout(FiveMinutesInMilliseconds);
        this.retries(5);

        await session.syncUsers([whale]);

        let isSuccess = await helperAddLiquidityPool(session, whale, "tokenUSD");
        assert.isTrue(isSuccess);

        isSuccess = await helperAddLiquidityPool(session, whale, "tokenEGLD");
        assert.isTrue(isSuccess);
    });

    it("Setup Lending Pool", async function () {
        this.timeout(FiveMinutesInMilliseconds);
        this.retries(5);
        let isSuccess;

        await session.syncUsers([whale]);

        isSuccess = await helperSetAssetLoanToValue(session, whale, "tokenUSD");
        assert.isTrue(isSuccess);

        isSuccess = await helperSetAssetLoanToValue(session, whale, "tokenEGLD");
        assert.isTrue(isSuccess);

        isSuccess = await helperSetAssetLiquidationBonus(session, whale, "tokenUSD");
        assert.isTrue(isSuccess);

        isSuccess = await helperSetAssetLiquidationBonus(session, whale, "tokenEGLD");
        assert.isTrue(isSuccess);

        isSuccess = await helperSetAggregatorForLP(session, whale, "tokenUSD");
        assert.isTrue(isSuccess);

        isSuccess = await helperSetAggregatorForLP(session, whale, "tokenEGLD");

    });

    it("enter market First User", async function () {
        this.timeout(FiveMinutesInMilliseconds);
        
        await session.syncUsers([whale, firstUser]);

        let address = await session.loadAddress("lendingAddr");
        let interactor = await createLendingInteractor(session, address);
        let { returnCode: returnCodeDeposit, accountNonce: accountNonceFirstUser, accountTokenId: accountTokenIdFirstUser } = await interactor.enter_market(firstUser);
        assert.isTrue(returnCodeDeposit.isSuccess());

        session.saveBreadcrumb({name: "accountNonceFirstUser", value: accountNonceFirstUser})
        session.saveBreadcrumb({name: "accountTokenIdFirstUser", value: accountTokenIdFirstUser})
    });

    it("enter market Second User", async function () {
        this.timeout(FiveMinutesInMilliseconds);
        
        await session.syncUsers([whale, secondUser]);

        let address = await session.loadAddress("lendingAddr");
        let interactor = await createLendingInteractor(session, address);
        let { returnCode: returnCodeDeposit, accountNonce: accountNonceSecondUser, accountTokenId: accountTokenIdSecondUser } = await interactor.enter_market(secondUser);
        assert.isTrue(returnCodeDeposit.isSuccess());

        session.saveBreadcrumb({name: "accountNonceSecondUser", value: accountNonceSecondUser})
        session.saveBreadcrumb({name: "accountTokenIdSecondUser", value: accountTokenIdSecondUser})

    });

    it("addCollateral token USD", async function () {
        this.timeout(FiveMinutesInMilliseconds);
        
        await session.syncUsers([whale, firstUser]);

        let tokenUSD = await session.loadToken("tokenUSD");
        let address = await session.loadAddress("lendingAddr");
        let interactor = await createLendingInteractor(session, address);
        let depositNonceUSDC = await session.loadBreadcrumb("accountNonceFirstUser");
        let accountTokenIdFirstUser = await session.loadBreadcrumb("accountTokenIdFirstUser");

        let paymentAccountNFT = TokenPayment.nonFungible(accountTokenIdFirstUser, depositNonceUSDC);
        let paymentUSD = TokenPayment.fungibleFromAmount(tokenUSD.identifier, "20", tokenUSD.decimals);
        // let payment = [paymentAccountNFT, paymentUSD];

        let returnCode = await interactor.addCollateral(firstUser, paymentAccountNFT, paymentUSD);
        assert.isTrue(returnCode.isSuccess());
    });


    it("addCollateral token EGLD", async function () {
        this.timeout(FiveMinutesInMilliseconds);
        await session.syncUsers([whale, secondUser]);
        
        let tokenEGLD = await session.loadToken("tokenEGLD");
        let address = await session.loadAddress("lendingAddr");
        let interactor = await createLendingInteractor(session, address);
        let depositNonceEGLD = await session.loadBreadcrumb("accountNonceSecondUser");
        let accountTokenIdSecondUser = await session.loadBreadcrumb("accountTokenIdSecondUser");

        let paymentAccountNFT = TokenPayment.nonFungible(accountTokenIdSecondUser, depositNonceEGLD);
        let paymentUSD = TokenPayment.fungibleFromAmount(tokenEGLD.identifier, "10", tokenEGLD.decimals);
        // let payment = [paymentAccountNFT, paymentUSD];

        let returnCode = await interactor.addCollateral(secondUser, paymentAccountNFT, paymentUSD);
        assert.isTrue(returnCode.isSuccess());
    });

    it("withdraw token EGLD", async function () {
        this.timeout(FiveMinutesInMilliseconds);
        
        await session.syncUsers([secondUser]);

        let tokenEGLD = await session.loadToken("tokenEGLD");
        let lendingAddress = await session.loadAddress("lendingAddr");
        let depositNonceEGLD = await session.loadBreadcrumb("accountNonceSecondUser");
        let accountTokenIdSecondUser = await session.loadBreadcrumb("accountTokenIdSecondUser");
        let paymentAccountNFT = TokenPayment.nonFungible(accountTokenIdSecondUser, depositNonceEGLD);

        let lendingInteractor = await createLendingInteractor(session, lendingAddress);

        // let liquidityAddress = await lendingInteractor.getLiquidityAddress(tokenEGLD.identifier);
        // let liquidityInteractorEGLD = await createLiquidityInteractor(session, liquidityAddress)
        // let lendTokenEGLD = await liquidityInteractorEGLD.getLendToken();

        let returnCode = await lendingInteractor.removeCollateral(secondUser, tokenEGLD.identifier, 5, paymentAccountNFT);
        assert.isTrue(returnCode.isSuccess());
    });

    it("borrow ABC token - collateral XYZ", async function () {
        this.timeout(FiveMinutesInMilliseconds);
        await session.syncUsers([firstUser, secondUser]);
        let tokenABC = await session.loadToken("tokenABC");
        let tokenXYZ = await session.loadToken("tokenXYZ");
        let depositNonceXYZ = await session.loadBreadcrumb("depositNonceXYZ");

        let lendingAddress = await session.loadAddress("lendingAddr");
        let lendingInteractor = await createLendingInteractor(session, lendingAddress);

        let liquidityAddressABC = await lendingInteractor.getLiquidityAddress(tokenABC.identifier);
        let liquidityInteractorABC = await createLiquidityInteractor(session, liquidityAddressABC);
        // await liquidityInteractorABC.getAggregatorAddress();
        // await liquidityInteractorABC.getReserves();

        let liquidityAddressXYZ = await lendingInteractor.getLiquidityAddress(tokenXYZ.identifier);
        let liquidityInteractorXYZ = await createLiquidityInteractor(session, liquidityAddressXYZ);
        // await liquidityInteractorXYZ.getAggregatorAddress();
        // await liquidityInteractorXYZ.getReserves();

        let lendTokenXYZ = await liquidityInteractorXYZ.getLendToken();
        let assetToBorrowABC = await liquidityInteractorABC.getPoolAsset();

        // await lendingInteractor.getAssetLoanToValue(tokenABC.identifier);
        // await lendingInteractor.getAssetLoanToValue(tokenXYZ.identifier);

        let collateralPayment = TokenPayment.metaEsdtFromAmount(lendTokenXYZ, depositNonceXYZ, 5, tokenXYZ.decimals)

        let { returnCode: returnBorrowCode, borrowNonce: returnBorrowNonce } = await lendingInteractor.borrow(firstUser, collateralPayment, assetToBorrowABC);
        assert.isTrue(returnBorrowCode.isSuccess());

        session.saveBreadcrumb({name: "borrowedNonceABC", value: returnBorrowNonce})
    });


    it("repay ABC token - collateral XYZ", async function () {
        this.timeout(FiveMinutesInMilliseconds);
        await session.syncUsers([firstUser, secondUser]);
        let tokenABC = await session.loadToken("tokenABC");
        let tokenXYZ = await session.loadToken("tokenXYZ");
        let borrowedNonceABC = await session.loadBreadcrumb("borrowedNonceABC");

        let lendingAddress = await session.loadAddress("lendingAddr");
        let lendingInteractor = await createLendingInteractor(session, lendingAddress);

        let liquidityAddressABC = await lendingInteractor.getLiquidityAddress(tokenABC.identifier);
        let liquidityInteractorABC = await createLiquidityInteractor(session, liquidityAddressABC);
        // await liquidityInteractorABC.getAggregatorAddress();
        // await liquidityInteractorABC.getReserves();

        let liquidityAddressXYZ = await lendingInteractor.getLiquidityAddress(tokenXYZ.identifier);
        let liquidityInteractorXYZ = await createLiquidityInteractor(session, liquidityAddressXYZ);
        // await liquidityInteractorXYZ.getAggregatorAddress();
        // await liquidityInteractorXYZ.getReserves();

        let borrowedTokenABC = await liquidityInteractorABC.getBorrowToken();
        let assetToRepayABC = await liquidityInteractorABC.getPoolAsset();

        // await lendingInteractor.getAssetLoanToValue(tokenABC.identifier);
        // await lendingInteractor.getAssetLoanToValue(tokenXYZ.identifier);

        let paymentOne = TokenPayment.metaEsdtFromAmount(borrowedTokenABC, borrowedNonceABC, 3.21, tokenXYZ.decimals);
        let paymentTwo = TokenPayment.fungibleFromAmount(assetToRepayABC, 5, tokenXYZ.decimals);
        let repayment = [paymentOne, paymentTwo];

        let returnCode = await lendingInteractor.repay(firstUser, repayment, assetToRepayABC);
        assert.isTrue(returnCode.isSuccess());
    });

    it("generate report", async function () {
        await session.generateReport();
    });

    it("destroy session", async function () {
        await session.destroy();
    });
});
