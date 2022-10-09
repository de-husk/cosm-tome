# Cosm-Client


// TODO: Figure out the public API interface for 3 of the main tx submodules before actually implementing the details
// <--
// <--
// + cosmwasm
// * account (only query)
// * bank (query and txs)

Easy to use, high level rust Cosmos SDK client rust library.

## Inspiration

The Cosmos SDK [already has a lot of different APIs](https://github.com/cosmos/cosmos-sdk/blob/main/docs/core/grpc_rest.md). So this library supports plugging in different backing APIs (Cosmos gRPC, Cosmos REST, Tendermint RPC, etc). We hide this complexity away from the cosmos modules (cosmwasm, account, bank) only exposing the same unified `CosmosClient` trait to all of them.

As more APIs are added to Cosmos SDK, we will simply add a new `CosmosClient` implementation file keeping the API module code untouched.

## Usage

TODO


