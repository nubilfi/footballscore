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
/// Please specify the `club_id` or use `club_name` to get its ID
#[cfg(feature = "cli")]
#[derive(Parser, Default, Serialize, Deserialize)]
pub struct FootballOpts {
    /// Api key (optional but either this or API_KEY environment variable must exist)
    #[clap(short = 'k', long)]
    api_key: Option<ApiStringType>,

    /// Next match (optional)
    #[clap(long)]
    next_match: Option<u8>,

    /// Club id (optional)
    #[clap(short = 'c', long)]
    club_id: Option<u16>,

    /// Club name (optional)
    #[clap(short = 'n', long)]
    club_name: Option<StringType>,
}

#[cfg(feature = "cli")]
impl FootballOpts {
    /// Parse options from stdin, requires `Config` instance.
    /// # Errors
    ///
    /// Returns error if call to retreive football data fails or if write to stdout fails
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

        Ok(FootballApi::new(api_key, &config.api_endpoint))
    }

    /// Extract options from `FootballOpts` and apply to `FootballApi`
    /// # Errors
    /// Returns Error if clap help output fails
    pub fn get_club(&self, default_club_id: u16, club_name: &str) -> Result<ClubInfo, Error> {
        let club = if let Some(club_id) = self.club_id {
            if let Some(next_match) = self.next_match {
                ClubInfo::from_parameter(club_id, next_match, "".into(), club_name.into())
            } else {
                ClubInfo::from_parameter(club_id, 0, "all".into(), club_name.into())
            }
        } else if self.club_id.is_none() {
            if let Some(next_match) = self.next_match {
                ClubInfo::from_parameter(default_club_id, next_match, "".into(), club_name.into())
            } else {
                ClubInfo::from_parameter(default_club_id, 0, "all".into(), club_name.into())
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

        if let Some(name) = &self.club_name {
            let club = self.get_club(config.club_id, name)?;
            let data = api.get_team_data(&club).await?;

            let output = vec![data.get_teams_information()];
            return Ok(output);
        }

        let club: ClubInfo = self.get_club(config.club_id, "")?;
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

#[cfg(test)]
mod tests {
    use log::info;
    use std::env::set_var;

    use crate::{
        config::{Config, TestEnvs},
        football_api::ClubInfo,
        Error,
    };

    #[cfg(feature = "cli")]
    use crate::football_opts::FootballOpts;

    #[test]
    fn test_api_help_msg() -> Result<(), Error> {
        let msg = FootballOpts::api_help_msg();
        assert!(msg.len() > 0);
        Ok(())
    }

    #[cfg(feature = "cli")]
    #[test]
    fn test_get_api() -> Result<(), Error> {
        let _env = TestEnvs::new(&["API_KEY", "API_ENDPOINT", "CLUB_ID"]);

        set_var("API_KEY", "1e5765fc0c22df4e4ccf20581c2ef3d7");
        set_var("API_ENDPOINT", "test.local");
        set_var("CLUB_ID", "529");

        let config = Config::init_config(None)?;
        drop(_env);

        let mut opts = FootballOpts::default();
        opts.apply_defaults(&config);
        let api = opts.get_api(&config)?;

        assert_eq!(
            format!("{api:?}"),
            "FootballApi(key=1e5765fc0c22df4e4ccf20581c2ef3d7,endpoint=test.local)".to_string()
        );

        let endpoint_fixtures = opts.get_club(529, "")?;
        let live = "StackString(\"all\")";
        let name = "StackString(\"\")";
        let expected = format!(
            "EndpointParams {{ team: 529, next: 0, live: {}, name: {} }}",
            live, name
        );

        assert_eq!(format!("{endpoint_fixtures:?}"), expected);

        let endpoint_teams = opts.get_club(0, "arsenal")?;
        let live = "StackString(\"all\")";
        let name = "StackString(\"arsenal\")";
        let expected = format!(
            "EndpointParams {{ team: 529, next: 0, live: {}, name: {} }}",
            live, name
        );

        assert_eq!(format!("{endpoint_teams:?}"), expected);
        Ok(())
    }

    #[test]
    fn test_apply_defaults() -> Result<(), Error> {
        let _env = TestEnvs::new(&["API_KEY", "API_ENDPOINT", "CLUB_ID"]);

        set_var("API_KEY", "1e5765fc0c22df4e4ccf20581c2ef3d7");
        set_var("API_ENDPOINT", "test.local");
        set_var("CLUB_ID", "529");

        let config = Config::init_config(None)?;
        drop(_env);

        let mut opts = FootballOpts::default();
        opts.apply_defaults(&config);

        assert_eq!(opts.club_id, Some(529));
        assert_eq!(opts.club_name, None);
        assert_eq!(opts.next_match, None);
        Ok(())
    }

    #[cfg(feature = "cli")]
    #[tokio::test]
    async fn test_run_opts() -> Result<(), Error> {
        let _env = TestEnvs::new(&["API_KEY", "API_ENDPOINT", "CLUB_ID"]);

        let config = Config::init_config(None)?;
        drop(_env);

        let mut opts = FootballOpts::default();
        opts.club_name = Some("arsenal".into());
        opts.apply_defaults(&config);

        let output = opts.run_opts(&config).await?;
        assert_eq!(output.len(), 1);
        info!("{:#?}", output);
        assert!(
            output[0].contains("Error: token - Error/Missing application key. Go to https://www.api-football.com/documentation-v3 to learn how to get your API application key.")
            || output[0].contains("Name:")
        );

        opts.club_name = None;
        opts.club_id = Some(529);
        opts.next_match = Some(1);
        let output = opts.run_opts(&config).await?;
        info!("{:#?}", output);
        assert!(
            output[0].contains("Error: token - Error/Missing application key. Go to https://www.api-football.com/documentation-v3 to learn how to get your API application key.")
            || output[0].contains("Match: Barcelona 0 vs")
        );

        Ok(())
    }

    #[test]
    fn test_get_fixtures() -> Result<(), Error> {
        // next fixture
        let mut opts = FootballOpts::default();
        opts.club_id = Some(529);
        opts.next_match = Some(1);
        let club = opts.get_club(opts.club_id.unwrap_or_default(), "")?;

        assert_eq!(
            club,
            ClubInfo::EndpointParams {
                team: 529,
                next: 1,
                live: "".into(),
                name: "".into()
            }
        );

        // live fixture
        let mut opts = FootballOpts::default();
        opts.club_id = Some(529);
        let club = opts.get_club(opts.club_id.unwrap_or_default(), "")?;

        assert_eq!(
            club,
            ClubInfo::EndpointParams {
                team: 529,
                next: 0,
                live: "all".into(),
                name: "".into()
            }
        );

        // club information
        let mut opts = FootballOpts::default();
        opts.club_id = None;
        opts.next_match = None;
        opts.club_name = Some("arsenal".into());
        let club = opts.get_club(
            opts.club_id.unwrap_or_default(),
            opts.club_name.clone().unwrap().as_str(),
        )?;

        assert_eq!(
            club,
            ClubInfo::EndpointParams {
                team: 0,
                next: 0,
                live: "all".into(),
                name: "arsenal".into()
            }
        );

        Ok(())
    }
}
