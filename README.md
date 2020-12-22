# Tangleproof

Library to send transactions and create a proof object for them which can be used at any later time to proof that a transaction existet in the Tangle (also called proof of inclusion), as long as the funds weren't moved with another wallet/lib.

Create a `.env` with the following values

```
node_url="http://api.lb-0.testnet.chrysalis2.com"
proof_name=proof_example.json
amount=1000
seed=3ff69866a124d8cf168e5b928eb603bacc2d241f1a9d70af5c10f2dd34137896
local_pow=false
```

Run example with `cargo run --release`

Change values in config.json if you want to send your own transactions

The first time you need to send the amount of iotas specified in config.json to the address shown in the console, so the output can be used.


For a proof to be valid one output of a transaction always needs to be used as input in the next transaction and the latest output needs to be unspent.
The output is not be available before the transaction is confirmed, so it can take a few seconds before a new proof is valid.

Con: You need IOTA token as long as you want to prove the existence of your transaction(s)

Pro: You only need to store your own transactions, no need to constantly listen to new transactions in the Tangle