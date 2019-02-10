### Basic multisig wallet
_Exonum implementation of single-noded simple cryptocurrency with multisignature wallet_

#### Implementation details
Multisignature transactions are implemented in the way that wallets hold pending transactions vector that will be executed as soon as 2/3 majority is reached. Pending transactions, in they turn, contain all approvals in respective vector.
Multisignature is an optional feature, which is enabled only if you have added signer to a personal wallet

#### Running
To run a node itself, clone this repository and perform `cargo run` in the root of it.


#### Wallets API

Endpoint root: `/api/services/cryptocurrency/v1`

------
`GET /wallet?pub_key=<string>`: Get a wallet by a public key.

------
`GET /wallet/txs?pub_key=<string>`: Get approved transactions of a wallet by public key.

------
`GET /wallets`: Get all wallets in network.

#### Transactions API

Transaction API is a stardard Exonum Explorer API
Endpoint root: `api/explorer/v1`

------
`POST /transactions`: Broadcast transaction to the network

Endpoint expects to receive serialized signed transaction as hex string
```
{
    "tx_body": string
}
```

There are 4 types of transaction:
- Create wallet
```
{
    "name": string
}
```

- Add signer
```
{
    "signer": public key
}
```

- Transfer funds
```
    "recepient": public key,
    "amount": number,
    "seed": number
}
```

- Sign pending transaction
```
{
    "origin": public key,
    "tx_hash": hash
}
```