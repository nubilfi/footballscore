#[cfg(feature = "cli")]
use footballscore::{config::Config, football_opts::FootballOpts, Error};

#[cfg(not(tarpaulin_include))]
#[cfg(feature = "cli")]
#[tokio::main]
async fn main() -> Result<(), Error> {
    let config = Config::init_config(None)?;

    match tokio::spawn(async move { FootballOpts::parse_opts(&config).await })
        .await
        .unwrap()
    {
        Ok(()) => Ok(()),
        Err(Error::InvalidInputError(e)) => {
            let help_message = FootballOpts::api_help_msg();
            eprintln!("{e}\n{help_message}");
            Ok(())
        }
        Err(Error::ReqwestError(req_err)) => {
            match req_err.url() {
                Some(_) => {
                    eprintln!("Network Request Error");
                }
                None => {
                    eprintln!("Invalid API Request");
                }
            }
            Ok(())
        }
        Err(e) => Err(e),
    }
}

#[cfg(not(tarpaulin_include))]
#[cfg(not(feature = "cli"))]
fn main() -> Result<(), Error> {
    Ok(())
}
