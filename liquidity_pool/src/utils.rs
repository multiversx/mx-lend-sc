elrond_wasm::imports!();

use elrond_wasm::*;
use crate::{IssueData, LEND_TOKEN_PREFIX, BORROW_TOKEN_PREFIX, LEND_TOKEN_NAME, DEBT_TOKEN_NAME, ReserveData};
use elrond_wasm::types::{H256, OptionalArg, BoxedBytes};

#[elrond_wasm_derive::module]
pub trait UtilsModule: crate::library::LibraryModule + crate::storage::StorageModule{

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
        let hash = self.keccak256(&debt_nonce.to_be_bytes()[..]);
        self.debt_nonce().set(&(debt_nonce + 1));
        hash
    }

    fn compute_health_factor(&self) -> u32 {
        1u32
    }

    fn _get_borrow_rate(
        &self,
        reserve_data: ReserveData<Self::BigUint>,
        #[var_args] utilisation: OptionalArg<Self::BigUint>,
    ) -> Self::BigUint {
        let u_current = utilisation
            .into_option()
            .unwrap_or_else(|| self.get_capital_utilisation());

        self.compute_borrow_rate(
            reserve_data.r_base,
            reserve_data.r_slope1,
            reserve_data.r_slope2,
            reserve_data.u_optimal,
            u_current,
        )
    }

    fn get_capital_utilisation(&self) -> Self::BigUint {
        let reserve_amount = self.reserves()
                            .get(&self.pool_asset().get())
                            .unwrap_or_else(Self::BigUint::zero);
        //TODO: change with view_reserve after putting all view functions in a module
        let borrowed_amount = self.get_total_borrow();

        self.compute_capital_utilisation(borrowed_amount, reserve_amount)
    }

}