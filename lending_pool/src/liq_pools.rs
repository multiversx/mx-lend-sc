elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi)]
pub struct LiqPoolMetadata {
    pub address: Address,
    pub asset_id: TokenIdentifier,
    pub lend_id: Option<TokenIdentifier>,
    pub borrow_id: Option<TokenIdentifier>,
}

impl LiqPoolMetadata {
    pub fn new(address: Address, asset_id: TokenIdentifier) -> Self {
        LiqPoolMetadata {
            address,
            asset_id,
            lend_id: Option::None,
            borrow_id: Option::None,
        }
    }
}

impl Default for LiqPoolMetadata {
    fn default() -> Self {
        LiqPoolMetadata {
            address: Address::zero(),
            asset_id: TokenIdentifier::egld(),
            lend_id: Option::None,
            borrow_id: Option::None,
        }
    }
}

#[elrond_wasm::module]
pub trait LiqPoolsModule {
    fn add_pool(&self, address: &Address, asset_id: &TokenIdentifier) {
        self.pools().insert(
            address.clone(),
            LiqPoolMetadata::new(address.clone(), asset_id.clone()),
        );
        self.asset_id_to_pool_address(asset_id).set(address);
    }

    fn set_lend_id(&self, address: &Address, lend_id: &TokenIdentifier) -> SCResult<()> {
        let metadata_opt = self.pools().get(address);
        require!(!metadata_opt.is_none(), "LiqPoolMetadata is None");

        let mut metadata = metadata_opt.unwrap_or_default();
        require!(metadata.mend_id.is_none(), "Lend Id already set");
        metadata.lend_id = lend_id.clone();

        self.lend_id_to_pool_address(lend_id).set(address);
        self.pools().insert(address, metadata);
        Ok(())
    }

    fn set_borrow_id(&self, address: &Address, borrow_id: &TokenIdentifier) -> SCResult<()> {
        let metadata_opt = self.pools().get(address);
        require!(!metadata_opt.is_none(), "LiqPoolMetadata is None");

        let mut metadata = metadata_opt.unwrap_or_default();
        require!(metadata.borrow_id.is_none(), "Borrow Id already set");
        metadata.borrow_id = borrow_id.clone();

        self.borrow_id_to_pool_address(borrow_id).set(address);
        self.pools().insert(address, metadata);
        Ok(())
    }

    fn get_pool_metadata_by_address(&self, address: &Address) -> Option<LiqPoolMetadata> {
        self.pools().get(address)
    }

    fn get_pool_metadata_by_asset_id(&self, asset_id: &TokenIdentifier) -> Option<LiqPoolMetadata> {
        if self.asset_id_to_pool_address(asset_id).is_empty() {
            return Option::None;
        }

        let address = self.asset_id_to_pool_address(asset_id).get();
        self.pools().get(&address)
    }

    fn get_pool_metadata_by_lend_id(&self, lend_id: &TokenIdentifier) -> Option<LiqPoolMetadata> {
        if self.lend_id_to_pool_address(lend_id).is_empty() {
            return Option::None;
        }

        let address = self.lend_id_to_pool_address(lend_id).get();
        self.pools().get(&address)
    }

    fn get_pool_metadata_by_borrow_id(
        &self,
        borrow_id: &TokenIdentifier,
    ) -> Option<LiqPoolMetadata> {
        if self.borrow_id_to_pool_address(borrow_id).is_empty() {
            return Option::None;
        }

        let address = self.borrow_id_to_pool_address(borrow_id).get();
        self.pools().get(&address)
    }

    #[storage_mapper("pools")]
    fn pools(&self) -> SafeMapMapper<Self::Storage, Address, LiqPoolMetadata>;

    #[storage_mapper("asset_id_to_pool_address")]
    fn asset_id_to_pool_address(
        &self,
        asset_id: TokenIdentifier,
    ) -> SingleValueMapper<Self::Storage, Address>;

    #[storage_mapper("lend_id_to_pool_address")]
    fn lend_id_to_pool_address(
        &self,
        lend_id: TokenIdentifier,
    ) -> SingleValueMapper<Self::Storage, Address>;

    #[storage_mapper("borrow_id_to_pool_address")]
    fn borrow_id_to_pool_address(
        &self,
        borrow_id: TokenIdentifier,
    ) -> SingleValueMapper<Self::Storage, Address>;

    #[proxy]
    fn liquidity_pool_proxy(&self, sc_address: Address) -> liquidity_pool::Proxy<Self::SendApi>;
}
