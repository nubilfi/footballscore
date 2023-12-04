use serde::{Deserialize, Serialize, Deserializer};
use std::fmt::{self, Write};

use crate::StringType;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Periods {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub second: Option<u32>,
}

impl Default for Periods {
    fn default() -> Self {
        Self {
            first: None,
            second: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct Venue {
    pub id: u16,
    pub name: StringType,
    pub city: StringType,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Status {
    pub long: StringType,
    pub short: StringType,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub elapsed: Option<u8>,
}

impl Default for Status {
    fn default() -> Self {
        Self {
            long: "Not Started".into(),
            short: "NS".into(),
            elapsed: None,
        }
    }
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Goals {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub home: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub away: Option<usize>,
}

impl Default for Goals {
    fn default() -> Self {
        Self {
            home: None,
            away: None,
        }
    }
}

impl fmt::Display for Goals {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.home, self.away) {
            (Some(home), Some(away)) => write!(f, "{} {}", home, away),
            (Some(home), None) => write!(f, "{} ", home),
            (None, Some(away)) => write!(f, " {}", away),
            (None, None) => write!(f, ""),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Halftime {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub home: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub away: Option<usize>,
}

impl Default for Halftime {
    fn default() -> Self {
        Self {
            home: None,
            away: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Fulltime {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub home: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub away: Option<usize>,
}

impl Default for Fulltime {
    fn default() -> Self {
        Self {
            home: None,
            away: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Extratime {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub home: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub away: Option<usize>,
}

impl Default for Extratime {
    fn default() -> Self {
        Self {
            home: None,
            away: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Penalty {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub home: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub away: Option<usize>,
}

impl Default for Penalty {
    fn default() -> Self {
        Self {
            home: None,
            away: None,
        }
    }
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
pub struct FootballData {
    pub get: StringType,

    #[serde(flatten)]
    pub parameters: Parameters,

    pub errors: Vec<Option<serde_json::Value>>,
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
        let value: serde_json::Value = Deserialize::deserialize(deserializer)?;

        if let Some(parameters) = value.get("parameters") {
            if let Some(live) = parameters.get("live").and_then(serde_json::Value::as_str) {
                return Ok(Parameters::Live(live.into()));
            } else if let Some(next) = parameters.get("next").and_then(serde_json::Value::as_str) {
                return Ok(Parameters::Next(next.into()));
            } else if let Some(team) = parameters.get("team").and_then(serde_json::Value::as_str) {
                return Ok(Parameters::Team(team.into()));
            }
        }

        Err(serde::de::Error::custom("Invalid JSON structure for `Parameters`"))
    }
}

impl Parameters {
    fn default() -> Self {
        // `Live` is hardcoded to `all` and `Team` will be available as CLI option
        Parameters::Next("1".into())
    }
}

impl Default for FootballData {
    fn default() -> Self {
        Self {
            get: "".into(),
            parameters: Parameters::default(),
            errors: Vec::new(),
            results: 0,
            paging: Paging::default(),
            response: Vec::new(),
        }
    }
}

impl FootballData {
    fn get_goals(&self) -> (Vec<Option<usize>>, Vec<Option<usize>>) {
        let (home_goals, away_goals): (Vec<Option<usize>>, Vec<Option<usize>>) = self.response
            .iter()
            .map(|resp| (resp.goals.home, resp.goals.away))
            .unzip();

        (home_goals, away_goals)
    }

    /// Write out formatted information about the fixtures for a mutable buffer.
    /// ```
    /// use footballscore::football_data::FootballData;
    /// # use anyhow::Error;
    /// # use std::io::{stdout, Write, Read};
    /// # use std::fs::File;
    /// # fn main() -> Result<(), Error> {
    /// # let mut buf = String::new();
    /// # let mut f = File::open("tests/resource/fixtures.json")?;
    /// # f.read_to_string(&mut buf)?;
    /// let data: FootballData = serde_json::from_str(&buf)?;
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
        let mut output = StringType::from("Match: ");

        if let Some(response) = self.response.first() {
            let (home_goals, away_goals) = self.get_goals();
            let home_team_name = &response.teams.home.name;

            if let Some(home_score) = home_goals.get(0).copied() {
                write!(output, "{} {:?}", home_team_name, home_score.unwrap_or_default()).unwrap();
            } else {
                write!(output, "{}", home_team_name).unwrap();
            }

            output.push_str(" vs ");

            if let Some(away_score) = away_goals.get(0).copied() {
                write!(output, "{:?} {}", away_score.unwrap_or_default(), &response.teams.away.name).unwrap();
            } else {
                write!(output, "{}", &response.teams.away.name).unwrap();
            }
        } else {
            write!(output, "{}", "no live event").unwrap();
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        football_data::{FootballData, Parameters, Paging},
        Error
    };
    use log::info;

    #[test]
    fn test_football_data() -> Result<(), Error> {
        let buf = include_str!("../tests/resource/fixtures.json");
        let data: FootballData = serde_json::from_str(buf)?;

        let buf = data.get_current_fixtures();

        assert!(buf.starts_with("Match: Barcelona 0 vs 1 Arsenal"));

        if let Some(response) = data.response.first() {
            let (home_goals, away_goals) = data.get_goals();
            let home_team_name = &response.teams.home.name;
            let away_team_name = &response.teams.away.name;

            if let Some(home_score) = home_goals.get(0).copied() {
                if let Some(away_score) = away_goals.get(0).copied() {
                    info!("{}: {} {:?} vs {} {:?}", buf.len(), home_team_name, home_score.unwrap_or_default(), away_team_name, away_score.unwrap_or_default());
                } else {
                    info!("{}: {} {:?} vs {}", buf.len(), home_team_name, home_score.unwrap_or_default(), away_team_name);
                }
            } else if let Some(away_score) = away_goals.get(0).copied() {
                info!("{}: {} vs {} {:?}", buf.len(), home_team_name, away_team_name, away_score.unwrap_or_default());
            } else {
                info!("{}: {} vs {}", buf.len(), home_team_name, away_team_name);
            }
        }

        Ok(())
    }

    #[test]
    fn test_default_football_data() -> Result<(), Error> {
        let default_data = FootballData::default();

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

        assert!(
            default_data.errors.is_empty(),
            "Expected no errors in default data"
        );

        assert_eq!(
            default_data.results,
            0,
            "Expected default results value"
        );

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
