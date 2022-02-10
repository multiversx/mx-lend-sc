use lending_pool_interaction::LendingSetup;

pub mod constants;
pub mod lending_pool_interaction;
pub mod setup;

#[test]
fn setup_all_test() {
    let _ = LendingSetup::new(lending_pool::contract_obj);
}
