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
$ git clone git@github.com:soleinik/test-sui.git && cd test-sui

```
3. _Build_ and _run_ while in project's root folder, execute `cargo run` as follows

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

Application will fetch most recent Checkpoint (from testnet), iterate over transactions (fetching by digests), find BalanceChanges (is any) and post them for consumption. Application also sets up itself as SUI network even listener (and dumps events on console)

4. To exit

Use `Ctrl-C` to abort excution

```
$ cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 0.25s
     Running `target/debug/app-cli`
[INFO  app_cli] ===============Starting subscription thread...==============
[INFO  app_cli] ===============Starting web thread...==============
[INFO  app_cli] ===============Entering receiving loop...===================
[INFO  app_lib] Entering WSS subscription loop...
[TRACE app_lib] processing checkpoint:0 with 0 balance(s)
[INFO  app_lib] Entering event loop...
[TRACE app_lib] processing checkpoint:25973173 with 1 balance(s)
[
    BalanceChange {
        address: "0x8f04773c6c07615ab1a17a8b078f78de7b52b11c423ee8574b599524f82d5419",
        coin: "0x2::sui::SUI",
        amount: 232760800,
    },
]
[TRACE app_lib] processing checkpoint:25973174 with 3 balance(s)
[
    BalanceChange {
        address: "0x7d20dcdb2bca4f508ea9613994683eb4e76e9c4ed371169677c1be02aaf0b58e",
        coin: "0x2::sui::SUI",
        amount: -239237141880,
    },
]
[
    BalanceChange {
        address: "0x8f04773c6c07615ab1a17a8b078f78de7b52b11c423ee8574b599524f82d5419",
        coin: "0x2::sui::SUI",
        amount: 238000000000,
    },
]
[
    BalanceChange {
        address: "0x910e829eafc78cc370d92c337dec033a0ab00ede9b3072918a505a98c3caea68",
        coin: "0x2::sui::SUI",
        amount: 1000000000,
    },
]
^C

```

## Tests

```
$ cargo test
    Finished test [unoptimized + debuginfo] target(s) in 0.25s
     Running unittests src/main.rs (target/debug/deps/app_cli-38095a22a5d0a3c6)


...
     Running tests/test-routes.rs (target/debug/deps/test_routes-970bacbd984e7dc2)

running 1 test
test web_app_test_routes_ok ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

...

```