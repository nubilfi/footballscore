use envy::Error as EnvyError;
use serde_json::Error as SerdeJsonError;
use std::{fmt::Error as FmtError, io::Error as IoError};
use thiserror::Error as ThisError;
use tokio::task::JoinError;
use url::ParseError as UrlParseError;

#[cfg(feature = "cli")]
use clap::Error as ClapError;

#[cfg(feature = "cli")]
use reqwest::Error as ReqwestError;

#[cfg(feature = "cli")]
use reqwest::header::InvalidHeaderValue;

use crate::StringType;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("Format Error {0}")]
    FmtError(#[from] FmtError),

    #[error("Environment Parsing Error {0}")]
    EnvyError(#[from] EnvyError),

    #[error("URL Parse Error {0}")]
    UrlParseError(#[from] UrlParseError),

    #[error("JSON Serde Error {0}")]
    SerdeJsonError(#[from] SerdeJsonError),

    #[error("IO Error {0}")]
    IoError(#[from] IoError),

    #[error("Invalid Value Error {0}")]
    InvalidValue(StringType),

    #[error("Invalid Input Error {0}")]
    InvalidInputError(StringType),

    #[cfg(feature = "cli")]
    #[error("Clap CLI Parser Error {0}")]
    ClapError(#[from] ClapError),

    #[cfg(feature = "cli")]
    #[error("Reqwest Error {0}")]
    ReqwestError(#[from] ReqwestError),

    #[cfg(feature = "cli")]
    #[error("Invalid Header Value {0}")]
    InvalidHeaderValue(#[from] InvalidHeaderValue),

    #[cfg(feature = "cli")]
    #[error("Task Join Error {0}")]
    JoinError(#[from] JoinError),
}
