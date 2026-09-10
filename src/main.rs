#[cfg(feature = "cli")]
use footballscore::{config::Config, football_opts::FootballOpts, Error};

#[cfg(not(tarpaulin_include))]
#[cfg(feature = "cli")]
#[allow(clippy::disallowed_methods)]
#[tokio::main]
async fn main() -> Result<(), Error> {
    let config = Config::init_config(None)?;

    match tokio::spawn(async move { FootballOpts::parse_opts(&config).await }).await? {
        Ok(()) => Ok(()),
        Err(Error::InvalidInputError(e)) => {
            let help_message = FootballOpts::api_help_msg();
            eprintln!("{e}\n{help_message}");
            Ok(())
        }
        Err(Error::ApiError { status, message }) => {
            match status {
                // football-data reports a bad token as 400 with its own message
                400 | 401 | 403 => {
                    if message.is_empty() {
                        eprintln!("Invalid or missing API key/auth token");
                    } else {
                        eprintln!("{message}");
                    }
                }
                404 => eprintln!("Resource not found on the API"),
                429 => eprintln!("Rate limited by the API, try again shortly"),
                _ => {
                    if message.is_empty() {
                        eprintln!("Network Request Error");
                    } else {
                        eprintln!("{message}");
                    }
                }
            }
            Ok(())
        }
        Err(Error::ReqwestError(req_err)) => {
            if footballscore::football_data::debug_enabled() {
                eprintln!("[debug] {req_err}");
            }

            match req_err.status().map(|s| s.as_u16()) {
                // the free API answers a bad token with 401 rather than a body
                Some(401 | 403) => eprintln!("Invalid or missing API key/auth token"),
                Some(404) => eprintln!("Resource not found on the API"),
                Some(429) => eprintln!("Rate limited by the API, try again shortly"),
                _ => match req_err.url() {
                    Some(_) => eprintln!("Network Request Error"),
                    None => eprintln!("Invalid API Request"),
                },
            }
            Ok(())
        }
        // the provider's own message, e.g. "Invalid token." or a throttling notice
        Err(Error::InvalidValue(detail)) => {
            eprintln!("{detail}");
            Ok(())
        }
        Err(e) => {
            // never show a raw Rust error; the detail is available on request
            eprintln!("Unable to retrieve data, set FOOTBALLSCORE_DEBUG=1 for details");

            if footballscore::football_data::debug_enabled() {
                eprintln!("[debug] {e}");
            }

            Ok(())
        }
    }
}

#[cfg(not(tarpaulin_include))]
#[cfg(not(feature = "cli"))]
fn main() -> Result<(), Error> {
    Ok(())
}
