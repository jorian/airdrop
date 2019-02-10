use derive_more;

use std::error::Error;
use std::fmt;
use std::io;

pub struct AirdropError {
    pub kind: ErrorKind,
    source: Option<Box<dyn Error + Send + Sync + 'static>>
}

#[derive(Debug, Display)]
pub enum ErrorKind {
    #[display(fmt = "The snapshot returned no addresses.")]
    EmptySnapshot,
    #[display(fmt = "Not enough balance in source address.")]
    BalanceInsufficient,
    #[display(fmt = "Not enough balance in source address.")]
    Io(io::Error),

    // todo nonexhaustive to not have a breaking change when adding an error type
}

impl Error for AirdropError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_ref()
            .map(|boxed| boxed.as_ref() as &(dyn Error + 'static))
    }
}

impl From<ErrorKind> for AirdropError {
    fn from(kind: ErrorKind) -> Self {
        AirdropError {
            kind,
            source: None
        }
    }
}