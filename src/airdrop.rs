/*

Airdrop
-------

- from snapshot
- from list of addresses
- define from-address
- define percentage / fixed amount to airdrop
- define which coin to airdrop (KMD or assetchain)

- z-support?
*/

pub struct Airdrop {

}

impl Airdrop {
    pub fn builder() -> AirdropBuilder {

    }

    pub fn print_string() {
        // should print a string to sign
        // multisig should include P2SH inputs.
    }
}

pub struct AirdropBuilder {

}

impl AirdropBuilder {
    pub fn using_file(&mut self, path: &Path) -> &mut Self {


        self
    }

    pub fn using_snapshot(&mut self, snapshot: Snapshot) -> &mut Self {


        self
    }

    pub fn source_address(&mut self, source: &str) -> &mut Self {


        self
    }

    pub fn create() -> Airdrop {

    }
}