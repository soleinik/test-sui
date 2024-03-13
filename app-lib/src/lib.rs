//! This module will asynchronously fetch latest Checkpoint and start daemon loop with even subscription
//! The hope is: find event that notifies of new Checkpoint creation and invoke async process to fetch and process lates checkpoint
//! Research item: find event notification that would signal new Checkpoint
//!

use anyhow::Result;
use log::{error, info, trace, warn};
use std::{str::FromStr, time::Duration};
use sui_sdk::{
    error::SuiRpcResult,
    rpc_types::{
        BalanceChange, CheckpointId, SuiTransactionBlockEffectsAPI, SuiTransactionBlockResponse,
        SuiTransactionBlockResponseOptions, TransactionFilter,
    },
    types::{base_types::SuiAddress, digests::TransactionDigest},
    SuiClient,
};

use tokio::{sync::mpsc::Sender, time::sleep};
use tokio_stream::StreamExt;

mod connection;

/// Library entry point
/// Fetches latest checkpoint and subscribes to events

const SENDER_ADDRESS: &str = "0x0000000000000000000000000000000000000000000000000000000000000000";

pub async fn lib_run(tx: Sender<app_data::BalanceChange>) -> Result<()> {
    //run checkpoint fetch in separate thread
    let client = connection::connection().await?;
    let _tx = tx.clone();
    tokio::spawn(async move {
        fetch_checkpoint_wrap(&client, None, &_tx).await;
    });

    // Subscription to transaction events
    // We listen to `ConcensusCommitPrologue` as it is one of the required transaction in Checkpoint's transactions set.
    // `ConsensusCommitProlog` transaction contains referenece to Checkpoint's `sequence number`. We use `seq_num` to fetch Checkoint with
    // full set of finalized transactions, that we itterate and find transactions with BalanceChange
    // NOTE: initial thought was to subscribe to Checkpoint event,that would be more optimal, but we were not able to receive the event from SUI public node(devnet,testnet).
    // Additional investigation required...

    let filter = TransactionFilter::FromAddress(SuiAddress::from_str(SENDER_ADDRESS).unwrap());

    let mut checkpoint_seq_num = u64::MIN;

    loop {
        info!("Entering WSS subscription loop...");

        //this is hard error
        let mut client = connection::connection().await?;

        //implements re-try logic and returns hard error if nothing worked
        let mut subscribe = match connection::subscription(&client, filter.clone()).await {
            Ok(subscribe) => subscribe,
            Err(err) => {
                match connection::handle_error(&err) {
                    Ok(_) => {
                        continue; //recoverable
                    }
                    Err(_) => {
                        return Err(anyhow::anyhow!(err)); //non-recoverable
                    }
                }
            }
        };

        info!("Entering event loop...");
        'outer: while let Some(evt) = subscribe.next().await {
            match evt {
                Ok(evt) => {
                    //ConsensusCommitPrologue has no balance changes... it points at checkpoint with other transaction that should have balance changes
                    let digest = evt.transaction_digest();

                    let mut tries = 0;
                    let (seq, resp) = loop {
                        // --------------------------------------------------------------
                        // get checkpoint's sequence number with exp re-tries for the transaction digest
                        // if error is permannent or re-tries exceeded - bail out and hard fail application
                        // NOTE: this code can be improved by re-establishing connection and preserving digest
                        // However, Checkpoint seq_number is something that we are after, so the proposed algoritm should be
                        // - wait to get valid `ConsensusCommitProlog` transaction
                        // - fetch Checkpoint seq_num (the current)
                        // - sequentially, fetch to close the gap between last known Checkpoint seq_num to the application and close the gap
                        //   between "last known" and "current" transaction. This would require either persist "last known" Checkpoit or
                        //   introduce application in-memory state (that not the best for conteineraized deployment, restarts and crash recoveries)
                        //
                        // Crate: [backoff](https://crates.io/crates/backoff)
                        let resp = fetch_transaction(&client, digest).await?;

                        //Data issue, maybe re-try here? ConsensusCommitPrologue should have checkpoint...
                        let Some(seq) = resp.checkpoint else {
                            if tries > 2 {
                                warn!("Giving up re-tries [#{tries}] - transaction {digest} does not have checkpoint");
                                continue 'outer;
                            }
                            warn!(
                                "Attempt #{tries} - transaction {digest} does not have checkpoint"
                            );
                            tries += 1;
                            //this is data race error... we want to predicable timing. It can be exponential backoff with short span
                            sleep(Duration::from_millis(100)).await;
                            //this is where we attempting to re-fetch incomplete `ConsensusCommitPrologue` (missing Checkpoint's seq_num)
                            continue;
                        };
                        if tries > 0 {
                            info!("Attempt #{tries} - Transaction's {digest} missing checkpoint [{seq}] sequence recovered!");
                        }
                        break (seq, resp);
                    };

                    // -------------------------------------------------------------
                    if checkpoint_seq_num != u64::MIN {
                        let diff = seq.abs_diff(checkpoint_seq_num);
                        if diff != 1 {
                            warn!(
                                "gap detected:=================> {seq} - {checkpoint_seq_num} = {diff}"
                            );
                        }
                    }

                    checkpoint_seq_num = seq;

                    fetch_checkpoint_wrap(&client, resp.checkpoint, &tx).await;
                }
                Err(err) => {
                    error!("Processing events error:{err}");
                    // looks like stream is broken here and we will have to re-subscribe
                    break;

                    // match connection::handle_error(&err) {
                    //     Ok(_) => (),
                    //     Err(_) => return Err(anyhow::Error::new(err)),
                    // }
                }
            }
        }
    }
}

async fn fetch_transaction(
    client: &SuiClient,
    digest: &TransactionDigest,
) -> SuiRpcResult<SuiTransactionBlockResponse> {
    backoff::future::retry(backoff::ExponentialBackoff::default(), || async {
        let res = client
            .read_api()
            .get_transaction_with_options(*digest, SuiTransactionBlockResponseOptions::default())
            .await;

        match res {
            Ok(ret_val) => Ok(ret_val),
            Err(e0) => match connection::handle_error(&e0) {
                Ok(_) => Err(e0).map_err(backoff::Error::transient),
                Err(_) => Err(e0).map_err(backoff::Error::permanent),
            },
        }
    })
    .await
}

async fn fetch_checkpoint_wrap(
    client: &SuiClient,
    checkpoint_seq_num: Option<u64>,
    tx: &Sender<app_data::BalanceChange>,
) {
    match backoff::future::retry(backoff::ExponentialBackoff::default(), || async {
        fetch_checkpoint(client, checkpoint_seq_num, tx).await
    })
    .await
    {
        Ok(_) => (),
        Err(err) => {
            error!("Fetching checkpoint error:{err}");
            std::process::exit(1)
        }
    }
}

async fn fetch_checkpoint(
    client: &SuiClient,
    checkpoint_seq_num: Option<u64>,
    tx: &Sender<app_data::BalanceChange>,
) -> std::result::Result<(), backoff::Error<sui_sdk::error::Error>> {
    let api = client.read_api();

    let digests = if let Some(seq_num) = checkpoint_seq_num {
        let cp = match api
            .get_checkpoint(CheckpointId::SequenceNumber(seq_num))
            .await
        {
            Ok(cp) => cp,
            Err(err) => match connection::handle_error(&err) {
                Ok(_) => {
                    println!("fetch Checkpoint[#{seq_num}] recoverable error:{err}");
                    return Err(backoff::Error::transient(err));
                }
                Err(_) => {
                    println!("fetch Checkpoint[#{seq_num}] NON-recoverable error:{err}");
                    return Err(backoff::Error::permanent(err));
                }
            },
        };

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
    publish(&balances, tx).await;
    Ok(())
}

/// Publish message to the consumer via channels
async fn publish(balances: &[BalanceChange], tx: &Sender<app_data::BalanceChange>) {
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
}
