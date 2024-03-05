use std::{env, error::Error};

use app_data::BalanceChange;
use log::{error, info};
use tokio::sync::mpsc::{self, Receiver, Sender};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();

    if let Err(_e) = env::var("RUST_LOG") {
        //println!("Defaulting log level to \"warn\". Unable to retrieve log level environment variable \"RUST_LOG\" Error:{e}");
        env::set_var("RUST_LOG", "trace");
    }

    env_logger::builder().format_timestamp(None).init();

    let (tx, mut rx): (Sender<BalanceChange>, Receiver<BalanceChange>) = mpsc::channel(100);

    info!("===============Starting subscription thread...==============");
    tokio::spawn(async move {
        loop {
            //maybe separate Checkpoint and event loop
            //or... wrap event loop inside the library
            let r = app_lib::lib_run(tx.clone()).await;
            error!("Library thread exited! Result:{r:#?}");
        }
    });

    info!("===============Entering receiving loop...===================");
    loop {
        if let Some(b) = rx.recv().await {
            //send it
            println!("{b:#?}")
        }
    }
}
