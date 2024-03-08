use serde::{Deserialize, Serialize};
use sui_sdk::types::object::Owner;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BalanceChange {
    pub address: String,
    pub coin: String,
    pub amount: i128,
}

impl From<&sui_sdk::rpc_types::BalanceChange> for BalanceChange {
    fn from(value: &sui_sdk::rpc_types::BalanceChange) -> Self {
        let addr = match value.owner {
            Owner::AddressOwner(sui) => sui.to_string(),
            _ => value.owner.to_string(),
        };
        BalanceChange {
            address: addr,
            coin: value.coin_type.to_string(),
            amount: value.amount,
        }
    }
}
