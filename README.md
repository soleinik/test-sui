# [SUI](https://suiexplorer.com/?network=testnet) network event consumer


# About
SUI network integration attempt... Consume data from the netwrk and listen to events  


## Project structure
This project consists of 3 crates
- [lib](./app-lib/) with main logic
- [launcher](./app-cli/) with binary app
- [data](./app-data/) shared data between services


## How to run

1. Install Rust
2. Clone repo

```
$ git clone git@github.com:soleinik/test-sui.git & cd test-sui

```
3. Build and run
while in project's root folder, execute `cargo run` as follows

```
$ cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 0.82s
     Running `target/debug/app-cli`
[INFO  app_lib] Entering WSS subscription loop...
[TRACE app_lib] checkpoint has 1 transactions...
[TRACE app_lib] checkpoint has 0 balances to pubslish...
[TRACE app_lib] ===========> done fetching!
...
```

Application will fetch most recent Checkpint (from testnet), iterate over transactions (fetching by digests), find BalanceChanges (is any) and post them for consumption. Application also sets up itself as SUI network even listener (and dumps events on console)

4. To exit

Use `Ctrl-C` to abort excution

```
$ cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 0.82s
     Running `target/debug/app-cli`
[INFO  app_lib] Entering WSS subscription loop...
[TRACE app_lib] checkpoint has 1 transactions...
[TRACE app_lib] checkpoint has 0 balances to pubslish...
[TRACE app_lib] ===========> done fetching!
[ERROR app_lib] Subscribing to events error:RPC call failed: ErrorObject { code: InvalidParams, message: "Invalid params", data: None }
RPC Error: RPC call failed: ErrorObject { code: InvalidParams, message: "Invalid params", data: None }
[INFO  app_lib] Entering WSS subscription loop...
^C
```