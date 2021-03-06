// Copyright 2019 TiKV Project Authors. Licensed under Apache-2.0.

use std::fmt::{self, Display, Formatter};
use std::{error, result};

use crate::call::RpcStatus;
use crate::grpc_sys::grpc_call_error;

#[cfg(feature = "prost-codec")]
use prost::DecodeError;
#[cfg(feature = "protobuf-codec")]
use protobuf::ProtobufError;

/// Errors generated from this library.
#[derive(Debug)]
pub enum Error {
    /// Codec error.
    Codec(Box<dyn error::Error + Send + Sync>),
    /// Failed to start an internal async call.
    CallFailure(grpc_call_error),
    /// Rpc request fail.
    RpcFailure(RpcStatus),
    /// Try to write to a finished rpc call.
    RpcFinished(Option<RpcStatus>),
    /// Remote is stopped.
    RemoteStopped,
    /// Failed to shutdown.
    ShutdownFailed,
    /// Failed to bind.
    BindFail(String, u16),
    /// gRPC completion queue is shutdown.
    QueueShutdown,
    /// Failed to create Google default credentials.
    GoogleAuthenticationFailed,
    /// Invalid format of metadata.
    InvalidMetadata(String),
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        write!(fmt, "{:?}", self)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Codec(_) => "gRPC Codec Error",
            Error::CallFailure(_) => "gRPC Call Error",
            Error::RpcFailure(_) => "gRPC Request Error",
            Error::RpcFinished(_) => "gRPC Finish Error",
            Error::RemoteStopped => "Remote is stopped.",
            Error::ShutdownFailed => "Failed to shutdown.",
            Error::BindFail(_, _) => "gRPC Bind Error",
            Error::QueueShutdown => "gRPC completion queue shutdown",
            Error::GoogleAuthenticationFailed => "Could not create google default credentials.",
            Error::InvalidMetadata(_) => "invalid format of metadata",
        }
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            Error::Codec(ref e) => Some(e.as_ref()),
            _ => None,
        }
    }

    // TODO: impl `Error::source`, but it may break backward compatibility,
    // Eg. TiKV still uses nightly-2018-07-18, which does not compile.
}

#[cfg(feature = "protobuf-codec")]
impl From<ProtobufError> for Error {
    fn from(e: ProtobufError) -> Error {
        Error::Codec(Box::new(e))
    }
}

#[cfg(feature = "prost-codec")]
impl From<DecodeError> for Error {
    fn from(e: DecodeError) -> Error {
        Error::Codec(Box::new(e))
    }
}

/// Type alias to use this library's [`Error`] type in a `Result`.
pub type Result<T> = result::Result<T, Error>;

#[cfg(all(test, feature = "protobuf-codec"))]
mod tests {
    use std::error::Error as StdError;

    use protobuf::error::WireError;
    use protobuf::ProtobufError;

    use super::Error;

    #[allow(deprecated)]
    #[test]
    fn test_convert() {
        let error = ProtobufError::WireError(WireError::UnexpectedEof);
        let e: Error = error.into();
        assert_eq!(e.description(), "gRPC Codec Error");
        assert!(e.cause().is_some());
    }
}
