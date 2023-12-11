#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::similar_names)]

//! Utility to retreive football score from api-football.com
//!
//! ```bash
//! Please specify club_id to show your favorite club match information. You can get the ID of your
//! favorite club from club_name option.
//!
//! USAGE:
//! footballscore [OPTIONS]
//!
//! FLAGS:
//! -h, --help      Prints help information
//! -V, --version   Prints version information
//!
//! OPTIONS:
//! -k, --api-key <api-key>             Api key (optional but either this or API_KEY environment variable must exist)
//!     --next-match <next-match>       Show next match (optional), but it must be `1`
//! -n, --club-name <club-name>         Get your favorite Club ID by name (optional)
//! -c, --club-id <club-id>             Your favorite Club ID (optional), if not specified `529 (Barcelona)` will be assumed

/// Configuration data
pub mod config;

/// Reqwest Client
pub mod football_api;

/// Representation of Football Fixtures Data from api-football.com
pub mod football_fixtures_data;

/// Representation of Football Teams Data from api-football.com
pub mod football_teams_data;

/// CLI App Options and implementation
pub mod football_opts;

/// `FootballUtil` Error
pub mod error;
pub use error::Error;

// -------- FEATURE --------
#[cfg(feature = "stackstring")]
use stack_string::{SmallString, StackString};

#[cfg(feature = "stackstring")]
pub type StringType = StackString;

#[cfg(feature = "stackstring")]
pub type ApiStringType = SmallString<32>;

#[cfg(feature = "stackstring")]
pub fn apistringtype_from_display(buf: impl std::fmt::Display) -> ApiStringType {
    SmallString::from_display(buf)
}

#[cfg(feature = "stackstring")]
#[macro_export]
macro_rules! format_string {
    ($($arg:tt)*) => {
        {
            use std::fmt::Write;
            let mut buf = stack_string::StackString::new();

            write!(buf, "{}", std::format_args!($($arg)*)).unwrap();
            buf
        }
    };
}

// -------- NOT FEATURE --------
#[cfg(not(feature = "stackstring"))]
pub type StringType = String;

#[cfg(not(feature = "stackstring"))]
pub type ApiStringType = String;

#[cfg(not(feature = "stackstring"))]
pub fn apistringtype_from_display(buf: impl std::fmt::Display) -> ApiStringType {
    format!("{buf}")
}

#[cfg(not(feature = "stackstring"))]
#[macro_export]
macro_rules! format_string {
    ($($arg:tt)*) => {
        {
            use std::fmt::Write;
            let mut buf = String::new();

            write!(buf, "{}", std::format_args!($($arg)*)).unwrap();
            buf
        }
    };
}
