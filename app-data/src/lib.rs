use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BalanceChange {
    pub address: String,
    pub coin: String,
    pub amout: i128,
}

impl From<&sui_sdk::rpc_types::BalanceChange> for BalanceChange {
    fn from(value: &sui_sdk::rpc_types::BalanceChange) -> Self {
        //let name =
        BalanceChange {
            address: value.owner.to_string(),
            coin: value.coin_type.to_string(),
            amout: value.amount,
        }
    }
}
