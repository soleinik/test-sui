//! This module will asynchronously fetch latest Checkpoint and start daemon loop with even subscription
//! The hope is: find event that notifies of new Checkpoint creation and invoke async process to fetch and process lates checkpoint
//! Research item: find event notification that would signal new Checkpoint
//!

use std::{error::Error, str::FromStr};

use log::{debug, error, info, trace, warn};
use sui_sdk::{
    rpc_types::{
        BalanceChange, CheckpointId, SuiTransactionBlockEffectsAPI,
        SuiTransactionBlockResponseOptions, TransactionFilter,
    },
    types::{base_types::SuiAddress, digests::TransactionDigest},
    SuiClientBuilder,
};

use tokio::sync::mpsc::Sender;
use tokio_stream::StreamExt;

/// Library entry point
/// Fetches latest checkpoint and subscribes to events

const SENDER_ADDRESS: &str = "0x0000000000000000000000000000000000000000000000000000000000000000";

pub async fn lib_run(tx: Sender<app_data::BalanceChange>) -> Result<(), Box<dyn Error>> {
    //run checkpoint fetch in separate thread
    let _tx = tx.clone();
    tokio::spawn(async move {
        fetch_checkpoint_wrap(None, &_tx).await;
    });

    let filter = TransactionFilter::FromAddress(SuiAddress::from_str(SENDER_ADDRESS).unwrap());

    let mut checkpoint_seq_num = u64::MIN;

    loop {
        info!("Entering WSS subscription loop...");
        let ws = SuiClientBuilder::default()
            .ws_url("wss://rpc.testnet.sui.io:443")
            .build("https://fullnode.testnet.sui.io:443")
            .await?;

        let subscribe = ws.read_api().subscribe_transaction(filter.clone()).await;

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
        while let Some(evt) = subscribe.next().await {
            match evt {
                Ok(evt) => {
                    //ConsensusCommitPrologue has no balance changes... it points at checkpoint with other transaction that should have balance changes
                    let digest = evt.transaction_digest();

                    let resp = ws
                        .read_api()
                        .get_transaction_with_options(
                            *digest,
                            SuiTransactionBlockResponseOptions::default(),
                        )
                        .await?;

                    if resp.checkpoint.is_none() {
                        debug!("checkpoint is none: {resp}");
                        continue;
                    }

                    let seq = resp.checkpoint.clone().unwrap();
                    if checkpoint_seq_num != u64::MIN {
                        let diff = seq.abs_diff(checkpoint_seq_num);
                        if diff != 1 {
                            warn!(
                                "gap detected:=================> {seq} - {checkpoint_seq_num} = {diff}"
                            );
                        }
                    }

                    checkpoint_seq_num = seq;

                    fetch_checkpoint_wrap(resp.checkpoint, &tx).await;
                }
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
        }
    }
}

async fn fetch_checkpoint_wrap(
    checkpoint_seq_num: Option<u64>,
    tx: &Sender<app_data::BalanceChange>,
) {
    let mut cnt = 0;
    loop {
        match fetch_checkpoint(checkpoint_seq_num, tx).await {
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
}

async fn fetch_checkpoint(
    checkpoint_seq_num: Option<u64>,
    tx: &Sender<app_data::BalanceChange>,
) -> Result<(), Box<dyn Error>> {
    let sui_net = SuiClientBuilder::default().build_testnet().await?;
    let api = sui_net.read_api();

    let digests = if let Some(v) = checkpoint_seq_num {
        let cp = api.get_checkpoint(CheckpointId::SequenceNumber(v)).await?;
        //ConsensusCommitPrologue and one more.
        if cp.transactions.len() < 2 {
            return Ok(());
        }

        cp.transactions
            .iter()
            .map(|v| v.to_owned())
            .collect::<Vec<TransactionDigest>>()
    } else {
        let page = api.get_checkpoints(None, Some(1), true).await?;
        page.data
            .into_iter()
            .filter(|cp| cp.transactions.len() > 1)
            .flat_map(|cp| cp.transactions.into_iter())
            .to_owned()
            .collect::<Vec<TransactionDigest>>()
    };

    let balances = api
        .multi_get_transactions_with_options(
            digests.clone(),
            SuiTransactionBlockResponseOptions::default().with_balance_changes(),
        )
        .await?
        .iter()
        .filter_map(|r| r.balance_changes.to_owned())
        .filter(|b| !b.is_empty())
        .flat_map(|b| b.into_iter())
        .collect::<Vec<BalanceChange>>();

    trace!(
        "processing checkpoint:{} with {} balance(s)",
        checkpoint_seq_num.unwrap_or_default(),
        balances.len()
    );
    publish(&balances, tx).await
}

/// Publish message to the consumer via channels
async fn publish(
    balances: &[BalanceChange],
    tx: &Sender<app_data::BalanceChange>,
) -> Result<(), Box<dyn Error>> {
    for b in balances {
        let msg = app_data::BalanceChange::from(b);
        loop {
            match tx.send(msg).await {
                Ok(_) => {
                    break;
                }
                Err(err) => todo!("Implement re-tries here (queue is bounded)... Error:{err}"),
            }
        }
    }

    Ok(())
}
