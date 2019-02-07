### Basic multisig wallet
_Exonum implementation of single-noded simple cryptocurrency with multisignature wallet_

#### Implementation details
Multisignature transactions are implemented in the way that wallets hold pending transactions vector that will be executed as soon as 2/3 majority is reached. Pending transactions, in they turn, contain all approvals in respective vector.

#### Running
To run a node itself, clone this repository and perform `cargo run` in the root of it.

#### Unimplemented
- Transaction index is currently mocked in returned result after transaction added to 'pending' vector
- Unit testing

#### API endpoints

Endpoint root: `/api/services/cryptocurrency/v1`

------
`GET /wallet?pub_key=<string>`: Get a wallet by a public key.

------
`GET /wallets`: Get all wallets in network.

------
`POST /wallets`: Create a wallet with specified public key and name.
Body:
```
{
    pub_key: <string>,      // Public key of wallet to be created with
    name: <number>          // Name of the wallet
}
```

------
`POST /wallets/signers`: Add a trusted signer to wallet.
Body:
```
{
    pub_key: <string>,      // Wallet address to add signer for
    signer: <string>        // Public key to be added as trusted signer
}
```

------
`POST /wallets/transfer`: Send a coin amount to specified public key.
Body:
```
{
    sender: <string>,       // Sender of funds
    recipient: <string>,    // Receiver of funds
    amount: <number>,       // Amount of coins to send
    seed: <number>          // Unique value
}
```

------
`POST /wallets/sign`: Sign a pending transaction in wallet.
Body:
```
{
    sender: <string>,       // Signer of transaction
    origin: <string>,       // Wallet, where pending transaction are stored
    tx_index: <number>      // Index of transaction in pending_txs vector
}
```
