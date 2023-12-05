use serde::{Deserialize, Serialize};

use crate::{football_api::ClubInfo, format_string, Error};

#[cfg(feature = "cli")]
use clap::{CommandFactory, Parser};

#[cfg(feature = "cli")]
use tokio::io::{stdout, AsyncWriteExt};

use crate::{config::Config, ApiStringType, StringType};

#[cfg(feature = "cli")]
use crate::football_api::FootballApi;

/// Utility to retrieve and format football data from api-football.com
///
/// Please specify the `club_id`
#[cfg(feature = "cli")]
#[derive(Parser, Default, Serialize, Deserialize)]
pub struct FootballOpts {
    /// Api key (optional but either this or API_KEY environment variable must
    /// exist)
    #[clap(short = 'k', long)]
    api_key: Option<ApiStringType>,

    /// Next match (optional)
    #[clap(long)]
    next_match: Option<u8>,

    /// Club id (optional)
    #[clap(short, long)]
    club_id: Option<u16>,
}

#[cfg(feature = "cli")]
impl FootballOpts {
    /// Parse options from stdin, requires `Config` instance.
    /// # Errors
    ///
    /// Returns error if call to retreive football data fails or if write to
    /// stdout fails
    pub async fn parse_opts(config: &Config) -> Result<(), Error> {
        let mut opts = Self::parse();
        opts.apply_defaults(config);

        let mut stdout = stdout();

        for output in opts.run_opts(config).await? {
            stdout.write_all(output.as_bytes()).await?;
        }

        Ok(())
    }

    /// # Errors
    /// Return Error if api key cannot be found
    #[cfg(feature = "cli")]
    fn get_api(&self, config: &Config) -> Result<FootballApi, Error> {
        let api_key = self
            .api_key
            .as_deref()
            .ok_or_else(|| Error::InvalidInputError(format_string!("invalid api key")))?;

        Ok(FootballApi::new(
            api_key,
            &config.api_endpoint,
            &config.api_path,
        ))
    }

    /// Extract options from `FootballOpts` and apply to `FootballApi`
    /// # Errors
    /// Returns Error if clap help output fails
    pub fn get_club(&self, default_club_id: u16) -> Result<ClubInfo, Error> {
        let club = if let Some(club_id) = self.club_id {
            if let Some(next_match) = self.next_match {
                ClubInfo::from_parameter(club_id, next_match, "".into())
            } else {
                ClubInfo::from_parameter(club_id, 0, "all".into())
            }
        } else if self.club_id.is_none() {
            if let Some(next_match) = self.next_match {
                ClubInfo::from_parameter(default_club_id, next_match, "".into())
            } else {
                ClubInfo::from_parameter(default_club_id, 0, "all".into())
            }
        } else {
            return Err(Error::InvalidInputError(format_string!(
                "\nERROR: You must specify the correct value\n"
            )));
        };

        Ok(club)
    }

    /// # Errors
    ///
    /// Returns error if call to retreive football data fails
    async fn run_opts(&self, config: &Config) -> Result<Vec<StringType>, Error> {
        let api = self.get_api(config)?;
        let club = self.get_club(config.club_id)?;

        let data = api.get_fixture_data(&club).await?;

        let output = vec![data.get_current_fixtures()];
        Ok(output)
    }

    fn apply_defaults(&mut self, config: &Config) {
        if self.api_key.is_none() {
            self.api_key = config.api_key.clone();
        }

        if self.club_id.is_none() {
            self.club_id = Some(config.club_id);
        }
    }

    #[must_use]
    pub fn api_help_msg() -> StringType {
        format_string!("{}", Self::command().render_help())
    }
}
