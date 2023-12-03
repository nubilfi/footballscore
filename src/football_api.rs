use std::{
    fmt::{self},
    hash::{Hash, Hasher},
};

use crate::Error;

#[cfg(feature = "cli")]
use reqwest::{Client, Url};

use crate::{
    apistringtype_from_display, football_data::FootballData, format_string, ApiStringType,
    StringType,
};

/// `FootballApi` contains a `reqwest` Client and all the metadata required to
/// query the api-football.com api.
#[cfg(feature = "cli")]
#[derive(Default, Clone)]
pub struct FootballApi {
    client: Client,
    api_key: ApiStringType,
    api_endpoint: StringType,
    api_path: StringType,
}

/// `live` and `next` is the only available parameter provided by the api.
/// The `Live` parameter cannot be used with `Next`
#[derive(Clone, Debug, PartialEq, Hash, Eq)]
pub enum ClubInfo {
    FixtureOpts {
        team: u32,
        next: u8,
        live: StringType,
    }
}

#[cfg(feature = "cli")]
impl Default for ClubInfo {
    fn default() -> Self {
        Self::FixtureOpts { team: 529, next: 1, live: "all".into() }
    }
}

impl fmt::Display for ClubInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FixtureOpts {
                team,
                next,
                live,
            } => {
                write!(f, "{team},{next},{live}")
            }
        }
    }
}

impl ClubInfo {
    #[inline]
    #[must_use]
    pub fn from_parameter(team: u32, next: u8, live: StringType) -> Self {
        Self::FixtureOpts {
            team,
            next,
            live,
        }
    }

    #[must_use]
    pub fn get_param_options(&self) -> Vec<(&'static str, ApiStringType)> {
        match self {
            Self::FixtureOpts {
                team,
                next,
                live,
            } => {
                let team = apistringtype_from_display(team);
                let next = apistringtype_from_display(next);
                let live = live;

                // the `live` parameter cannot be used with `next`
                if live.is_empty() {
                    return vec![("team", team), ("next", next)];
                } else {
                    return vec![("team", team), ("live", live.into())];
                }
            }
        }
    }
}

#[cfg(feature = "cli")]
impl PartialEq for FootballApi {
    fn eq(&self, other: &Self) -> bool {
        self.api_key == other.api_key
            && self.api_endpoint == other.api_endpoint
            && self.api_path == other.api_path
    }
}

#[cfg(feature = "cli")]
impl fmt::Debug for FootballApi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let api_key = &self.api_key;
        let api_endpoint = &self.api_endpoint;

        write!(f, "FootballApi(key={api_key},endpoint={api_endpoint})")
    }
}

#[cfg(feature = "cli")]
impl Hash for FootballApi {
    fn hash<H: Hasher>(&self, state: &mut H) {
        format!("{self:?}").hash(state);
    }
}

#[derive(Clone, Copy)]
enum FootballCommands {
    FootballScore,
}

impl FootballCommands {
    fn to_str(self) -> &'static str {
        match self {
            Self::FootballScore => "", // you can use this as an additional `api_path`
        }
    }
}

impl fmt::Display for FootballCommands {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.to_str())
    }
}

#[cfg(feature = "cli")]
impl FootballApi {
    /// Create `FootballApi` instance specifying `api_key`, `api_endpoint` and `api_path`
    #[must_use]
    pub fn new(api_key: &str, api_endpoint: &str, api_path: &str) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.into(),
            api_endpoint: api_endpoint.into(),
            api_path: api_path.into(),
        }
    }

    #[must_use]
    pub fn with_key(self, api_key: &str) -> Self {
        Self {
            api_key: api_key.into(),
            ..self
        }
    }

    #[must_use]
    pub fn with_endpoint(self, api_endpoint: &str) -> Self {
        Self {
            api_endpoint: api_endpoint.into(),
            ..self
        }
    }

    #[must_use]
    pub fn with_path(self, api_path: &str) -> Self {
        Self {
            api_path: api_path.into(),
            ..self
        }
    }

    fn get_api_options(&self, club: &ClubInfo) -> Vec<(&'static str, ApiStringType)> {
        let options = club.get_param_options();

        options
    }

    /// Get `FootballData` from api
    /// # Error
    ///
    /// Will return error if `FootballApi::run_api` fails
    pub async fn get_fixture_data(&self, club: &ClubInfo) -> Result<FootballData, Error> {
        let options = self.get_api_options(club);

        self.run_api(FootballCommands::FootballScore, &options).await
    }

    async fn run_api<T: serde::de::DeserializeOwned>(
        &self,
        command: FootballCommands,
        options: &[(&'static str, ApiStringType)],
    ) -> Result<T, Error> {
        let api_endpoint = &self.api_endpoint;
        let api_path = &self.api_path;

        let command = format_string!("{command}");

        self._run_api(&command, options, api_endpoint, api_path)
            .await
    }

    async fn _run_api<T: serde::de::DeserializeOwned>(
        &self,
        command: &str,
        options: &[(&'static str, ApiStringType)],
        api_endpoint: &str,
        api_path: &str,
    ) -> Result<T, Error> {
        let base_url = format!("https://{api_endpoint}/{api_path}?{command}");

        let url = Url::parse_with_params(&base_url, options)?;

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::HeaderName::from_static("x-rapidapi-key"),
            reqwest::header::HeaderValue::from_str(&self.api_key).unwrap(),
        );

        self.client
            .get(url)
            .headers(headers)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use log::info;
    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };

    use crate::{football_api::ClubInfo, ApiStringType, Error};

    #[cfg(feature = "cli")]
    use crate::football_api::FootballApi;

    #[cfg(feature = "cli")]
    #[tokio::test]
    async fn test_process_opts() -> Result<(), Error> {
        let api_key = "1e5765fc0c22df4e4ccf20581c2ef3d7";
        let api_endpoint = "v3.football.api-sports.io";
        let api_path = "fixtures";

        let api = FootballApi::new(api_key, api_endpoint, api_path);
        let club_info = ClubInfo::from_parameter(529, 0, "all".into());

        let mut hasher0 = DefaultHasher::new();
        club_info.hash(&mut hasher0);
        assert_eq!(hasher0.finish(), 5107599476288424662);

        let club = ClubInfo::from_parameter(529, 0, "all".into());

        let fixture = api.get_fixture_data(&club).await?;

        assert_eq!(
            &fixture.get_current_fixtures(),
            "Match: no live event" // no live match
        );

        Ok(())
    }

    #[cfg(feature = "cli")]
    #[test]
    fn test_footballapi() -> Result<(), Error> {
        let api = FootballApi::new(
            "1e5765fc0c22df4e4ccf20581c2ef3d7",
            "v3.football.api-sports.io",
            "fixtures",
        );

        let api2 = FootballApi::default()
            .with_key("1e5765fc0c22df4e4ccf20581c2ef3d7")
            .with_endpoint("v3.football.api-sports.io")
            .with_path("fixtures");
        assert_eq!(api, api2);

        assert_eq!(
            format!("{api:?}"),
            "FootballApi(key=1e5765fc0c22df4e4ccf20581c2ef3d7,endpoint=v3.football.api-sports.io)"
                .to_string()
        );

        let mut hasher0 = DefaultHasher::new();
        api.hash(&mut hasher0);
        let mut hasher1 = DefaultHasher::new();
        "FootballApi(key=1e5765fc0c22df4e4ccf20581c2ef3d7,endpoint=v3.football.api-sports.io)"
            .to_string()
            .hash(&mut hasher1);

        info!("{:?}", api);
        assert_eq!(hasher0.finish(), hasher1.finish());

        let club = ClubInfo::from_parameter(529, 0, "all".into());
        let opts = api.get_api_options(&club);
        let expected: Vec<(&str, ApiStringType)> = vec![("team", "529".into()), ("live", "all".into())];
        assert_eq!(opts, expected);

        Ok(())
    }

    #[test]
    fn test_clubinfo_default() -> Result<(), Error> {
        assert_eq!(ClubInfo::default(),  ClubInfo::from_parameter(529, 1, "all".into()));

        Ok(())
    }
}
