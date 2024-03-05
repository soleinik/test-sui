use log::error;
use std::error::Error;
use sui_sdk::{
    rpc_types::{SuiTransactionBlockEffects, TransactionFilter},
    SuiClient, SuiClientBuilder,
};
use tokio_stream::Stream;

pub(crate) async fn connection() -> Result<SuiClient, sui_sdk::error::Error> {
    SuiClientBuilder::default()
        .ws_url("wss://rpc.testnet.sui.io:443")
        .build("https://fullnode.testnet.sui.io:443")
        .await
}

pub(crate) async fn subscription(
    client: &SuiClient,
    filter: TransactionFilter,
) -> Result<
    impl Stream<Item = Result<SuiTransactionBlockEffects, sui_sdk::error::Error>>,
    Box<dyn Error>,
> {
    backoff::future::retry(backoff::ExponentialBackoff::default(), || async {
        Ok(_subscription(client, filter.clone()).await?)
    })
    .await
}

async fn _subscription(
    client: &SuiClient,
    filter: TransactionFilter,
) -> Result<
    impl Stream<Item = Result<SuiTransactionBlockEffects, sui_sdk::error::Error>>,
    Box<dyn Error>,
> {
    loop {
        let subscribe = client
            .read_api()
            .subscribe_transaction(filter.clone())
            .await;

        match subscribe {
            Ok(subscribe) => return Ok(subscribe),
            Err(err) => {
                error!("Subscribing to events error:{err}");
                handle_error(&err)?;
                continue;
            }
        };
    }
}

pub(crate) fn handle_error(err: &sui_sdk::error::Error) -> Result<(), sui_sdk::error::Error> {
    match err {
        sui_sdk::error::Error::RpcError(e) => {
            println!("RPC Error: {e}");
            //continue;
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

    Ok(())
}
