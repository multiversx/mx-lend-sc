import { BigIntValue, BytesValue, TokenPayment } from "@elrondnetwork/erdjs";
import { createAirdropService, createESDTInteractor, INetworkProvider, ITestSession, ITestUser, TestSession } from "@elrondnetwork/erdjs-snippets";
import { constants } from "buffer";
import { assert } from "chai";
import { createInteractor } from "./lendingPoolInteractor";

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

    it("airdrop EGLD", async function () {
        session.expectLongInteraction(this);

        let payment = TokenPayment.egldFromAmount(0.1);
        await session.syncUsers([whale]);
        await createAirdropService(session).sendToEachUser(whale, [firstUser, secondUser], [payment]);
    });

    it("issue pool token", async function () {
        session.expectLongInteraction(this);

        let interactor = await createESDTInteractor(session);
        await session.syncUsers([whale]);
        await session.saveToken("tokenABC", await interactor.issueFungibleToken(whale, { name: "ABC", ticker: "ABC", decimals: 0, supply: "10000000000" }));
    });

    it("setup", async function () {
        session.expectLongInteraction(this);

        await session.syncUsers([whale]);

        let token = await session.loadToken("tokenABC");
        let interactor = await createInteractor(session);

        // Deploy dummy liquidity pool
        let { address: dummyAddress, returnCode: dummyReturnCode } = await interactor.deployDummyLiquidityPool(whale, token.identifier);
        assert.isTrue(dummyReturnCode.isSuccess());
        await session.saveAddress("dummyLiquidityPool", dummyAddress);

        // Deploy lending pool
        let { address, returnCode } = await interactor.deploy(whale, dummyAddress);
        assert.isTrue(returnCode.isSuccess());
        await session.saveAddress("contractAddress", address);

        // Setup Liquidity pool
        interactor = await createInteractor(session, address)
        let returnCodeSetup = await interactor.addLiquidityPool(whale, token.identifier, 0, 40000000, 1000000000, 800000000, 100000000, 700000000);
        assert.isTrue(returnCodeSetup.isSuccess());

        // Issue Lend Tokens
        interactor = await createInteractor(session, address)
        returnCodeSetup = await interactor.issueLend(whale, token.identifier);
        assert.isTrue(returnCodeSetup.isSuccess());

        // Issue Borrow Tokens
        interactor = await createInteractor(session, address)
        returnCodeSetup = await interactor.issueBorrow(whale, token.identifier);
        assert.isTrue(returnCodeSetup.isSuccess());

        // Set Lend Roles
        interactor = await createInteractor(session, address)
        returnCodeSetup = await interactor.setLendRoles(whale, token.identifier);
        assert.isTrue(returnCodeSetup.isSuccess());

        // Set Borrow Roles
        interactor = await createInteractor(session, address)
        returnCodeSetup = await interactor.setBorrowRoles(whale, token.identifier);
        assert.isTrue(returnCodeSetup.isSuccess());

        // Set Asset LTV
        interactor = await createInteractor(session, address)
        returnCodeSetup = await interactor.setAssetLoanToValue(whale, token.identifier, 500000000);
        assert.isTrue(returnCodeSetup.isSuccess());

        // Set Liquidation Bonus
        interactor = await createInteractor(session, address)
        returnCodeSetup = await interactor.setAssetLiquidationBonus(whale, token.identifier, 40000000);
        assert.isTrue(returnCodeSetup.isSuccess());
    });

    it("airdrop pool_token to users", async function () {
        session.expectLongInteraction(this);

        let token = await session.loadToken("tokenABC");
        let airdrop = createAirdropService(session);

        await session.syncUsers([whale]);
        await airdrop.sendToEachUser(whale, [firstUser, secondUser], [TokenPayment.fungibleFromAmount(token.identifier, "100000000", token.decimals)]);
    });

    it("deposit token", async function () {
        session.expectLongInteraction(this);

        let token = await session.loadToken("tokenABC");
        let address = await session.loadAddress("contractAddress");
        let interactor = await createInteractor(session, address);
        let paymentOne = TokenPayment.fungibleFromAmount(token.identifier, "500", token.decimals);
        let paymentTwo = TokenPayment.fungibleFromAmount(token.identifier, "700", token.decimals);

        await session.syncUsers([firstUser, secondUser]);
        await interactor.deposit(firstUser, paymentOne);
        await interactor.deposit(secondUser, paymentTwo);
    });
});
