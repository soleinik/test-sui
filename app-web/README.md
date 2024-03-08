# Application web app (aka consumer)


# About
Demo web app consumer


# How to use
Launcher will start SUI listener and post BalanceChanges to Web App 

- to get accounts (from another terminal)

```
$ curl -s  http://localhost:8080/accounts | jq
[
  "0x660ea6bc10f2d6c2d40b829850ab746a6ad93c2674537c71e21809b0486254c6",
  "0x7d20dcdb2bca4f508ea9613994683eb4e76e9c4ed371169677c1be02aaf0b58e",
  "0x8f04773c6c07615ab1a17a8b078f78de7b52b11c423ee8574b599524f82d5419",
  "0xc9c8e0d738d7f090144847b38a8283fbe8050923875771b8c315a461721c04a4"
]
```
- to get particular account balance

```
$ curl -s  http://localhost:8080/balances/0x7d20dcdb2bca4f508ea9613994683eb4e76e9c4ed371169677c1be02aaf0b58e | jq
{
  "address": "0x7d20dcdb2bca4f508ea9613994683eb4e76e9c4ed371169677c1be02aaf0b58e",
  "coin": "0x2::sui::SUI",
  "amount": -2069052294800
}
```
- to get ALL balances

```
$ curl -s  http://localhost:8080/balances | jq
[
  {
    "address": "0x06cfa9ad90b141e59b27e8fe0814dfeca8897e367293366aa167e9ddd3d3c9e5",
    "coin": "0x15b41d2aa19733d3e10cb27198f6744cc04f8ee5c5f5d74a3ba68d2130e8069f::vapor::VAPOR",
    "amount": 100000000000
  },
  {
    "address": "0x14772446d188d6b6cffdb3ae3b197c9d8d96d992d382e1937f08ca3832160456",
    "coin": "0x2::sui::SUI",
    "amount": -2331664
  },
  {
    "address": "0x3f3b11a18ffe59368cb935771df277ac531bf60b9a0a201c78e9d1aabe7bc214",
    "coin": "0x2::sui::SUI",
    "amount": -6156144
  },
  {
    "address": "0x660ea6bc10f2d6c2d40b829850ab746a6ad93c2674537c71e21809b0486254c6",
    "coin": "0x2::sui::SUI",
    "amount": -14849088
  },
  {
    "address": "0x751fa304ebd8623a820d6e3af2b6db69c6c6fc6708634a3090781cbad8b8ee06",
    "coin": "0x2::sui::SUI",
    "amount": -2373928
  },
  {
    "address": "0x7d20dcdb2bca4f508ea9613994683eb4e76e9c4ed371169677c1be02aaf0b58e",
    "coin": "0x2::sui::SUI",
    "amount": -3708677707960
  },
  {
    "address": "0x8f04773c6c07615ab1a17a8b078f78de7b52b11c423ee8574b599524f82d5419",
    "coin": "0x2::sui::SUI",
    "amount": 3705584111080
  },
  {
    "address": "0xbb217b40ac041f09f683daa3ca8e4927d869acf8c7f546441550913cb5341c75",
    "coin": "0x2::sui::SUI",
    "amount": 2000000000
  },
  {
    "address": "0xc9c8e0d738d7f090144847b38a8283fbe8050923875771b8c315a461721c04a4",
    "coin": "0x2::sui::SUI",
    "amount": -2332272
  },
  {
    "address": "0xd8370b8e732671933bd2c62b1bdb1eff71647765127b3e4cb9a10f1ee8dcfafa",
    "coin": "0x2::sui::SUI",
    "amount": -2703380
  },
  {
    "address": "0xf5ad86df467d1d945072aafc047413b8cf9e6237f9505ea63c3612ee7ecf0306",
    "coin": "0x2::sui::SUI",
    "amount": 1000000000
  }
]

```
