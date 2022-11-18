# Cosm-Tome

[![cosm-tome on crates.io](https://img.shields.io/crates/v/cosm-tome.svg)](https://crates.io/crates/cosm-tome) [![Docs](https://docs.rs/cosm-tome/badge.svg)](https://docs.rs/cosm-tome)

Easy to use, high level Cosmos SDK rust client library.

## Inspiration

The Cosmos SDK [already has a lot of different APIs](https://github.com/cosmos/cosmos-sdk/blob/main/docs/docs/core/06-grpc_rest.md). So this library supports plugging in different backing APIs (Cosmos gRPC, Cosmos REST, Tendermint RPC, etc). We hide this complexity away from the cosmos modules (cosmwasm, auth, bank) only exposing the same unified `CosmosClient` trait to all of them.

As more APIs are added to Cosmos SDK, we will simply add a new `CosmosClient` implementation file keeping the cosmos module code untouched.

## Crate Status

### Clients

| Backing API | Dev Status |
| ------------- | ------------- | 
| Tendermint RPC | 🔨 |
| Cosmos SDK gRPC | 🔨 | 
| Cosmos SDK REST | 🚫 |

### Modules

| Cosmos Module | Dev Status |
| ------------- | ------------- | 
| Auth | ✅ |
| Authz | 🚫 |
| Bank | ✅ |
| Tendermint | 🔨 |
| Crisis | 🚫 |
| Distribution | 🚫 |
| Evidence | 🚫 |
| Feegrant | 🚫 |
| Gov | 🚫 |
| Mint | 🚫 |
| Params | 🚫 |
| Slashing | 🚫 |
| Staking | 🚫 |
| Tx | 🔨 |
| Upgrade | 🚫 |
| Vesting | 🚫 |
| CosmWasm | 🔨 |
| IBC | 🚫 |


## Usage

TODO


## Development 

`cargo t` to run the unit tests.

