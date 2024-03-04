//! This module will asynchronously fetch latest Checkpoint and start daemon loop with even subscription
//! The hope is: find event that notifies of new Checkpoint creation and invoke async process to fetch and process lates checkpoint
//! Research item: find event notification that would signal new Checkpoint
//!

use std::error::Error;

use log::{debug, error, info, trace, warn};
use sui_sdk::{
    rpc_types::{BalanceChange, EventFilter, SuiTransactionBlockResponseOptions},
    types::digests::TransactionDigest,
    SuiClientBuilder,
};

use tokio::sync::mpsc::Sender;
use tokio_stream::StreamExt;

/// Library entry point
/// Fetches latest checkpoint and subscribes to events

pub async fn lib_run(tx: Sender<app_data::BalanceChange>) -> Result<(), Box<dyn Error>> {
    //run checkpoint fetch in separate thread
    tokio::spawn(async move {
        fetch_checkpoint_wrap(&tx).await;
    });

    loop {
        info!("Entering WSS subscription loop...");
        let ws = SuiClientBuilder::default()
            .ws_url("wss://rpc.testnet.sui.io:443")
            .build("https://fullnode.testnet.sui.io:443")
            .await?;

        let subscribe = ws
            .event_api()
            .subscribe_event(EventFilter::All(vec![]))
            .await;

        let mut subscribe = match subscribe {
            Ok(subscribe) => subscribe,
            Err(err) => {
                error!("Subscribing to events error:{err}");
                match err {
                    sui_sdk::error::Error::RpcError(e) => {
                        println!("RPC Error: {e}");
                        continue;
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
                        println!(
                            "Server Version mismatch Error: {client_version} == {server_version}"
                        );
                        //hard error
                        std::process::exit(1);
                    }
                    sui_sdk::error::Error::InsufficientFund { address, amount } => {
                        println!("InsufficientFund Error: {address} and {amount}")
                    }
                }

                //maybe exponential backoff here?
                continue;
            }
        };

        info!("Entering event loop...");
        loop {
            if let Some(evt) = subscribe.next().await {
                match evt {
                    Ok(evt) => println!("event:{}", evt),
                    Err(err) => {
                        error!("Processing events error:{err}");
                        match err {
                            sui_sdk::error::Error::RpcError(e) => println!("RPC Error: {e}"),
                            sui_sdk::error::Error::JsonRpcError(e) => {
                                println!("JSONRPC Error: {e}")
                            }
                            sui_sdk::error::Error::BcsSerialisationError(e) => {
                                println!("BcsSer Error: {e}")
                            }
                            sui_sdk::error::Error::UserInputError(e) => {
                                println!("User Input Error: {e}")
                            }
                            sui_sdk::error::Error::Subscription(e) => {
                                println!("Subscription Error: {e}")
                            }
                            sui_sdk::error::Error::FailToConfirmTransactionStatus(x, y) => {
                                println!("FailToConfirmTransactionStatus Error: {x} & {y}")
                            }
                            sui_sdk::error::Error::DataError(e) => println!("Data Error: {e}"),
                            sui_sdk::error::Error::ServerVersionMismatch {
                                client_version,
                                server_version,
                            } => println!(
                                    "Server Version mismatch Error: {client_version} == {server_version}"
                                ),
                            sui_sdk::error::Error::InsufficientFund { address, amount } => {
                                println!("InsufficientFund Error: {address} and {amount}")
                            }
                        }
                        break;
                    }
                }
            } else {
                info!("Processing events:None - End Of Stream? Have to re-subscribe");
                break;
            };
        }
    }
}

async fn fetch_checkpoint_wrap(tx: &Sender<app_data::BalanceChange>) {
    let mut cnt = 0;
    loop {
        match fetch_checkpoint(tx).await {
            Ok(_) => {
                break;
            }
            Err(err) => {
                cnt += 1;
                if cnt > 3 {
                    error!("Fetching checkpoint error:{err}");
                    std::process::exit(1)
                } else {
                    warn!("attempt[{cnt}] to fetch Checkpoint failed! Error:{err}");
                }
            }
        }
    }
    trace!("===========> done fetching!");
}

async fn fetch_checkpoint(tx: &Sender<app_data::BalanceChange>) -> Result<(), Box<dyn Error>> {
    let sui_net = SuiClientBuilder::default().build_testnet().await?;
    let api = sui_net.read_api();

    //this will fetch one page with one the most recent (descending order) Checkpoint
    let page = api.get_checkpoints(None, Some(1), true).await?;

    let digests = page
        .data
        .into_iter()
        .flat_map(|cp| cp.transactions.into_iter())
        .to_owned()
        .collect::<Vec<TransactionDigest>>();

    trace!("checkpoint has {} transactions...", digests.len());

    let balances = api
        .multi_get_transactions_with_options(
            digests,
            SuiTransactionBlockResponseOptions::full_content(),
        )
        .await?
        .iter()
        .filter_map(|r| r.balance_changes.to_owned())
        .flat_map(|b| b.into_iter())
        .collect::<Vec<BalanceChange>>();

    trace!("checkpoint has {} balances to pubslish...", balances.len());
    publish(&balances, tx).await
}

/// Publish message to the consumer via channels
async fn publish(
    balances: &[BalanceChange],
    tx: &Sender<app_data::BalanceChange>,
) -> Result<(), Box<dyn Error>> {
    for b in balances {
        let msg = app_data::BalanceChange::from(b);
        debug!("Publishing balance:{msg:#?}");
        loop {
            match tx.send(msg).await {
                Ok(_) => break,
                Err(err) => todo!("Implement re-tries here (queue is bounded)... Error:{err}"),
            }
        }
    }

    Ok(())
}
