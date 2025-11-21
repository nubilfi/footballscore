use std::{
    fmt,
    hash::{Hash, Hasher},
};

use crate::Error;

#[cfg(feature = "cli")]
use reqwest::{Client, Url};

use crate::{
    apistringtype_from_display, football_fixtures_data::FootballFixturesData,
    football_teams_data::FootballTeamsData, format_string, ApiStringType, StringType,
};

/// `FootballApi` contains a `reqwest` Client and all the metadata required to
/// query the api-football.com api.
#[cfg(feature = "cli")]
#[derive(Default, Clone)]
pub struct FootballApi {
    client: Client,
    api_key: ApiStringType,
    api_endpoint: StringType,
}

/// `live` and `next` is the only available parameter provided by the api.
/// The `Live` parameter cannot be used with `Next`.
/// `Name` will be used only for Teams endpoint.
#[derive(Clone, Debug, PartialEq, Hash, Eq)]
pub enum ClubInfo {
    EndpointParams {
        team: u16,
        next: u8,
        live: StringType,
        name: StringType,
    },
}

#[cfg(feature = "cli")]
impl Default for ClubInfo {
    fn default() -> Self {
        Self::EndpointParams {
            team: 529,
            next: 1,
            live: "all".into(),
            name: "".into(),
        }
    }
}

impl fmt::Display for ClubInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EndpointParams {
                team,
                next,
                live,
                name,
            } => {
                write!(f, "{team},{next},{live},{name}")
            }
        }
    }
}

impl ClubInfo {
    #[inline]
    #[must_use]
    pub fn from_parameter(team: u16, next: u8, live: StringType, name: StringType) -> Self {
        Self::EndpointParams {
            team,
            next,
            live,
            name,
        }
    }

    #[must_use]
    pub fn get_param_options(&self) -> Vec<(&'static str, ApiStringType)> {
        match self {
            Self::EndpointParams {
                team,
                next,
                live,
                name,
            } => {
                match name.as_str() {
                    "" => {
                        let team_str = apistringtype_from_display(team);
                        let next_str = apistringtype_from_display(next);

                        // the `live` parameter cannot be used with `next`
                        if live.is_empty() {
                            return vec![("team", team_str), ("next", next_str)];
                        }

                        vec![("team", team_str), ("live", live.into())]
                    }
                    _ => vec![("name", apistringtype_from_display(name))],
                }
            }
        }
    }
}

#[cfg(feature = "cli")]
impl PartialEq for FootballApi {
    fn eq(&self, other: &Self) -> bool {
        self.api_key == other.api_key && self.api_endpoint == other.api_endpoint
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
    FootballFixture,
    FootballTeam,
}

impl FootballCommands {
    fn to_str(self) -> &'static str {
        match self {
            Self::FootballFixture => "fixtures", // you can use this as an additional `api path url`
            Self::FootballTeam => "teams",       // you can use this as an additional `api path url`
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
    /// Create `FootballApi` instance specifying `api_key`, `api_endpoint`
    #[must_use]
    pub fn new(api_key: &str, api_endpoint: &str) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.into(),
            api_endpoint: api_endpoint.into(),
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

    #[allow(clippy::unused_self)]
    fn get_api_options(&self, club: &ClubInfo) -> Vec<(&'static str, ApiStringType)> {
        club.get_param_options()
    }

    /// Get `FootballFixturesData` from api
    /// # Errors
    ///
    /// Will return error if `FootballApi::run_api` fails
    pub async fn get_fixture_data(&self, club: &ClubInfo) -> Result<FootballFixturesData, Error> {
        let options = self.get_api_options(club);
        self.run_api(FootballCommands::FootballFixture, &options)
            .await
    }

    /// Get `FootballTeamsData` from api
    /// # Errors
    ///
    /// Will return error if `FootballApi::run_api` fails
    pub async fn get_team_data(&self, club: &ClubInfo) -> Result<FootballTeamsData, Error> {
        let options = self.get_api_options(club);
        self.run_api(FootballCommands::FootballTeam, &options).await
    }

    async fn run_api<T: serde::de::DeserializeOwned>(
        &self,
        command: FootballCommands,
        options: &[(&'static str, ApiStringType)],
    ) -> Result<T, Error> {
        let api_endpoint = &self.api_endpoint;
        let command = format_string!("{command}");
        self.run_api_client(&command, options, api_endpoint).await
    }

    async fn run_api_client<T: serde::de::DeserializeOwned>(
        &self,
        command: &str,
        options: &[(&'static str, ApiStringType)],
        api_endpoint: &str,
    ) -> Result<T, Error> {
        let base_url = format!("https://{api_endpoint}/{command}?");
        let url = Url::parse_with_params(&base_url, options)?;
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::HeaderName::from_static("x-rapidapi-key"),
            reqwest::header::HeaderValue::from_str(self.api_key.as_str())?,
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
#[allow(clippy::disallowed_methods)]
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

        let api = FootballApi::new(api_key, api_endpoint);

        // Fixtures
        let club_info = ClubInfo::from_parameter(529, 0, "all".into(), "".into());

        let mut hasher0 = DefaultHasher::new();
        club_info.hash(&mut hasher0);
        assert_eq!(hasher0.finish(), 17875426778410589958);

        let club = ClubInfo::from_parameter(529, 0, "all".into(), "".into());

        let fixture = api.get_fixture_data(&club).await?;

        assert_eq!(
            &fixture.get_current_fixtures(),
            "Error: token - Error/Missing application key. Go to https://www.api-football.com/documentation-v3 to learn how to get your API application key.\n"
        );

        // Teams
        let club_info = ClubInfo::from_parameter(0, 0, "".into(), "arsenal".into());

        let mut hasher0 = DefaultHasher::new();
        club_info.hash(&mut hasher0);
        assert_eq!(hasher0.finish(), 8926715139541391656);

        let club = ClubInfo::from_parameter(0, 0, "".into(), "arsenal".into());

        let team = api.get_team_data(&club).await?;

        assert_eq!(
            &team.get_teams_information(),
            "Error: token - Error/Missing application key. Go to https://www.api-football.com/documentation-v3 to learn how to get your API application key.\n"
        );

        Ok(())
    }

    #[cfg(feature = "cli")]
    #[test]
    fn test_footballapi() -> Result<(), Error> {
        let api = FootballApi::new(
            "1e5765fc0c22df4e4ccf20581c2ef3d7",
            "v3.football.api-sports.io",
        );

        let api2 = FootballApi::default()
            .with_key("1e5765fc0c22df4e4ccf20581c2ef3d7")
            .with_endpoint("v3.football.api-sports.io");
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

        // Fixtures
        let club = ClubInfo::from_parameter(529, 0, "all".into(), "".into());
        let opts = api.get_api_options(&club);
        let expected: Vec<(&str, ApiStringType)> =
            vec![("team", "529".into()), ("live", "all".into())];
        assert_eq!(opts, expected);

        // Teams
        let club = ClubInfo::from_parameter(0, 0, "".into(), "arsenal".into());
        let opts = api.get_api_options(&club);
        let expected: Vec<(&str, ApiStringType)> = vec![("name", "arsenal".into())];
        assert_eq!(opts, expected);

        Ok(())
    }

    #[test]
    fn test_clubinfo_default() -> Result<(), Error> {
        assert_eq!(
            ClubInfo::default(),
            ClubInfo::from_parameter(529, 1, "all".into(), "".into())
        );

        Ok(())
    }
}
