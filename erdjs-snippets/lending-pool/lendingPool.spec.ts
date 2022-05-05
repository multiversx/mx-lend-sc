import {  TokenPayment } from "@elrondnetwork/erdjs";
import { createAirdropService, createESDTInteractor, INetworkProvider, ITestSession, ITestUser, TestSession } from "@elrondnetwork/erdjs-snippets";
import { assert } from "chai";
import { createLendingInteractor } from "./lendingPoolInteractor";
import { createLiquidityInteractor } from "./liquidityPoolInteractor";

describe("lending snippet", async function () {
    this.bail(true);

    let suite = this;
    let session: ITestSession;
    let provider: INetworkProvider;
    let whale: ITestUser;
    let firstUser: ITestUser;
    let secondUser: ITestUser;

    this.beforeAll(async function () {
        session = await TestSession.loadOnSuite("devnet", suite);
        provider = session.networkProvider;
        whale = session.users.getUser("whale");
        firstUser = session.users.getUser("firstUser");
        secondUser = session.users.getUser("secondUser");
        await session.syncNetworkConfig();
    });

    it("Airdrop EGLD", async function () {
        session.expectLongInteraction(this);

        let payment = TokenPayment.egldFromAmount(0.1);
        await session.syncUsers([whale]);
        await createAirdropService(session).sendToEachUser(whale, [firstUser, secondUser], [payment]);
    });

    it("Issue Pool Token", async function () {
        session.expectLongInteraction(this);

        let interactor = await createESDTInteractor(session);
        await session.syncUsers([whale]);
        await session.saveToken("tokenABC", await interactor.issueFungibleToken(whale, { name: "ABC", ticker: "ABC", decimals: 18, supply: "1000000000000000000000" }));
    });

    it("Deploy", async function () {
        session.expectLongInteraction(this);

        await session.syncUsers([whale]);

        let token = await session.loadToken("tokenABC");
        let interactor = await createLendingInteractor(session);

        // Deploy dummy liquidity pool
        let { address: dummyAddress, returnCode: dummyReturnCode } = await interactor.deployDummyLiquidityPool(whale, token.identifier);
        assert.isTrue(dummyReturnCode.isSuccess());
        await session.saveAddress("dummyLiquidityPool", dummyAddress);

        // Deploy lending pool
        let { address, returnCode } = await interactor.deploy(whale, dummyAddress);
        assert.isTrue(returnCode.isSuccess());
        await session.saveAddress("contractAddress", address);
    });


    it("Create Liquidity Pool", async function () {
        session.expectLongInteraction(this);

        await session.syncUsers([whale]);

        let token = await session.loadToken("tokenABC");
        let lendAddress = await session.loadAddress("contractAddress");
        let interactor = await createLendingInteractor(session, lendAddress);

        // Setup Liquidity pool
        let returnCode = await interactor.addLiquidityPool(whale, token.identifier, 0, 40000000, 1000000000, 800000000, 100000000, 700000000);
        assert.isTrue(returnCode.isSuccess());

    });

    it("Issue Lending Token", async function () {
        session.expectLongInteraction(this);

        await session.syncUsers([whale]);

        let token = await session.loadToken("tokenABC");
        let lendAddress = await session.loadAddress("contractAddress");
        let interactor = await createLendingInteractor(session, lendAddress);

        // Issue Lend Tokens
        let returnCode = await interactor.issueLend(whale, token.identifier);
        assert.isTrue(returnCode.isSuccess());
    });

    it("Issue Borrow Token", async function () {
        session.expectLongInteraction(this);

        await session.syncUsers([whale]);

        let token = await session.loadToken("tokenABC");
        let lendAddress = await session.loadAddress("contractAddress");
        let interactor = await createLendingInteractor(session, lendAddress);

        // Issue Borrow Tokens
        let returnCode = await interactor.issueBorrow(whale, token.identifier);
        assert.isTrue(returnCode.isSuccess());
    });

    it("Setup Lending Pool", async function () {
        session.expectLongInteraction(this);

        await session.syncUsers([whale]);

        let token = await session.loadToken("tokenABC");
        let lendAddress = await session.loadAddress("contractAddress");
        let interactor = await createLendingInteractor(session, lendAddress);

        // Set Lend Roles
        let returnCode = await interactor.setLendRoles(whale, token.identifier);
        assert.isTrue(returnCode.isSuccess());

        // Set Borrow Roles
        returnCode = await interactor.setBorrowRoles(whale, token.identifier);
        assert.isTrue(returnCode.isSuccess());

        // Set Asset LTV
        returnCode = await interactor.setAssetLoanToValue(whale, token.identifier, 500000000);
        assert.isTrue(returnCode.isSuccess());

        // Set Liquidation Bonus
        returnCode = await interactor.setAssetLiquidationBonus(whale, token.identifier, 40000000);
        assert.isTrue(returnCode.isSuccess());
    });


    it("airdrop pool_token to users", async function () {
        session.expectLongInteraction(this);

        let token = await session.loadToken("tokenABC");
        let airdrop = createAirdropService(session);

        await session.syncUsers([whale]);
        await airdrop.sendToEachUser(whale, [firstUser, secondUser], [TokenPayment.fungibleFromAmount(token.identifier, "100", token.decimals)]);
    });

    it("deposit token", async function () {
        session.expectLongInteraction(this);

        let token = await session.loadToken("tokenABC");
        let address = await session.loadAddress("contractAddress");
        let interactor = await createLendingInteractor(session, address);
        let paymentOne = TokenPayment.fungibleFromAmount(token.identifier, "10", token.decimals);
        let paymentTwo = TokenPayment.fungibleFromAmount(token.identifier, "10", token.decimals);


        await session.syncUsers([firstUser, secondUser]);
        let { returnCode: returnCodeDeposit1, depositNonce: depositNonceOne } = await interactor.deposit(firstUser, paymentOne);
        assert.isTrue(returnCodeDeposit1.isSuccess());

        let { returnCode: returnCodeDeposit2, depositNonce: depositNonceTwo } = await interactor.deposit(secondUser, paymentTwo);
        assert.isTrue(returnCodeDeposit2.isSuccess());

        session.saveBreadcrumb("depositNonceOne", depositNonceOne)
        session.saveBreadcrumb("depositNonceTwo", depositNonceTwo)
    });


    it("withdraw token", async function () {
        session.expectLongInteraction(this);
        await session.syncUsers([firstUser, secondUser]);
        let depositNonceOne = await session.loadBreadcrumb("depositNonceOne");
        let depositNonceTwo = await session.loadBreadcrumb("depositNonceTwo");
        let lendingAddress = await session.loadAddress("contractAddress");
        let lendingInteractor = await createLendingInteractor(session, lendingAddress);
        let token = await session.loadToken("tokenABC");
        let liquidityAddress = await lendingInteractor.getLiquidityAddress(token.identifier);
        let liquidityInteractor = await createLiquidityInteractor(session, liquidityAddress)
        let lendToken = await liquidityInteractor.getLendingToken();

        let paymentOne = TokenPayment.metaEsdtFromAmount(lendToken, depositNonceOne, 7, token.decimals)
        let paymentTwo = TokenPayment.metaEsdtFromAmount(lendToken, depositNonceTwo, 7, token.decimals)

        let returnCode = await lendingInteractor.withdraw(firstUser, paymentOne);
        assert.isTrue(returnCode.isSuccess());

        returnCode = await lendingInteractor.withdraw(secondUser, paymentTwo);
        assert.isTrue(returnCode.isSuccess());

    });
});
