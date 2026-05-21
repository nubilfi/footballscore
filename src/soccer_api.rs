use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::fmt::Write;

use crate::StringType;

#[cfg(feature = "cli")]
use reqwest::{Client, Url};

#[cfg(feature = "cli")]
use crate::{apistringtype_from_display, ApiStringType, Error};


/// Shared team / goals types (used by both endpoints)
/// `api-football.com` and `api.soccerdataapi.com`
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct SoccerTeam {
    pub id: Option<u32>,
    pub name: Option<StringType>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct SoccerTeams {
    pub home: SoccerTeam,
    pub away: SoccerTeam,
}

/// Goals from api.soccerdataapi.com `/match/` endpoint
/// the API uses `-1` for periods that did not take place (extra time, penalties)
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct SoccerGoals {
    pub home_ft_goals: i32,
    pub away_ft_goals: i32,
    pub home_ht_goals: i32,
    pub away_ht_goals: i32,
    pub home_et_goals: i32,
    pub away_et_goals: i32,
    pub home_pen_goals: i32,
    pub away_pen_goals: i32,
}

impl SoccerGoals {
    /// Full-time goals for the home side, `-1` (not played) `0`
    #[inline]
    #[must_use]
    pub fn home(&self) -> usize {
        self.home_ft_goals.max(0) as usize
    }

    /// Full-time goals for the away side, `-1` (not played) `0`
    #[inline]
    #[must_use]
    pub fn away(&self) -> usize {
        self.away_ft_goals.max(0) as usize
    }
}

/// Full match detail returned by `GET /match/?match_id=<id>&auth_token=<token>`
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct SoccerMatchData {
    pub id: Option<u32>,
    pub date: Option<StringType>,
    pub time: Option<StringType>,
    pub status: Option<StringType>,
    pub teams: Option<SoccerTeams>,
    pub goals: Option<SoccerGoals>,
}

/// Error payload the API returns when a match can't
/// be fetched: `{"detail": "Error fetching match"}`
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SoccerApiError {
    pub detail: StringType,
}

impl SoccerMatchData {
    /// Format match data in the same style as
    /// `FootballFixturesData::get_current_fixtures`
    #[must_use]
    pub fn get_current_fixtures(&self) -> StringType {
        let mut output = StringType::new();

        if let Some(teams) = &self.teams {
            let home_name = teams.home.name.as_deref().map_or("Unknown", |v| v);
            let away_name = teams.away.name.as_deref().map_or("Unknown", |v| v);

            // Default to 0 - 0 when goals are absent (pre-match) or all -1
            let (home_score, away_score) = self
                .goals
                .as_ref()
                .map(|g| (g.home(), g.away()))
                .unwrap_or((0, 0));

            let _ = write!(
                output,
                "Match: {home_name} {home_score} vs {away_score} {away_name}"
            );

            if let (Some(date), Some(time)) = (&self.date, &self.time) {
                let _ = write!(output, "\nNext match on {date} at {time}");
            }

            if let Some(status) = &self.status {
                let _ = write!(output, "\n\tStatus: {status}");
            }

            let _ = write!(output, "\n\tHome team: {home_name}");
            let _ = write!(output, "\n\tAway team: {away_name}");
            output.push('\n');
        } else {
            let _ = write!(output, "Match: no match event");
        }

        output
    }
}

/// A single entry inside `match_previews` from the upcoming list
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct UpcomingMatchPreview {
    pub id: u32,
    pub date: StringType, // returned by the API "21/05/2026"
    pub time: StringType,
    pub teams: SoccerTeams,
}

/// One league block inside the upcoming response `results` array
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct UpcomingLeagueBlock {
    pub league_id: u32,
    pub league_name: StringType,
    pub match_previews: Vec<UpcomingMatchPreview>,
}

/// Top-level response from `GET /match-previews-upcoming/?auth_token=<token>`
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct UpcomingFixturesResponse {
    pub count: u32,
    pub results: Vec<UpcomingLeagueBlock>,
}

impl UpcomingFixturesResponse {
    /// Find the soonest upcoming match (on or after `today`) that involves
    /// the given `team_id` (either as home or away)
    ///
    /// Returns `None` when no matching fixture is found
    #[must_use]
    pub fn find_next_for_team(
        &self,
        team_id: u32,
        today: NaiveDate,
    ) -> Option<&UpcomingMatchPreview> {
        self.results
            .iter()
            .flat_map(|block| &block.match_previews)
            .filter(|m| {
                // skip entries where either side is "None" (placeholder fixtures)
                let home_valid = m.teams.home.name.as_deref().map_or("None", |v| v) != "None";
                let away_valid = m.teams.away.name.as_deref().map_or("None", |v| v) != "None";

                if !home_valid || !away_valid {
                    return false;
                }

                let involves_team = m.teams.home.id == Some(team_id)
                    || m.teams.away.id == Some(team_id);

                if !involves_team {
                    return false;
                }

                // parse "DD/MM/YYYY"
                NaiveDate::parse_from_str(&m.date, "%d/%m/%Y")
                    .map(|d| d >= today)
                    .unwrap_or(false)
            })
            .min_by_key(|m| {
                // pick the earliest date
                NaiveDate::parse_from_str(&m.date, "%d/%m/%Y").unwrap_or(NaiveDate::MAX)
            })
    }

    /// Return every unique team found across all upcoming fixtures
    /// sorted by name, whose name contains `query` (case-insensitive)
    #[must_use]
    pub fn search_teams(&self, query: &str) -> Vec<(u32, StringType)> {
        let query_lower = query.to_lowercase();

        let mut seen: std::collections::HashMap<u32, StringType> = std::collections::HashMap::new();

        for block in &self.results {
            for m in &block.match_previews {
                for team in [&m.teams.home, &m.teams.away] {
                    if let (Some(id), Some(name)) = (team.id, &team.name) {
                        if name != "None" && name.to_lowercase().contains(&query_lower) {
                            seen.entry(id).or_insert_with(|| name.clone());
                        }
                    }
                }
            }
        }

        let mut results: Vec<(u32, StringType)> = seen.into_iter().collect();
        results.sort_by(|a, b| a.1.cmp(&b.1));
        results
    }
}

#[cfg(feature = "cli")]
#[derive(Default, Clone)]
pub struct SoccerApi {
    client: Client,
    auth_token: ApiStringType,
    soccer_endpoint: StringType,
}

#[cfg(feature = "cli")]
impl SoccerApi {
    #[must_use]
    pub fn new(auth_token: &str, soccer_endpoint: &str) -> Self {
        Self {
            client: Client::new(),
            auth_token: auth_token.into(),
            soccer_endpoint: soccer_endpoint.into(),
        }
    }

    /// Fetch the full upcoming fixtures list from `/match-previews-upcoming/`
    ///
    /// # Errors
    /// Returns an error if the HTTP request or JSON parsing fails
    pub async fn get_upcoming_fixtures(&self) -> Result<UpcomingFixturesResponse, Error> {
        let base_url = format!("https://{}/match-previews-upcoming/", self.soccer_endpoint);
        let url = Url::parse_with_params(&base_url, &[("auth_token", self.auth_token.as_str())])?;

        let bytes = self
            .client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;

        serde_json::from_slice::<UpcomingFixturesResponse>(&bytes).map_err(Into::into)
    }

    /// Fetch full match detail from `/match/?match_id=<id>`
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails, the server returns a non-2xx
    /// status, or the JSON cannot be parsed. Also returns an error when the API
    /// replies with a `{"detail": "error..."}` payload (match not found/API error)
    pub async fn get_match_detail(&self, match_id: u32) -> Result<SoccerMatchData, Error> {
        let base_url = format!("https://{}/match/", self.soccer_endpoint);
        let match_id_str: ApiStringType = apistringtype_from_display(match_id);
        let url = Url::parse_with_params(
            &base_url,
            &[
                ("match_id", match_id_str.as_str()),
                ("auth_token", self.auth_token.as_str()),
            ],
        )?;

        let bytes = self
            .client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;

        // the API might returns {"detail": "messages..."} with HTTP 200 on failure
        if let Ok(err_payload) = serde_json::from_slice::<SoccerApiError>(&bytes) {
            return Err(Error::InvalidValue(err_payload.detail));
        }

        serde_json::from_slice::<SoccerMatchData>(&bytes).map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use crate::{
        soccer_api::{
            SoccerGoals, SoccerMatchData, SoccerTeam, SoccerTeams, UpcomingFixturesResponse,
        },
        Error,
    };

    #[test]
    fn test_goals_pre_match_all_minus_one() {
        // when the API sends -1 for all fields (pre-match), both sides = 0
        let goals = SoccerGoals {
            home_ft_goals: -1,
            away_ft_goals: -1,
            home_ht_goals: -1,
            away_ht_goals: -1,
            home_et_goals: -1,
            away_et_goals: -1,
            home_pen_goals: -1,
            away_pen_goals: -1,
        };
        assert_eq!(goals.home(), 0);
        assert_eq!(goals.away(), 0);
    }

    #[test]
    fn test_goals_zero_zero() {
        let goals = SoccerGoals {
            home_ft_goals: 0,
            away_ft_goals: 0,
            ..Default::default()
        };
        assert_eq!(goals.home(), 0);
        assert_eq!(goals.away(), 0);
    }

    #[test]
    fn test_goals_live_score() {
        let goals = SoccerGoals {
            home_ft_goals: 2,
            away_ft_goals: 1,
            home_ht_goals: 1,
            away_ht_goals: 0,
            home_et_goals: -1,
            away_et_goals: -1,
            home_pen_goals: -1,
            away_pen_goals: -1,
        };
        assert_eq!(goals.home(), 2);
        assert_eq!(goals.away(), 1);
    }

    #[test]
    fn test_match_detail_pre_match_parse() -> Result<(), Error> {
        let buf = include_str!("../tests/resource/match_detail.json");
        let data: SoccerMatchData = serde_json::from_str(buf)?;

        assert_eq!(data.id, Some(962063));
        assert_eq!(data.status.as_ref().map(|s| s.as_str()), Some("pre-match"));

        let teams = data.teams.as_ref().expect("teams should be present");
        assert_eq!(
            teams.home.name.as_ref().map(|s| s.as_str()),
            Some("Barcelona")
        );
        assert_eq!(
            teams.away.name.as_ref().map(|s| s.as_str()),
            Some("Real Madrid")
        );

        let goals = data.goals.as_ref().expect("goals should be present");
        assert_eq!(goals.home(), 0);
        assert_eq!(goals.away(), 0);

        Ok(())
    }

    #[test]
    fn test_match_detail_live_parse() -> Result<(), Error> {
        let buf = include_str!("../tests/resource/match_detail_live.json");
        let data: SoccerMatchData = serde_json::from_str(buf)?;

        assert_eq!(data.status.as_ref().map(|s| s.as_str()), Some("live"));

        let goals = data.goals.as_ref().expect("goals should be present");
        assert_eq!(goals.home(), 2);
        assert_eq!(goals.away(), 1);

        Ok(())
    }

    #[test]
    fn test_get_current_fixtures_pre_match() -> Result<(), Error> {
        let buf = include_str!("../tests/resource/match_detail.json");
        let data: SoccerMatchData = serde_json::from_str(buf)?;
        let output = data.get_current_fixtures();

        assert!(
            output.starts_with("Match: Barcelona 0 vs 0 Real Madrid"),
            "unexpected output: {output}"
        );
        assert!(output.contains("Status: pre-match"));
        assert!(output.contains("Home team: Barcelona"));
        assert!(output.contains("Away team: Real Madrid"));
        Ok(())
    }

    #[test]
    fn test_get_current_fixtures_live() -> Result<(), Error> {
        let buf = include_str!("../tests/resource/match_detail_live.json");
        let data: SoccerMatchData = serde_json::from_str(buf)?;
        let output = data.get_current_fixtures();

        assert!(
            output.starts_with("Match: Barcelona 2 vs 1 Real Madrid"),
            "unexpected output: {output}"
        );
        assert!(output.contains("Status: live"));
        Ok(())
    }

    #[test]
    fn test_get_current_fixtures_no_teams() {
        // when teams is None (e.g. malformed response) fall back gracefully
        let data = SoccerMatchData::default();
        let output = data.get_current_fixtures();
        assert_eq!(output, "Match: no match event");
    }

    #[test]
    fn test_upcoming_parse() -> Result<(), Error> {
        let buf = include_str!("../tests/resource/upcoming.json");
        let data: UpcomingFixturesResponse = serde_json::from_str(buf)?;

        assert_eq!(data.count, 3);
        assert_eq!(data.results.len(), 2);

        let la_liga = &data.results[0];
        assert_eq!(la_liga.league_name.as_str(), "La Liga");
        assert_eq!(la_liga.match_previews.len(), 3);

        Ok(())
    }

    fn load_upcoming() -> UpcomingFixturesResponse {
        let buf = include_str!("../tests/resource/upcoming.json");
        serde_json::from_str(buf).expect("valid upcoming.json")
    }

    #[test]
    fn test_find_next_for_team_found() {
        let data = load_upcoming();
        // Example two fixtures:
        //
        // Barcelona (4884) has two fixtures: 10/05 and 13/05
        // With today = 10/05, the earliest is 10/05 (id 962063)
        let today = NaiveDate::from_ymd_opt(2026, 5, 10).unwrap();
        let result = data.find_next_for_team(4884, today);

        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.id, 962063);
        assert_eq!(m.date.as_str(), "10/05/2026");
    }

    #[test]
    fn test_find_next_for_team_skips_past_dates() {
        let data = load_upcoming();
        // With today = 11/05, the 10/05 Barcelona match is in the past
        // the next one should be 13/05 (id 962070)
        let today = NaiveDate::from_ymd_opt(2026, 5, 11).unwrap();
        let result = data.find_next_for_team(4884, today);

        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.id, 962070);
        assert_eq!(m.date.as_str(), "13/05/2026");
    }

    #[test]
    fn test_find_next_for_team_not_found() {
        let data = load_upcoming();
        // Team ID 9999 does not exist in the fixture data
        let today = NaiveDate::from_ymd_opt(2026, 5, 10).unwrap();
        let result = data.find_next_for_team(9999, today);
        assert!(result.is_none());
    }

    #[test]
    fn test_find_next_for_team_skips_none_placeholders() {
        let data = load_upcoming();
        // Team ID 18798 appears only in a "None vs None" placeholder fixture
        // and must be skipped
        let today = NaiveDate::from_ymd_opt(2026, 5, 10).unwrap();
        let result = data.find_next_for_team(18798, today);
        assert!(result.is_none());
    }

    #[test]
    fn test_find_next_for_team_all_dates_past() {
        let data = load_upcoming();
        // If today is far in the future, no fixture qualifies
        let today = NaiveDate::from_ymd_opt(2030, 1, 1).unwrap();
        let result = data.find_next_for_team(4884, today);
        assert!(result.is_none());
    }

    #[test]
    fn test_search_teams_found() {
        let data = load_upcoming();
        let results = data.search_teams("barcelona");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, 4884);
        assert_eq!(results[0].1.as_str(), "Barcelona");
    }

    #[test]
    fn test_search_teams_case_insensitive() {
        let data = load_upcoming();
        // "REAL" should match "Real Madrid" and "Real Oviedo"
        // (not in this fixture, but "Real Madrid" is)
        let results = data.search_teams("REAL");
        assert!(!results.is_empty());
        assert!(results.iter().any(|(_, name)| name.contains("Real Madrid")));
    }

    #[test]
    fn test_search_teams_partial_match() {
        let data = load_upcoming();
        // "man" should match "Manchester United"
        let results = data.search_teams("man");
        assert!(results.iter().any(|(_, name)| name.contains("Manchester")));
    }

    #[test]
    fn test_search_teams_not_found() {
        let data = load_upcoming();
        let results = data.search_teams("zzznomatch");
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_teams_excludes_none_placeholders() {
        let data = load_upcoming();
        // the placeholder team named "None" must never appear in results
        let results = data.search_teams("None");
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_teams_deduplicates() {
        let data = load_upcoming();
        // Barcelona appears in two fixtures (home and away), must appear once
        let results = data.search_teams("barcelona");
        let barca_count = results.iter().filter(|(id, _)| *id == 4884).count();
        assert_eq!(barca_count, 1);
    }

    #[test]
    fn test_search_teams_sorted_by_name() {
        let data = load_upcoming();
        // a broad query returns multiple teams, results must be sorted
        let results = data.search_teams("a");
        let names: Vec<&str> = results.iter().map(|(_, n)| n.as_str()).collect();
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted);
    }

    #[test]
    fn test_match_data_with_no_goals_field() {
        // goals is Option if absent entirely, score defaults to 0 vs 0
        let data = SoccerMatchData {
            id: Some(1),
            date: Some("01/06/2026".into()),
            time: Some("20:00".into()),
            status: Some("pre-match".into()),
            teams: Some(SoccerTeams {
                home: SoccerTeam {
                    id: Some(1),
                    name: Some("Home FC".into()),
                },
                away: SoccerTeam {
                    id: Some(2),
                    name: Some("Away FC".into()),
                },
            }),
            goals: None,
        };

        let output = data.get_current_fixtures();
        assert!(output.starts_with("Match: Home FC 0 vs 0 Away FC"));
    }
}
