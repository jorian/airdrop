Komodo Airdrop
==============

This Airdrop crate makes it fairly easy to perform an airdrop from a Komodo Platform Assetchain, using KMD or any other assetchain.

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
        .source_address("RQT7m4jcnWQxwqQQzh77WKNCuZotkRkAuk")
        .payout_ratio(0.75)
        .build()
        .unwrap();
    }
```

##### notes:
- ratio is applied to both balance and interest. any change includes interest against the same ratio

##### todo:

- [ ] documentation
- [x] define static payout amount in addition to ratio
- [ ] let the builder pattern work with Results (trait type?)
- [x] send back any remainders and/or interest to fund_address
- [x] add P2SH inputs to support multisig airdrop
- [x] airdrop take a reference to snapshot
- [ ] use JSON file as input to Airdrop
- [ ] only include necessary utxos as input to the transaction, to prevent using all utxos when it's not needed since the required amount to airdrop has been reached.
    - [ ] what about interest? if KMD && interest_included then spend all inputs to include all interest.
    - [ ] else spend only required utxos
        - with some kind of ordering of utxos to only use largest utxos
- [ ] use a global komodod daemon client instead of instantiating it a couple of times.

##### long term maybe's:
- [ ] serialize multisig raw tx for easy multiparty signing