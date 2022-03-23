use std::error::Error;
use std::io;

use komodo_rpc_client::ApiError;
use komodo_snapshot::SnapshotError;

#[derive(Debug, Display)]
#[display(fmt = "{}", kind)]
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
    #[display(fmt = "An I/O error occured.")]
    Io(io::Error),
    #[display(fmt = "Something went wrong during the komodod RPC.")]
    ApiError(komodo_rpc_client::ApiError),
    #[display(fmt = "Something went wrong during a snapshot.")]
    SnapshotError(komodo_rpc_client::ApiError),
    #[display(fmt = "Serde error.")]
    Serde(serde_json::error::Error),
    #[display(fmt = "Ambiguous setup.")]
    AmbiguousConfig(String),
    #[display(fmt = "undefined.")]
    NotDefined(String),
    Other(String),

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

impl From<SnapshotError> for AirdropError {
    fn from(e: SnapshotError) -> Self {
        ErrorKind::SnapshotError(e).into()
    }
}

impl From<ApiError> for AirdropError {
    fn from(e: ApiError) -> Self {
        ErrorKind::ApiError(e).into()
    }
}

impl From<serde_json::Error> for AirdropError {
    fn from(e: serde_json::Error) -> Self {
        ErrorKind::Serde(e).into()
    }
}