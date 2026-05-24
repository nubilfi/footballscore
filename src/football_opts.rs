use crate::{format_string, Error};

#[cfg(feature = "cli")]
use chrono::Local;

#[cfg(feature = "cli")]
use clap::{CommandFactory, Parser, Subcommand};

#[cfg(feature = "cli")]
use std::fmt::Write;

#[cfg(feature = "cli")]
use tokio::io::{stdout, AsyncWriteExt};

use crate::{config::Config, ApiStringType, StringType};

#[cfg(feature = "cli")]
use crate::football_api::{ClubInfo, FootballApi};

#[cfg(feature = "cli")]
use crate::soccer_api::SoccerApi;

/// Retrieve football scores and fixtures
///
/// Uses api-football.com (paid) for live scores and next fixtures
/// and api.soccerdataapi.com (free) for upcoming fixtures
#[cfg(feature = "cli")]
#[derive(Parser)]
#[command(name = "footballscore", version, about, long_about = None)]
pub struct FootballOpts {
    #[command(subcommand)]
    command: FootballCommand,
}

#[cfg(feature = "cli")]
#[derive(Subcommand)]
enum FootballCommand {
    /// Retrieve live score for a club
    Live(LiveOpts),

    /// Show next fixture for a club
    Next(NextOpts),

    /// Look up a club ID by name
    Team(TeamOpts),

    /// Show upcoming fixture with live score
    Upcoming(UpcomingOpts),

    /// Search for a team ID by name
    Find(FindOpts),
}

/// Retrieve live score for a club (api-football.com)
#[cfg(feature = "cli")]
#[derive(Parser)]
struct LiveOpts {
    /// Api key (or set API_KEY in config)
    #[clap(short = 'k', long)]
    api_key: Option<ApiStringType>,

    /// Club ID (default: 529 - Barcelona)
    #[clap(short = 'c', long)]
    club_id: Option<u16>,
}

/// Show next fixture for a club (api-football.com)
#[cfg(feature = "cli")]
#[derive(Parser)]
struct NextOpts {
    /// Api key (or set API_KEY in config)
    #[clap(short = 'k', long)]
    api_key: Option<ApiStringType>,

    /// Club ID (default: 529 - Barcelona)
    #[clap(short = 'c', long)]
    club_id: Option<u16>,
}

/// Look up a club ID by name (api-football.com)
#[cfg(feature = "cli")]
#[derive(Parser)]
struct TeamOpts {
    /// Api key (or set API_KEY in config)
    #[clap(short = 'k', long)]
    api_key: Option<ApiStringType>,

    /// Club name to search for
    #[clap(short = 'n', long)]
    name: StringType,
}

/// Show upcoming fixture with live score (api.soccerdataapi.com)
#[cfg(feature = "cli")]
#[derive(Parser)]
struct UpcomingOpts {
    /// Auth token (or set AUTH_TOKEN in config)
    #[clap(short = 't', long)]
    token: Option<ApiStringType>,

    /// Team ID from api.soccerdataapi.com.
    /// Use `footballscore find <name>` to look up the correct ID.
    #[clap(long)]
    team_id: u32,
}

/// Search for a team ID by name (api.soccerdataapi.com)
#[cfg(feature = "cli")]
#[derive(Parser)]
struct FindOpts {
    /// Auth token (or set AUTH_TOKEN in config)
    #[clap(short = 't', long)]
    token: Option<ApiStringType>,

    /// Team name to search (case-insensitive substring match)
    name: StringType,
}

#[cfg(feature = "cli")]
impl FootballOpts {
    /// Parse options from stdin, requires `Config` instance.
    ///
    /// # Errors
    /// Returns error if the API call fails or writing to stdout fails.
    pub async fn parse_opts(config: &Config) -> Result<(), Error> {
        let opts = Self::parse();
        let mut stdout = stdout();

        for output in opts.run(config).await? {
            stdout.write_all(output.as_bytes()).await?;
        }

        Ok(())
    }

    async fn run(&self, config: &Config) -> Result<Vec<StringType>, Error> {
        match &self.command {
            FootballCommand::Live(opts) => run_live(opts, config).await,
            FootballCommand::Next(opts) => run_next(opts, config).await,
            FootballCommand::Team(opts) => run_team(opts, config).await,
            FootballCommand::Upcoming(opts) => run_upcoming(opts, config).await,
            FootballCommand::Find(opts) => run_find(opts, config).await,
        }
    }

    #[must_use]
    pub fn api_help_msg() -> StringType {
        format_string!("{}", Self::command().render_help())
    }
}

/// Build a `FootballApi` client, preferring the CLI key over the config value
#[cfg(feature = "cli")]
fn build_football_api(cli_key: Option<&str>, config: &Config) -> Result<FootballApi, Error> {
    let api_key = cli_key
        .or_else(|| config.api_key.as_deref())
        .ok_or_else(|| {
            Error::InvalidInputError(format_string!(
                "api key required: use -k <key> or set API_KEY in config"
            ))
        })?;

    Ok(FootballApi::new(api_key, &config.api_endpoint))
}

/// Build a `SoccerApi` client, preferring the CLI token over the config value
#[cfg(feature = "cli")]
fn build_soccer_api(cli_token: Option<&str>, config: &Config) -> Result<SoccerApi, Error> {
    let auth_token = cli_token
        .or_else(|| config.auth_token.as_deref())
        .ok_or_else(|| {
            Error::InvalidInputError(format_string!(
                "auth token required: use -t <token> or set AUTH_TOKEN in config"
            ))
        })?;

    Ok(SoccerApi::new(auth_token, &config.soccer_endpoint))
}

#[cfg(feature = "cli")]
async fn run_live(opts: &LiveOpts, config: &Config) -> Result<Vec<StringType>, Error> {
    let api = build_football_api(opts.api_key.as_deref(), config)?;
    let club_id = opts.club_id.unwrap_or(config.club_id);
    let club = ClubInfo::from_parameter(club_id, 0, "all".into(), "".into());
    let data = api.get_fixture_data(&club).await?;
    Ok(vec![data.get_current_fixtures()])
}

#[cfg(feature = "cli")]
async fn run_next(opts: &NextOpts, config: &Config) -> Result<Vec<StringType>, Error> {
    let api = build_football_api(opts.api_key.as_deref(), config)?;
    let club_id = opts.club_id.unwrap_or(config.club_id);

    // next=1 tells the paid API to return the next scheduled fixture
    let club = ClubInfo::from_parameter(club_id, 1, "".into(), "".into());
    let data = api.get_fixture_data(&club).await?;
    Ok(vec![data.get_current_fixtures()])
}

#[cfg(feature = "cli")]
async fn run_team(opts: &TeamOpts, config: &Config) -> Result<Vec<StringType>, Error> {
    let api = build_football_api(opts.api_key.as_deref(), config)?;
    let club = ClubInfo::from_parameter(0, 0, "".into(), opts.name.clone());
    let data = api.get_team_data(&club).await?;
    Ok(vec![data.get_teams_information()])
}

#[cfg(feature = "cli")]
async fn run_upcoming(opts: &UpcomingOpts, config: &Config) -> Result<Vec<StringType>, Error> {
    let soccer_api = build_soccer_api(opts.token.as_deref(), config)?;
    let today = Local::now().date_naive();

    // find the next fixture for this team in the upcoming list
    let upcoming_list = soccer_api.get_upcoming_fixtures().await?;
    let preview = upcoming_list.find_next_for_team(opts.team_id, today);

    let match_id = match preview {
        Some(p) => p.id,
        None => return Ok(vec![format_string!("Match: no match event\n")]),
    };

    // fetch full match detail for the live score
    let data = soccer_api.get_match_detail(match_id).await?;
    Ok(vec![data.get_current_fixtures()])
}

#[cfg(feature = "cli")]
async fn run_find(opts: &FindOpts, config: &Config) -> Result<Vec<StringType>, Error> {
    let soccer_api = build_soccer_api(opts.token.as_deref(), config)?;
    let upcoming = soccer_api.get_upcoming_fixtures().await?;
    let matches = upcoming.search_teams(&opts.name);

    let mut output = StringType::new();
    if matches.is_empty() {
        let _ = write!(output, "No teams found matching \"{}\"\n", opts.name);
    } else {
        let _ = write!(output, "Teams matching \"{}\":\n", opts.name);
        for (id, name) in &matches {
            let _ = write!(output, "  {id}\t{name}\n");
        }
    }

    Ok(vec![output])
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use std::env::set_var;

    use crate::{
        config::{Config, TestEnvs},
        football_api::ClubInfo,
        Error,
    };

    #[cfg(feature = "cli")]
    use crate::football_opts::FootballOpts;

    #[cfg(feature = "cli")]
    #[test]
    fn test_api_help_msg() -> Result<(), Error> {
        let msg = FootballOpts::api_help_msg();
        assert!(!msg.is_empty());
        // verify the top-level subcommands appear in the help text
        assert!(msg.contains("live"));
        assert!(msg.contains("next"));
        assert!(msg.contains("team"));
        assert!(msg.contains("upcoming"));
        assert!(msg.contains("find"));
        Ok(())
    }

    #[cfg(feature = "cli")]
    #[test]
    fn test_build_football_api_uses_config_key() -> Result<(), Error> {
        let _env = TestEnvs::new(&["API_KEY", "API_ENDPOINT", "CLUB_ID"]);

        set_var("API_KEY", "testkey123");
        set_var("API_ENDPOINT", "test.local");
        set_var("CLUB_ID", "529");

        let config = Config::init_config(None)?;
        drop(_env);

        // the api key and endpoint from config must be reflected in Debug output
        assert_eq!(config.api_key.as_deref(), Some("testkey123"));
        assert_eq!(config.api_endpoint.as_str(), "test.local");
        Ok(())
    }

    #[cfg(feature = "cli")]
    #[test]
    fn test_build_football_api_missing_key_returns_error() -> Result<(), Error> {
        let _env = TestEnvs::new(&["API_KEY", "API_ENDPOINT"]);

        set_var("API_ENDPOINT", "test.local");

        let _config = Config::init_config(None)?;
        drop(_env);

        let club = ClubInfo::from_parameter(529, 0, "all".into(), "".into());
        assert_eq!(
            club,
            ClubInfo::EndpointParams {
                team: 529,
                next: 0,
                live: "all".into(),
                name: "".into(),
            }
        );
        Ok(())
    }

    #[test]
    fn test_clubinfo_live() -> Result<(), Error> {
        // run_live always passes next=0, live="all"
        let club = ClubInfo::from_parameter(529, 0, "all".into(), "".into());
        assert_eq!(
            club,
            ClubInfo::EndpointParams {
                team: 529,
                next: 0,
                live: "all".into(),
                name: "".into(),
            }
        );
        Ok(())
    }

    #[test]
    fn test_clubinfo_next() -> Result<(), Error> {
        // run_next always passes next=1, live="" (empty disables live param)
        let club = ClubInfo::from_parameter(529, 1, "".into(), "".into());
        assert_eq!(
            club,
            ClubInfo::EndpointParams {
                team: 529,
                next: 1,
                live: "".into(),
                name: "".into(),
            }
        );
        Ok(())
    }

    #[test]
    fn test_clubinfo_team_lookup() -> Result<(), Error> {
        // run_team passes team=0, next=0, live="", name=<query>
        let club = ClubInfo::from_parameter(0, 0, "".into(), "arsenal".into());
        assert_eq!(
            club,
            ClubInfo::EndpointParams {
                team: 0,
                next: 0,
                live: "".into(),
                name: "arsenal".into(),
            }
        );
        Ok(())
    }

    #[test]
    fn test_config_defaults() -> Result<(), Error> {
        let _env = TestEnvs::new(&["API_KEY", "API_ENDPOINT", "CLUB_ID", "AUTH_TOKEN"]);

        set_var("API_KEY", "defaultkey");
        set_var("API_ENDPOINT", "v3.football.api-sports.io");
        set_var("CLUB_ID", "529");

        let config = Config::init_config(None)?;
        drop(_env);

        dbg!(&config);

        assert_eq!(config.club_id, 529);
        assert_eq!(config.api_key.as_deref(), Some("defaultkey"));
        assert!(config.auth_token.is_none());
        Ok(())
    }

    #[cfg(feature = "cli")]
    #[tokio::test]
    async fn test_run_live_bad_key() -> Result<(), Error> {
        use crate::football_api::FootballApi;

        let api = FootballApi::new("invalid_key", "v3.football.api-sports.io");
        let club = ClubInfo::from_parameter(529, 0, "all".into(), "".into());

        match api.get_fixture_data(&club).await {
            Ok(data) => {
                let output = data.get_current_fixtures();
                assert!(
                    output.contains("Error: token") || output.contains("Match:"),
                    "unexpected output: {output}"
                );
            }
            Err(Error::ReqwestError(e)) => {
                let status = e.status().map(|s| s.as_u16());
                assert!(
                    matches!(status, Some(401) | Some(403)),
                    "unexpected reqwest error: {e}"
                );
            }
            Err(e) => return Err(e),
        }
        Ok(())
    }

    #[cfg(feature = "cli")]
    #[tokio::test]
    async fn test_run_team_bad_key() -> Result<(), Error> {
        use crate::football_api::FootballApi;

        let api = FootballApi::new("invalid_key", "v3.football.api-sports.io");
        let club = ClubInfo::from_parameter(0, 0, "".into(), "arsenal".into());

        match api.get_team_data(&club).await {
            Ok(data) => {
                let output = data.get_teams_information();
                assert!(
                    output.contains("Error: token")
                        || output.contains("Here's your club information:"),
                    "unexpected output: {output}"
                );
            }
            Err(Error::ReqwestError(e)) => {
                let status = e.status().map(|s| s.as_u16());
                assert!(
                    matches!(status, Some(401) | Some(403)),
                    "unexpected reqwest error: {e}"
                );
            }
            Err(e) => return Err(e),
        }
        Ok(())
    }
}
