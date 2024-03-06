use anyhow::Result;

use log::error;
use sui_sdk::{
    rpc_types::{SuiTransactionBlockEffects, TransactionFilter},
    SuiClient, SuiClientBuilder,
};
use tokio_stream::Stream;

pub(crate) async fn connection() -> std::result::Result<SuiClient, sui_sdk::error::Error> {
    //sui_sdk::error::Error> {
    SuiClientBuilder::default()
        .ws_url("wss://rpc.testnet.sui.io:443")
        .build("https://fullnode.testnet.sui.io:443")
        .await
}

pub(crate) async fn subscription(
    client: &SuiClient,
    filter: TransactionFilter,
) -> std::result::Result<
    impl Stream<Item = Result<SuiTransactionBlockEffects, sui_sdk::error::Error>>,
    //backoff::Error<sui_sdk::error::Error>,
    sui_sdk::error::Error,
> {
    //looks like if subscription fails, we need new SuiClient...
    // println!("Subscribing...");
    // backoff::future::retry(backoff::ExponentialBackoff::default(), || async {
    //     Ok(_subscription(client, filter.clone()).await?)
    // })
    // .await
    _subscription(client, filter).await
}

async fn _subscription(
    client: &SuiClient,
    filter: TransactionFilter,
) -> std::result::Result<
    impl Stream<Item = Result<SuiTransactionBlockEffects, sui_sdk::error::Error>>,
    //backoff::Error<sui_sdk::error::Error>,
    sui_sdk::error::Error,
> {
    //loop {
    let subscribe = client
        .read_api()
        .subscribe_transaction(filter.clone())
        .await;
    subscribe

    // match subscribe {
    //     Ok(subscribe) => return Ok(subscribe),
    //     Err(err) => {
    //         error!("Subscribing to events error:{err}");
    //     //     return match handle_error(&err) {
    //     //         Ok(_) => Err(backoff::Error::transient(err)),
    //     //         Err(_) => Err(backoff::Error::permanent(err)),
    //     //     };
    //     // }
    // };
    //}
}

/// maps
pub(crate) fn handle_error(
    err: &sui_sdk::error::Error,
) -> std::result::Result<(), &sui_sdk::error::Error> {
    match err {
        sui_sdk::error::Error::RpcError(e) => {
            match e {
                jsonrpsee::core::Error::Call(v) => match v {
                    jsonrpsee::types::error::CallError::InvalidParams(vv) => {
                        //this seem to be recoverble error
                        error!("Recoverable CallError - InvalidParams[{vv}]");
                        return Ok(());
                    }
                    jsonrpsee::types::error::CallError::Failed(vv) => {
                        error!("Recoverable CallError - failed[{vv}]");
                        return Ok(());
                    }
                    jsonrpsee::types::error::CallError::Custom(vv) => {
                        error!("Recoverable CallError - custom[{vv:?}]");
                        return Ok(());
                    }
                },
                _ => (),
            }
        }
        sui_sdk::error::Error::JsonRpcError(e) => println!("JSONRPC Error: {e}"),
        sui_sdk::error::Error::BcsSerialisationError(e) => {
            println!("BcsSer Error: {e}")
        }
        sui_sdk::error::Error::UserInputError(e) => println!("User Input Error: {e}"),
        sui_sdk::error::Error::Subscription(e) => println!("Subscription Error: {e}"),
        sui_sdk::error::Error::FailToConfirmTransactionStatus(x, y) => {
            println!("FailToConfirmTransactionStatus Error: {x} & {y}")
        }
        sui_sdk::error::Error::DataError(e) => println!("Data Error: {e}"),
        sui_sdk::error::Error::ServerVersionMismatch {
            client_version,
            server_version,
        } => {
            println!("Server Version mismatch Error: {client_version} == {server_version}");
            //hard error
            std::process::exit(1);
        }
        sui_sdk::error::Error::InsufficientFund { address, amount } => {
            println!("InsufficientFund Error: {address} and {amount}")
        }
    }
    error!("Unhandled, NON-Recoverable RpcError[{err}]");
    Err(err)
}
