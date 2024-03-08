use app_data::BalanceChange;
use rand::{distributions::Alphanumeric, Rng};

pub fn get_balance_rand(amount: i128) -> BalanceChange {
    BalanceChange {
        address: address_rand(32),
        coin: coin_rand(),
        amount,
    }
}

fn coin_rand() -> String {
    address_rand(12)
}

fn address_rand(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}
