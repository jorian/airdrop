mod airdrop;
mod snapshot;

use core::fmt;

#[derive(Debug)]
pub enum Chain {
    KMD     ,
    REVS    ,
    SUPERNET,
    DEX     ,
    PANGEA  ,
    JUMBLR  ,
    BET     ,
    CRYPTO  ,
    HODL    ,
    MSHARK  ,
    BOTS    ,
    MGW     ,
    COQUI   ,
    WLC     ,
    KV      ,
    CEAL    ,
    MESH    ,
    MNZ     ,
    AXO     ,
    ETOMIC  ,
    BTCH    ,
    PIZZA   ,
    BEER    ,
    NINJA   ,
    OOT     ,
    BNTN    ,
    CHAIN   ,
    PRLPAY  ,
    DSEC    ,
    GLXT    ,
    EQL     ,
    ZILLA   ,
    RFOX    ,
    SEC     ,
    CCL     ,
    PIRATE  ,
    MGNX    ,
    PGT     ,
    KMDICE  ,
    DION    ,
    ZEX     ,
    KSB     ,
    OUR     ,
}

impl fmt::Display for Chain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
