Komodo Airdrop
==============

This Airdrop crate makes it easy to perform an airdrop from a Komodo Platform Assetchain, using KMD or any other assetchain.

```rust
extern crate komodo_airdrop;    

fn main() {    
    let snapshot = komodo_airdrop::Snapshot::builder()
        .on_chain(Chain::CHAIN)
        .exclude_addresses(vec![
            String::from("REbwjC5mfQBRevH78CuRjcvQudxa3ii5Ma"),
            String::from("RU9F2EdDzUNK4LUMgjLTMDmtCmDK1a9vrY"),
            String::from("RYEufBcEUsofxwt4bGUdroRGmAQxBR8aJG")
        ])
        .using_threshold(1.0) // only include addresses that contain at least 1 CHAIN
        .build()
        .unwrap();
    
    let airdrop = komodo_airdrop::Airdrop::builder()
        .using_chain(Chain::KMD)
        .include_interest(true)
        .using_snapshot(&snapshot)
        .fund_address("RQT7m4jcnWQxwqQQzh77WKNCuZotkRkAuk")
        .payout_ratio(0.75)
        .build()
        .unwrap();
}
```

If an airdrop happens from a multisig address (starts with `b`):

```rust
let signing_string = airdrop.signing_string(Some(String::from("<redeem_script here>")));
println!("{}", signing_string);

```


If not:

```rust
let signing_string = airdrop.signing_string(None);
println!("{}", signing_string);   
```
    
`signing_string` creates a raw transaction from the inputs of all utxos in the `fund_address`, and uses the resulting hex (a running daemon of the fund_address blockchain is required)
to create a string that can be used as parameter string for the `signrawtransaction` daemon RPC, where in case of a multisig, a private key (WIF) needs to be supplied manually by the signer.

#### notes:
- ratio is applied to both balance and interest. any change includes interest against the same ratio

#### todo:

- [ ] documentation
    - [x] why this airdrop crate is needed: KMD platform addresses are the same etc, so the only differentiator is an AC
- [x] define static payout amount in addition to ratio
- [x] ~~let the builder pattern work with Results (trait type?)~~  impossible, checking is done in `build()` now
- [x] send back any remainders and/or interest to fund_address
- [x] add P2SH inputs to support multisig airdrop
- [x] airdrop take a reference to snapshot
- [ ] use JSON file as input to Airdrop
- [ ] only include necessary utxos as input to the transaction, to prevent using all utxos when it's not needed since the required amount to airdrop has been reached.
    - [ ] what about interest? if KMD && interest_included then spend all inputs to include all interest.
    - [ ] else spend only required utxos
        - with some kind of ordering of utxos to only use largest utxos
- [ ] use a global komodod daemon client instead of instantiating it a couple of times.
- [ ] use an Enum for `ratio` and `amount` in `Airdrop`
- [ ] Enum: add nonexhaustive to not have a breaking change when adding a new Enum variant

##### scenarios tested

- [ ] Multisig yes/no
- [ ] Ratio
- [ ] Amount
    - payout full amount
- [ ] Interest included


#### long term maybe's:
- [ ] serialize multisig raw tx for easy multiparty signing
    - would likely be separate crate