elrond_wasm::imports!();

use crate::{
    IssueData, PoolParams, BORROW_TOKEN_PREFIX, DEBT_TOKEN_NAME, LEND_TOKEN_NAME, LEND_TOKEN_PREFIX,
};
use elrond_wasm::types::{BoxedBytes, OptionalArg, H256};
use elrond_wasm::*;

#[elrond_wasm::module]
pub trait UtilsModule: crate::library::LibraryModule + crate::storage::StorageModule {
    fn prepare_issue_data(&self, prefix: BoxedBytes, ticker: BoxedBytes) -> IssueData {
        let prefixed_ticker = [prefix.as_slice(), ticker.as_slice()].concat();
        let mut issue_data = IssueData {
            name: BoxedBytes::zeros(0),
            ticker: TokenIdentifier::from(BoxedBytes::from(prefixed_ticker)),
            is_empty_ticker: true,
        };

        if prefix == BoxedBytes::from(LEND_TOKEN_PREFIX) {
            let name = [LEND_TOKEN_NAME, ticker.as_slice()].concat();
            issue_data.name = BoxedBytes::from(name.as_slice());
            issue_data.is_empty_ticker = self.lend_token().is_empty();
        } else if prefix == BoxedBytes::from(BORROW_TOKEN_PREFIX) {
            let name = [DEBT_TOKEN_NAME, ticker.as_slice()].concat();
            issue_data.name = BoxedBytes::from(name.as_slice());
            issue_data.is_empty_ticker = self.borrow_token().is_empty();
        }

        issue_data
    }

    fn get_nft_hash(&self) -> H256 {
        let debt_nonce = self.debt_nonce().get();
        let hash = self.crypto().keccak256(&debt_nonce.to_be_bytes()[..]);
        self.debt_nonce().set(&(debt_nonce + 1));
        hash
    }

    fn compute_health_factor(&self) -> u32 {
        1u32
    }

    fn _get_borrow_rate(
        &self,
        pool_params: PoolParams<Self::BigUint>,
        #[var_args] utilisation: OptionalArg<Self::BigUint>,
    ) -> Self::BigUint {
        let u_current = utilisation
            .into_option()
            .unwrap_or_else(|| self.get_capital_utilisation());

        self.compute_borrow_rate(
            pool_params.r_base,
            pool_params.r_slope1,
            pool_params.r_slope2,
            pool_params.u_optimal,
            u_current,
        )
    }

    fn get_capital_utilisation(&self) -> Self::BigUint {
        let reserve_amount = self.reserves(&self.pool_asset().get()).get();
        //TODO: change with view_reserve after putting all view functions in a module
        let borrowed_amount = self.total_borrow().get();

        self.compute_capital_utilisation(borrowed_amount, reserve_amount)
    }

    fn get_debt_interest(&self, amount: Self::BigUint, timestamp: u64) -> Self::BigUint {
        let now = self.blockchain().get_block_timestamp();
        let time_diff = Self::BigUint::from(now - timestamp);

        let borrow_rate = self.get_borrow_rate();

        self.compute_debt(amount, time_diff, borrow_rate)
    }

    fn get_deposit_rate(&self) -> Self::BigUint {
        let utilisation = self.get_capital_utilisation();
        let pool_params = self.pool_params().get();
        let reserve_factor = pool_params.reserve_factor.clone();
        let borrow_rate =
            self._get_borrow_rate(pool_params, OptionalArg::Some(utilisation.clone()));

        self.compute_deposit_rate(utilisation, borrow_rate, reserve_factor)
    }

    fn get_borrow_rate(&self) -> Self::BigUint {
        let pool_params = self.pool_params().get();
        self._get_borrow_rate(pool_params, OptionalArg::None)
    }
}
