#![no_std]

use elrond_wasm::{only_owner, require};

elrond_wasm::imports!();

#[elrond_wasm_derive::contract(LiquidityPoolImpl)]
pub trait LiquidityPool {

    #[view(getReserveName)]
    #[storage_get("reserve_name")]
    fn get_reserve_name(&self) -> TokenIdentifier;

    #[storage_set("reserve_name")]
    fn set_reserve_name(&self, esdt_token: &TokenIdentifier);

    #[view(getReserveAmount)]
    #[storage_get("reserve_amount")]
    fn get_reserve_amount(&self) -> BigUint;

    #[storage_set("reserve_amount")]
    fn set_reserve_amount(&self, amount: &BigUint);

    #[view(getBorrowedAmount)]
    #[storage_get("borrowed_amount")]
    fn get_borrowed_amount(&self) -> BigUint;

    #[storage_set("borrowed_amount")]
    fn set_borrowed_amount(&self, amount: &BigUint);

    #[init]
    fn init(&self, esdt_token: TokenIdentifier) {
        self.set_reserve_name(&esdt_token);
    }

    #[payable("ESDT")]
    #[endpoint]
    fn receive_deposit(&self, #[payment] amount: BigUint) -> SCResult<()> {
        require!(amount != BigUint::zero(), "Amount can not be zero!");

        let mut reserve_amount = self.get_reserve_amount();

        reserve_amount += amount;

        self.set_reserve_amount(&reserve_amount);

        Ok(())
    }

    #[endpoint]
    fn send_asset(&self, address: Address, amount: BigUint) -> SCResult<()> {
        require!(amount != BigUint::zero(), "Amount can not be zero!");
        require!(address != Address::zero(), "Invalid input address!");

        let mut reserve_amount = self.get_reserve_amount();
        reserve_amount -= amount.clone();

        self.set_reserve_amount(&reserve_amount);

        let reserve = self.get_reserve_name();

        self.send()
            .direct_esdt_via_async_call(&address, reserve.as_slice(), &amount, &[]);
    }

    #[view]
    fn get_capital_utilisation() -> SCResult<BigUint> {
        let borrowed_amount = self.get_borrowed_amount();
        let reserve_amount = self.get_reserve_amount();

        let utilisation = borrowed_amount / reserve_amount;

        Ok(utilisation)
    }

    #[payable("EGLD")]
    #[view]
    fn owner(&self, #[payment] cost: BigUint) -> SCResult<BigUint> {
        only_owner!(self, "only european comission members can view!");
        require!(cost == BigUint::from(5u32), "too much provided");
        let reserve = self.get_reserve_amount();

        Ok(reserve)
    }
}
