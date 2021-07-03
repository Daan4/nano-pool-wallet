# nano-pool-wallet

[![Rust](https://github.com/Daan4/nano-pool-wallet/actions/workflows/rust.yml/badge.svg)](https://github.com/Daan4/nano-pool-wallet/actions/workflows/rust.yml)

**Proof of concept; work in progress; no guarantees given as per the license; should only be used for educational purposes**

Also check out the Feeless project: https://github.com/feeless/feeless , which was a helpful resource for creating this project. Specifically for figuring out how to do the key derivations using Rust.

## Introduction

A nano currency wallet that consists of one main account and a pool of accounts. Pool accounts are used for transactions going in/out of the main account. A pool address is reserved for each specific transaction, either for a send of any amount, for a receive of any amount, or for a receive of a specific amount. Any unexpected balances found in pool accounts are automatically refunded to the sender.

## Sending

The balance is sent from the main account to a free pool account. If no free pool account exists a new one will be generated. Then the balance is sent from the pool account to the destination address. Finally the pool account is freed to be used for other transactions.

## Receiving any amount

A free pool account is reserved, if no free account exists a new one will be generated. The reserved pool account address should be shared with the sender. As soon as any amount is received it is sent to the main account and the pool account will be freed. If no amount is received within a given time a timeout will occur and the pool account is freed. Any transaction received after that will be refunded as usual.

## Receiving a specific amount

A free pool account is reserved, if no free account exists a new one will be generated. The reserved pool account address should be shared with the sender. As soon as the specified amount is received it is sent to the main account and the pool account will be freed. Any amounts receives besides the specified amount are refunded. If the amount is not received within a given time a timeout will occur and the pool account is freed. Any transaction received after that will be refunded as usual.

## Refunding unexpected transactions

All pool accounts have their incoming balances monitored. If an unexpected balance is received it is immediately returned to the sender.
