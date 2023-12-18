use serde::{Deserialize, Deserializer, Serialize};
use std::{collections::HashMap, fmt::Write};

use crate::StringType;

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct Venue {
    pub id: Option<u16>,
    pub name: Option<StringType>,
    pub address: Option<StringType>,
    pub city: Option<StringType>,
    pub capacity: Option<u32>,
    pub surface: Option<StringType>,
    pub image: Option<StringType>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct Team {
    pub id: Option<u16>,
    pub name: Option<StringType>,
    pub code: Option<StringType>,
    pub country: Option<StringType>,
    pub founded: Option<u16>,
    pub national: Option<bool>,
    pub logo: Option<StringType>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Response {
    pub team: Team,
    pub venue: Venue,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum FootballTeamsErrors {
    Empty(Vec<Option<serde_json::Value>>),
    WithMessages(HashMap<String, String>),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FootballErrorMessage {
    pub access: Option<String>,
    pub token: Option<String>,
    pub requests: Option<String>,
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FootballTeamsData {
    pub get: StringType,

    #[serde(flatten)]
    pub parameters: Parameters,

    pub errors: FootballTeamsErrors,
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
    Name(StringType),
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
                    "name" => Parameters::Name(param_value.as_str().unwrap_or("").into()),
                    _ => return Err(Error::custom(format!("Encountered an issue with parameter naming `{param_name}` in the teams data")))
                };
                return Ok(param);
            }
        }

        Err(Error::custom(
            "Invalid JSON structure detected while parsing `Parameters` for teams data",
        ))
    }
}

impl Parameters {
    fn default() -> Self {
        Parameters::Name("".into())
    }
}

impl Default for FootballTeamsData {
    fn default() -> Self {
        Self {
            get: "".into(),
            parameters: Parameters::default(),
            errors: FootballTeamsErrors::Empty(Vec::new()),
            results: 0,
            paging: Paging::default(),
            response: Vec::new(),
        }
    }
}

impl FootballTeamsData {
    /// Write out formatted information about the teams for a mutable buffer.
    /// ```
    /// use footballscore::football_teams_data::FootballTeamsData;
    /// # use anyhow::Error;
    /// # use std::io::{stdout, Write, Read};
    /// # use std::fs::File;
    /// # fn main() -> Result<(), Error> {
    /// # let mut buf = String::new();
    /// # let mut f = File::open("tests/resource/teams.json")?;
    /// # f.read_to_string(&mut buf)?;
    /// let data: FootballTeamsData = serde_json::from_str(&buf)?;
    ///
    /// let buf = data.get_teams_information();
    ///
    /// assert!(buf.starts_with("Here's your club information:"));
    /// assert!(buf.contains("Name: Barcelona"));
    ///
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn get_teams_information(&self) -> StringType {
        let mut output = StringType::from("");

        if let Some(response) = self.response.first() {
            let team_info = &response.team;
            let venue_info = &response.venue;

            output.push_str("Here's your club information:\n");

            if let Some(name) = &team_info.name {
                writeln!(output, "Name: {name}").unwrap();
            }

            writeln!(output, "Club ID: {}", team_info.id.unwrap_or_default()).unwrap();

            if let Some(venue_name) = &venue_info.name {
                writeln!(output, "Venue: {venue_name}").unwrap();
            }

            output.push('\n');
        } else if let FootballTeamsErrors::WithMessages(error_messages) = &self.errors {
            let mut buffer = String::with_capacity(500);

            let print_error = |output: &mut String, field_name: &str, error: &str| {
                writeln!(output, "Error: {field_name} - {error}").unwrap();
            };

            for field_name in &["access", "token", "requests", "name"] {
                if let Some(error) = error_messages.get(*field_name) {
                    print_error(&mut buffer, field_name, error);
                }
            }

            if !buffer.is_empty() {
                output.push_str(&buffer);
            }
        } else {
            output.push_str("Your club data is unavailable");
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        football_teams_data::{FootballTeamsData, FootballTeamsErrors, Paging, Parameters},
        Error,
    };
    use log::info;

    #[test]
    fn test_football_teams_data() -> Result<(), Error> {
        let buf = include_str!("../tests/resource/teams.json");
        let data: FootballTeamsData = serde_json::from_str(buf)?;

        let buf = data.get_teams_information();

        assert!(buf.starts_with("Here's your club information:"));
        assert!(buf.contains("\nName: Barcelona"));

        if let Some(response) = data.response.first() {
            let team_info = &response.team;
            let venue_info = &response.venue;

            info!(
                "{}: Name: {}\nClubID: {}\n",
                buf.len(),
                team_info.name.clone().unwrap_or_default(),
                team_info.id.clone().unwrap_or_default()
            );
            info!(
                "{}: Venue: {}\n",
                buf.len(),
                venue_info.name.clone().unwrap_or_default()
            );
        }

        Ok(())
    }

    #[test]
    fn test_default_football_data() -> Result<(), Error> {
        let default_data = FootballTeamsData::default();

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

        if let FootballTeamsErrors::Empty(empty_errors) = &default_data.errors {
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
