use serde::{Deserialize, Deserializer, Serialize};
use std::{collections::HashMap, fmt::Write};

use crate::StringType;

#[derive(Serialize, Default, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Periods {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub second: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct Venue {
    pub id: Option<u16>,
    pub name: StringType,
    pub city: StringType,
}

#[derive(Serialize, Default, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Status {
    pub long: StringType,
    pub short: StringType,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub elapsed: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct Fixture {
    pub id: u32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub referee: Option<StringType>,

    pub timezone: StringType,
    pub date: StringType,
    pub timestamp: u32,
    pub periods: Periods,
    pub venue: Venue,
    pub status: Status,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct League {
    pub id: u16,
    pub name: StringType,
    pub country: StringType,
    pub logo: StringType,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub flag: Option<String>,
    pub season: u16,
    pub round: StringType,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct Home {
    pub id: u16,
    pub name: StringType,
    pub logo: StringType,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub winner: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct Away {
    pub id: u16,
    pub name: StringType,
    pub logo: StringType,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub winner: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct Teams {
    pub home: Home,
    pub away: Away,
}

#[derive(Serialize, Default, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Goals {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub home: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub away: Option<usize>,
}

#[derive(Serialize, Default, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Halftime {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub home: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub away: Option<usize>,
}

#[derive(Serialize, Default, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Fulltime {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub home: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub away: Option<usize>,
}

#[derive(Serialize, Default, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Extratime {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub home: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub away: Option<usize>,
}

#[derive(Serialize, Default, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Penalty {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub home: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub away: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct Score {
    pub halftime: Halftime,
    pub fulltime: Fulltime,
    pub extratime: Extratime,
    pub penalty: Penalty,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Response {
    pub fixture: Fixture,
    pub league: League,
    pub teams: Teams,
    pub goals: Goals,
    pub score: Score,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum FootballErrors {
    Empty(Vec<Option<serde_json::Value>>),
    WithMessages(HashMap<String, String>),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FootballErrorMessage {
    pub access: Option<String>,
    pub token: Option<String>,
    pub requests: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FootballFixturesData {
    pub get: StringType,

    #[serde(flatten)]
    pub parameters: Parameters,

    pub errors: FootballErrors,
    pub results: usize,
    pub paging: Paging,
    pub response: Vec<Response>,
}

#[derive(Serialize, Debug, Default, Clone, Deserialize, PartialEq, Eq)]
pub struct Paging {
    pub current: u16,
    pub total: u16,
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub enum Parameters {
    Live(StringType),
    Next(StringType),
    Team(StringType),
}

impl<'de> Deserialize<'de> for Parameters {
    fn deserialize<D>(deserializer: D) -> Result<Parameters, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        let value: serde_json::Value = Deserialize::deserialize(deserializer)?;

        if let Some(parameters) = value.get("parameters").and_then(|p| p.as_object()) {
            if let Some((param_name, param_value)) = parameters.into_iter().next() {
                let param = match param_name.as_str() {
                    "live" => Parameters::Live(param_value.as_str().unwrap_or("").into()),
                    "next" => Parameters::Next(param_value.as_str().unwrap_or("").into()),
                    "team" => Parameters::Team(param_value.as_str().unwrap_or("").into()),
                    _ => return Err(Error::custom(format!("Encountered an issue with parameter naming `{param_name}` in the fixtures data")))
                };
                return Ok(param);
            }
        }

        Err(Error::custom(
            "Invalid JSON structure detected while parsing `Parameters` for fixtures data",
        ))
    }
}

impl Parameters {
    fn default() -> Self {
        // `Live` is hardcoded to `all` and `Team` will be available as CLI option
        Parameters::Next("1".into())
    }
}

impl Default for FootballFixturesData {
    fn default() -> Self {
        Self {
            get: "".into(),
            parameters: Parameters::default(),
            errors: FootballErrors::Empty(Vec::new()),
            results: 0,
            paging: Paging::default(),
            response: Vec::new(),
        }
    }
}

impl FootballFixturesData {
    fn get_goals(&self) -> (Vec<Option<usize>>, Vec<Option<usize>>) {
        let (home_goals, away_goals): (Vec<Option<usize>>, Vec<Option<usize>>) = self
            .response
            .iter()
            .map(|resp| (resp.goals.home, resp.goals.away))
            .unzip();

        (home_goals, away_goals)
    }

    /// Write out formatted information about the fixtures for a mutable buffer.
    /// ```
    /// use footballscore::football_fixtures_data::FootballFixturesData;
    /// # use anyhow::Error;
    /// # use std::io::{stdout, Write, Read};
    /// # use std::fs::File;
    /// # fn main() -> Result<(), Error> {
    /// # let mut buf = String::new();
    /// # let mut f = File::open("tests/resource/fixtures.json")?;
    /// # f.read_to_string(&mut buf)?;
    /// let data: FootballFixturesData = serde_json::from_str(&buf)?;
    ///
    /// let buf = data.get_current_fixtures();
    ///
    /// assert!(buf.starts_with("Match: Barcelona 0 vs 1 Arsenal"));
    ///
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn get_current_fixtures(&self) -> StringType {
        let mut output = StringType::from("");

        if let Some(response) = self.response.first() {
            output.push_str("Match: ");

            let (home_goals, away_goals) = self.get_goals();
            let home_team_name = &response.teams.home.name;

            if let Some(home_score) = home_goals.first().copied() {
                write!(
                    output,
                    "{} {:?}",
                    home_team_name,
                    home_score.unwrap_or_default()
                )
                .unwrap();
            } else {
                write!(output, "{home_team_name}").unwrap();
            }

            output.push_str(" vs ");

            if let Some(away_score) = away_goals.first().copied() {
                write!(
                    output,
                    "{:?} {}",
                    away_score.unwrap_or_default(),
                    &response.teams.away.name
                )
                .unwrap();
            } else {
                write!(output, "{}", &response.teams.away.name).unwrap();
            }

            write!(output, "\nNext match on {}\n", &response.fixture.date).unwrap();

            write!(
                output,
                "\tLeague: {} - {}/{}",
                &response.league.name, &response.league.season, &response.league.round
            )
            .unwrap();
            write!(
                output,
                "\n\tVenue: {}, {}",
                &response.fixture.venue.name, &response.fixture.venue.city
            )
            .unwrap();
            write!(output, "\n\tHome team: {}", &response.teams.home.name).unwrap();
            write!(output, "\n\tAway team: {}", &response.teams.away.name).unwrap();

            output.push('\n');
        } else if let FootballErrors::WithMessages(error_messages) = &self.errors {
            let mut buffer = String::with_capacity(500);

            let print_error = |output: &mut String, field_name: &str, error: &str| {
                writeln!(output, "Error: {field_name} - {error}").unwrap_or_default();
            };

            for field_name in &["access", "token", "requests"] {
                if let Some(error) = error_messages.get(*field_name) {
                    print_error(&mut buffer, field_name, error);
                }
            }

            if !buffer.is_empty() {
                output.push_str(&buffer);
            }
        } else {
            write!(output, "Match: no live event").unwrap();
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        football_fixtures_data::{FootballErrors, FootballFixturesData, Paging, Parameters},
        Error,
    };
    use log::info;

    #[test]
    fn test_football_data() -> Result<(), Error> {
        let buf = include_str!("../tests/resource/fixtures.json");
        let data: FootballFixturesData = serde_json::from_str(buf)?;

        let buf = data.get_current_fixtures();

        assert!(buf.starts_with("Match: Barcelona 0 vs 1 Arsenal"));

        if let Some(response) = data.response.first() {
            let (home_goals, away_goals) = data.get_goals();
            let home_team_name = &response.teams.home.name;
            let away_team_name = &response.teams.away.name;

            if let Some(home_score) = home_goals.first().copied() {
                if let Some(away_score) = away_goals.first().copied() {
                    info!(
                        "{}: {} {:?} vs {} {:?}",
                        buf.len(),
                        home_team_name,
                        home_score.unwrap_or_default(),
                        away_team_name,
                        away_score.unwrap_or_default()
                    );
                } else {
                    info!(
                        "{}: {} {:?} vs {}",
                        buf.len(),
                        home_team_name,
                        home_score.unwrap_or_default(),
                        away_team_name
                    );
                }
            } else if let Some(away_score) = away_goals.first().copied() {
                info!(
                    "{}: {} vs {} {:?}",
                    buf.len(),
                    home_team_name,
                    away_team_name,
                    away_score.unwrap_or_default()
                );
            } else {
                info!("{}: {} vs {}", buf.len(), home_team_name, away_team_name);
            }
        }

        Ok(())
    }

    #[test]
    fn test_default_football_data() -> Result<(), Error> {
        let default_data = FootballFixturesData::default();

        assert_eq!(
            default_data.get,
            "".to_string(),
            "Expected default get value"
        );

        assert_eq!(
            default_data.parameters,
            Parameters::default(),
            "Expected default parameters"
        );

        if let FootballErrors::Empty(empty_errors) = &default_data.errors {
            assert!(
                empty_errors.is_empty(),
                "Expected no errors in default data"
            );
        } else {
            panic!("Unexpected non-empty errors variant in default data");
        }

        assert_eq!(default_data.results, 0, "Expected default results value");

        assert_eq!(
            default_data.paging,
            Paging::default(),
            "Expected default paging"
        );

        assert!(
            default_data.response.is_empty(),
            "Expected no response data in default"
        );

        Ok(())
    }
}
