use chrono::{DateTime, NaiveDate};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Write;

use crate::StringType;

#[cfg(feature = "cli")]
use reqwest::{Client, Url};

#[cfg(feature = "cli")]
use crate::{apistringtype_from_display, ApiStringType, Error};

/// Deserialize, a shape this client doesn't expect yields the
/// type's default instead of failing the whole payload
///
/// The provider changes response details without notice, and a single unfamiliar
/// field is not a reason to lose the rest of a fixture list
fn lenient<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: DeserializeOwned + Default,
{
    let value = Value::deserialize(deserializer)?;
    Ok(serde_json::from_value(value).unwrap_or_default())
}

/// Kickoff timestamps are RFC 3339 UTC (`2026-09-12T14:00:00Z`) fall back to
/// the bare `YYYY-MM-DD` prefix if the offset is ever missing
#[must_use]
fn parse_utc_date(utc_date: &str) -> Option<NaiveDate> {
    DateTime::parse_from_rfc3339(utc_date)
        .map(|dt| dt.date_naive())
        .ok()
        .or_else(|| NaiveDate::parse_from_str(utc_date.get(0..10)?, "%Y-%m-%d").ok())
}

/// Split a kickoff timestamp into `(date, time)`: `2026-09-12T14:00:00Z` becomes
/// `("2026-09-12", "14:00")`
#[must_use]
fn split_kickoff(utc_date: &str) -> Option<(&str, &str)> {
    Some((utc_date.get(0..10)?, utc_date.get(11..16)?))
}

/// User-facing status text, the API sends enum names (`IN_PLAY`, `TIMED`) which
/// read better in lowercase
#[must_use]
pub fn status_display(status: &str) -> &'static str {
    match status.to_ascii_uppercase().as_str() {
        "IN_PLAY" => "live",
        "PAUSED" => "paused",
        "TIMED" => "timed",
        "SCHEDULED" => "scheduled",
        "FINISHED" => "finished",
        "POSTPONED" => "postponed",
        "CANCELLED" => "cancelled",
        "SUSPENDED" => "suspended",
        "AWARDED" => "awarded",
        _ => "unknown",
    }
}

/// The area a competition is played in e.g. `{ id: 2072, name: "England", code: "ENG" }`
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct CompetitionArea {
    #[serde(default, deserialize_with = "lenient")]
    pub id: u32,
    #[serde(default, deserialize_with = "lenient")]
    pub name: StringType,
    #[serde(default, deserialize_with = "lenient")]
    pub code: StringType,
}

/// One entry from `GET /competitions`
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct Competition {
    #[serde(default, deserialize_with = "lenient")]
    pub id: u32,
    #[serde(default, deserialize_with = "lenient")]
    pub name: StringType,
    #[serde(default, deserialize_with = "lenient")]
    pub code: StringType,
    #[serde(default, deserialize_with = "lenient")]
    pub area: Option<CompetitionArea>,
}

/// Top-level response from `GET /competitions`
///
/// Deserialization never fails an entry that cannot be read is dropped, so an
/// unfamiliar payload yields no competitions rather than an error the caller
/// has to show its user
#[derive(Serialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct CompetitionsResponse {
    pub competitions: Vec<Competition>,
}

impl<'de> Deserialize<'de> for CompetitionsResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;

        let competitions = value
            .get("competitions")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|entry| serde_json::from_value::<Competition>(entry).ok())
            .collect();

        Ok(Self { competitions })
    }
}

/// One team from `GET /competitions/{id}/teams`
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TeamEntry {
    #[serde(default, deserialize_with = "lenient")]
    pub id: u32,
    #[serde(default, deserialize_with = "lenient")]
    pub name: StringType,
    #[serde(default, deserialize_with = "lenient")]
    pub short_name: Option<StringType>,
}

/// Top-level response from `GET /competitions/{id}/teams`
///
/// Also carries the competition the teams belong to, which is what gets shown
/// to the user when nothing matches
#[derive(Serialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct TeamsResponse {
    pub competition_name: StringType,
    pub teams: Vec<TeamEntry>,
}

impl<'de> Deserialize<'de> for TeamsResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;

        let competition_name = value
            .get("competition")
            .and_then(|c| c.get("name"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .into();

        let teams = value
            .get("teams")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|entry| serde_json::from_value::<TeamEntry>(entry).ok())
            .collect();

        Ok(Self {
            competition_name,
            teams,
        })
    }
}

impl TeamsResponse {
    /// Every team whose `name` or `shortName` contains `query`
    /// (case-insensitive) sorted by display name
    #[must_use]
    pub fn search_teams(&self, query: &str) -> Vec<(u32, StringType)> {
        let query_lower = query.to_lowercase();

        let mut results: Vec<(u32, StringType)> = self
            .teams
            .iter()
            .filter(|team| {
                let name_match = team.name.to_lowercase().contains(&query_lower);
                let short_match = team
                    .short_name
                    .as_ref()
                    .is_some_and(|short| short.to_lowercase().contains(&query_lower));

                name_match || short_match
            })
            .map(|team| (team.id, team.display_name().into()))
            .collect();

        results.sort_by(|a, b| a.1.cmp(&b.1));
        results
    }
}

impl TeamEntry {
    /// `shortName` is what fans call the club (`Liverpool`) the long `name`
    /// (`Liverpool FC`) is the fallback
    #[must_use]
    pub fn display_name(&self) -> &str {
        self.short_name
            .as_ref()
            .map_or_else(|| self.name.as_str(), StringType::as_str)
    }
}

/// A competition reference nested in a match, e.g. `{id: 2021, name: "Premier League"}`
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct CompetitionRef {
    #[serde(default, deserialize_with = "lenient")]
    pub id: u32,
    #[serde(default, deserialize_with = "lenient")]
    pub name: StringType,
    #[serde(default, deserialize_with = "lenient")]
    pub code: StringType,
}

/// One side of a match
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MatchSide {
    #[serde(default, deserialize_with = "lenient")]
    pub id: Option<u32>,
    #[serde(default, deserialize_with = "lenient")]
    pub name: StringType,
    #[serde(default, deserialize_with = "lenient")]
    pub short_name: Option<StringType>,
}

impl MatchSide {
    #[must_use]
    pub fn display_name(&self) -> &str {
        self.short_name
            .as_ref()
            .map_or_else(|| self.name.as_str(), StringType::as_str)
    }
}

/// A home/away goal pair, `None` before kickoff or where the API omits it
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ScorePair {
    #[serde(default, deserialize_with = "lenient")]
    pub home: Option<i32>,
    #[serde(default, deserialize_with = "lenient")]
    pub away: Option<i32>,
}

/// The score block of a match, `fullTime` is what gets displayed
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MatchScore {
    #[serde(default, deserialize_with = "lenient")]
    pub full_time: Option<ScorePair>,
    #[serde(default, deserialize_with = "lenient")]
    pub half_time: Option<ScorePair>,
}

/// One match from `GET /teams/{id}/matches`
///
/// Carries its own status, minute and score, so a single request answers both
/// "what is next" and "what is the score"
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TeamMatch {
    #[serde(default, deserialize_with = "lenient")]
    pub id: u32,
    #[serde(default, deserialize_with = "lenient")]
    pub utc_date: StringType,
    #[serde(default, deserialize_with = "lenient")]
    pub status: StringType,
    #[serde(default, deserialize_with = "lenient")]
    pub minute: Option<u32>,
    #[serde(default, deserialize_with = "lenient")]
    pub score: Option<MatchScore>,
    #[serde(default, deserialize_with = "lenient")]
    pub home_team: Option<MatchSide>,
    #[serde(default, deserialize_with = "lenient")]
    pub away_team: Option<MatchSide>,
    #[serde(default, deserialize_with = "lenient")]
    pub competition: Option<CompetitionRef>,
}

impl TeamMatch {
    /// Whether the match is being played right now
    #[must_use]
    pub fn is_in_play(&self) -> bool {
        self.status.eq_ignore_ascii_case("IN_PLAY") || self.status.eq_ignore_ascii_case("PAUSED")
    }

    /// Whether the match is over
    #[must_use]
    pub fn is_finished(&self) -> bool {
        self.status.eq_ignore_ascii_case("FINISHED") || self.status.eq_ignore_ascii_case("AWARDED")
    }

    /// Whether the match is still to be played
    #[must_use]
    pub fn is_scheduled(&self) -> bool {
        self.status.eq_ignore_ascii_case("SCHEDULED") || self.status.eq_ignore_ascii_case("TIMED")
    }

    /// Full-time score `None` (not played) reads as `0`
    #[must_use]
    pub fn score(&self) -> (usize, usize) {
        let goals = |pair: Option<&ScorePair>, field: fn(&ScorePair) -> Option<i32>| {
            pair.and_then(field)
                .map_or(0, |v| usize::try_from(v.max(0)).unwrap_or(0))
        };

        self.score.as_ref().map_or((0, 0), |s| {
            (
                goals(s.full_time.as_ref(), |p| p.home),
                goals(s.full_time.as_ref(), |p| p.away),
            )
        })
    }

    /// Format the match in the same style the api-football side uses
    #[must_use]
    pub fn get_current_fixtures(&self) -> StringType {
        let mut output = StringType::new();

        let home_name = self
            .home_team
            .as_ref()
            .map_or("Unknown", MatchSide::display_name);
        let away_name = self
            .away_team
            .as_ref()
            .map_or("Unknown", MatchSide::display_name);
        let (home_score, away_score) = self.score();

        let _ = write!(
            output,
            "Match: {home_name} {home_score} vs {away_score} {away_name}"
        );

        if let Some((date, time)) = split_kickoff(&self.utc_date) {
            let _ = write!(output, "\nNext match on {date} at {time}");
        }

        let _ = write!(output, "\n\tStatus: {}", status_display(&self.status));

        // only annotate the minute while the match is actually running
        if self.is_in_play() {
            if let Some(minute) = self.minute {
                let _ = write!(output, " ({minute}')");
            }
        }

        let _ = write!(output, "\n\tHome team: {home_name}");
        let _ = write!(output, "\n\tAway team: {away_name}");
        output.push('\n');

        output
    }
}

/// Top-level response from `GET /teams/{id}/matches`
///
/// a match that cannot be read is dropped
#[derive(Serialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct TeamMatchesResponse {
    pub matches: Vec<TeamMatch>,
}

impl<'de> Deserialize<'de> for TeamMatchesResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;

        let matches = value
            .get("matches")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|entry| serde_json::from_value::<TeamMatch>(entry).ok())
            .collect();

        Ok(Self { matches })
    }
}

impl TeamMatchesResponse {
    /// The match to show one in play if there is one, otherwise the soonest
    /// upcoming fixture on or after `today`
    #[must_use]
    pub fn next_match(&self, today: NaiveDate) -> Option<&TeamMatch> {
        if let Some(live) = self.matches.iter().find(|m| m.is_in_play()) {
            return Some(live);
        }

        self.matches
            .iter()
            .filter(|m| m.is_scheduled())
            .filter(|m| parse_utc_date(&m.utc_date).is_some_and(|d| d >= today))
            .min_by(|a, b| {
                // same kickoff date is broken on the full timestamp, which
                // compares chronologically in its fixed ISO layout
                let ka = (parse_utc_date(&a.utc_date), a.utc_date.as_str());
                let kb = (parse_utc_date(&b.utc_date), b.utc_date.as_str());
                ka.cmp(&kb)
            })
    }
}

/// How much of a response body `FOOTBALLSCORE_DEBUG` prints
#[cfg(feature = "cli")]
const DEBUG_BODY_LIMIT: usize = 4000;

/// Whether `FOOTBALLSCORE_DEBUG` is set
///
/// The provider reshapes its responses from time to time, so there has to be a
/// way to see what actually came back without that noise reaching normal users
#[cfg(feature = "cli")]
#[must_use]
pub fn debug_enabled() -> bool {
    std::env::var_os("FOOTBALLSCORE_DEBUG").is_some()
}

/// football-data.org client
///
/// Free tier 10 requests/minute, auth is the `X-Auth-Token` header
#[cfg(feature = "cli")]
#[derive(Default, Clone)]
pub struct FootballDataApi {
    client: Client,
    token: ApiStringType,
    endpoint: StringType,
}

#[cfg(feature = "cli")]
impl FootballDataApi {
    #[must_use]
    pub fn new(token: &str, endpoint: &str) -> Self {
        Self {
            client: Client::new(),
            token: token.into(),
            endpoint: endpoint.into(),
        }
    }

    /// Issue a GET with the auth header and return the body
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails or the body cannot be read
    /// A non-2xx status becomes an [`Error::ApiError`] carrying the status and
    /// the API's own message, so the caller can show something meaningful
    async fn get_checked(&self, url: Url) -> Result<Vec<u8>, Error> {
        // keep the path for debug output the full URL may carry credentials
        let path = StringType::from(url.path());

        let response = self
            .client
            .get(url)
            .header("X-Auth-Token", self.token.as_str())
            .send()
            .await?;

        let status = response.status();
        let bytes = response.bytes().await?;

        if debug_enabled() {
            let body = String::from_utf8_lossy(&bytes);
            let preview: String = body.chars().take(DEBUG_BODY_LIMIT).collect();
            let elided = if body.chars().count() > DEBUG_BODY_LIMIT {
                " [truncated]"
            } else {
                ""
            };

            eprintln!("[debug] GET {path} ({status}) -> {preview}{elided}");
        }

        if !status.is_success() {
            // the API explains failures in a JSON body e.g. a bad token comes
            // back as 400 with a "restricted" message
            let message = serde_json::from_slice::<Value>(&bytes)
                .ok()
                .and_then(|v| {
                    v.get("message")
                        .and_then(Value::as_str)
                        .map(StringType::from)
                })
                .unwrap_or_default();

            return Err(Error::ApiError {
                status: status.as_u16(),
                message,
            });
        }

        Ok(bytes.to_vec())
    }

    /// Fetch every competition the token can see from `/competitions`
    ///
    /// # Errors
    /// Returns an error if the HTTP request or JSON parsing fails
    pub async fn get_competitions(&self) -> Result<CompetitionsResponse, Error> {
        let url = Url::parse(&format!("https://{}/competitions", self.endpoint))?;
        let bytes = self.get_checked(url).await?;

        serde_json::from_slice::<CompetitionsResponse>(&bytes).map_err(Into::into)
    }

    /// Fetch a competition's teams from `/competitions/{id}/teams`
    ///
    /// `season` is optional, the API serves the current season without it
    ///
    /// # Errors
    /// Returns an error if the HTTP request or JSON parsing fails
    pub async fn get_teams(
        &self,
        competition_id: u32,
        season: Option<&str>,
    ) -> Result<TeamsResponse, Error> {
        let id: ApiStringType = apistringtype_from_display(competition_id);
        let mut url = Url::parse(&format!(
            "https://{}/competitions/{id}/teams",
            self.endpoint
        ))?;

        if let Some(season) = season.filter(|s| !s.is_empty()) {
            url.query_pairs_mut().append_pair("season", season);
        }

        let bytes = self.get_checked(url).await?;

        serde_json::from_slice::<TeamsResponse>(&bytes).map_err(Into::into)
    }

    /// Fetch a team's fixtures from `/teams/{id}/matches`
    ///
    /// Each match carries its own status, minute and score, so this single
    /// request covers both the next fixture and the live score
    ///
    /// `season` is optional, the API serves the current season without it
    ///
    /// # Errors
    /// Returns an error if the HTTP request or JSON parsing fails
    pub async fn get_team_matches(
        &self,
        team_id: u32,
        season: Option<&str>,
    ) -> Result<TeamMatchesResponse, Error> {
        let id: ApiStringType = apistringtype_from_display(team_id);
        let mut url = Url::parse(&format!("https://{}/teams/{id}/matches", self.endpoint))?;

        if let Some(season) = season.filter(|s| !s.is_empty()) {
            url.query_pairs_mut().append_pair("season", season);
        }

        let bytes = self.get_checked(url).await?;

        serde_json::from_slice::<TeamMatchesResponse>(&bytes).map_err(Into::into)
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use chrono::NaiveDate;

    use crate::{
        football_data::{
            status_display, CompetitionsResponse, MatchSide, TeamMatchesResponse, TeamsResponse,
        },
        Error,
    };

    fn load_competitions() -> CompetitionsResponse {
        let buf = include_str!("../tests/resource/competitions.json");
        serde_json::from_str(buf).expect("valid competitions.json")
    }

    fn load_teams() -> TeamsResponse {
        let buf = include_str!("../tests/resource/teams_pl.json");
        serde_json::from_str(buf).expect("valid teams_pl.json")
    }

    fn load_matches() -> TeamMatchesResponse {
        let buf = include_str!("../tests/resource/team_matches.json");
        serde_json::from_str(buf).expect("valid team_matches.json")
    }

    /// today, for every matches test below
    fn matches_today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 9, 9).expect("valid date")
    }

    #[test]
    fn test_competitions_parse() -> Result<(), Error> {
        let data = load_competitions();

        assert_eq!(data.competitions.len(), 4);
        assert_eq!(data.competitions[0].id, 2021);
        assert_eq!(data.competitions[0].name.as_str(), "Premier League");
        assert_eq!(data.competitions[0].code.as_str(), "PL");
        assert_eq!(
            data.competitions[0].area.as_ref().map(|a| a.name.as_str()),
            Some("England")
        );
        Ok(())
    }

    #[test]
    fn test_teams_parse() -> Result<(), Error> {
        let data = load_teams();

        assert_eq!(data.competition_name.as_str(), "Premier League");
        assert_eq!(data.teams.len(), 3);
        assert_eq!(data.teams[1].id, 64);
        assert_eq!(data.teams[1].display_name(), "Liverpool");
        Ok(())
    }

    #[test]
    fn test_search_teams_matches_short_name() {
        let data = load_teams();

        // "liverpool" matches shortName even though the long name is "Liverpool FC"
        let results = data.search_teams("liverpool");
        assert_eq!(results, vec![(64, "Liverpool".into())]);
    }

    #[test]
    fn test_search_teams_matches_long_name() {
        let data = load_teams();
        // the long name contains "FC"; the short name does not
        let results = data.search_teams("FC");
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_search_teams_case_insensitive_and_sorted() {
        let data = load_teams();
        // "c" hits every team's long name ("... FC")
        let results = data.search_teams("c");

        assert_eq!(results.len(), 3);
        let names: Vec<&str> = results.iter().map(|(_, n)| n.as_str()).collect();
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted);
    }

    #[test]
    fn test_search_teams_not_found() {
        let data = load_teams();
        assert!(data.search_teams("zzznomatch").is_empty());
    }

    #[test]
    fn test_matches_parse() -> Result<(), Error> {
        let data = load_matches();

        assert_eq!(data.matches.len(), 5);

        let finished = &data.matches[0];
        assert_eq!(finished.status.as_str(), "FINISHED");
        assert_eq!(finished.score(), (2, 2));

        Ok(())
    }

    #[test]
    fn test_next_match_prefers_in_play() {
        let data = load_matches();
        // Liverpool are 67' into the fabricated 590001 and also have fixtures
        // scheduled later, the running match is the one to show
        let m = data.next_match(matches_today()).expect("Liverpool match");

        assert_eq!(m.id, 590001);
        assert!(m.is_in_play());
    }

    #[test]
    fn test_next_match_skips_finished() {
        let data = load_matches();
        // strip the live match, the earliest upcoming one is the CL fixture
        // today at 19:00, ahead of the 12/09 league game
        let without_live = TeamMatchesResponse {
            matches: data
                .matches
                .iter()
                .filter(|m| !m.is_in_play())
                .cloned()
                .collect(),
        };
        let m = without_live
            .next_match(matches_today())
            .expect("upcoming match");

        assert_eq!(m.status.as_str(), "TIMED");
        assert_eq!(
            m.home_team.as_ref().map(MatchSide::display_name),
            Some("Liverpool")
        );
        assert_eq!(
            m.away_team.as_ref().map(MatchSide::display_name),
            Some("Atleti")
        );
    }

    #[test]
    fn test_next_match_skips_past_fixtures() {
        let data = load_matches();
        // from tomorrow, today's 19:00 CL game is gone, the next one is 12/09
        let tomorrow = NaiveDate::from_ymd_opt(2026, 9, 10).expect("valid date");
        let without_live = TeamMatchesResponse {
            matches: data
                .matches
                .iter()
                .filter(|m| !m.is_in_play())
                .cloned()
                .collect(),
        };
        let m = without_live.next_match(tomorrow).expect("upcoming match");

        assert!(m.utc_date.as_str().starts_with("2026-09-12"));
    }

    #[test]
    fn test_next_match_none_when_all_past() {
        // an in-play match always wins regardless of date, so strip it before
        // checking that old scheduled fixtures are ignored
        let without_live = TeamMatchesResponse {
            matches: load_matches()
                .matches
                .iter()
                .filter(|m| !m.is_in_play())
                .cloned()
                .collect(),
        };
        let today = NaiveDate::from_ymd_opt(2030, 1, 1).expect("valid date");
        assert!(without_live.next_match(today).is_none());
    }

    #[test]
    fn test_next_match_none_when_empty() {
        let data = TeamMatchesResponse::default();
        assert!(data.next_match(matches_today()).is_none());
    }

    #[test]
    fn test_live_display_shows_minute_and_score() {
        let data = load_matches();
        let m = data.next_match(matches_today()).expect("live match");
        let output = m.get_current_fixtures();

        assert!(
            output.starts_with("Match: Man City 1 vs 2 Liverpool"),
            "unexpected output: {output}"
        );
        assert!(
            output.contains("Status: live (67')"),
            "unexpected: {output}"
        );
        assert!(output.contains("Next match on 2026-09-09 at 18:30"));
    }

    #[test]
    fn test_scheduled_display_hides_minute() {
        let data = load_matches();
        let m = data
            .matches
            .iter()
            .find(|m| m.status.as_str() == "SCHEDULED")
            .expect("scheduled match");
        let output = m.get_current_fixtures();

        // scores are null before kickoff, and there is no minute to show
        assert!(
            output.starts_with("Match: Bournemouth 0 vs 0 Liverpool"),
            "unexpected output: {output}"
        );
        assert!(output.contains("Status: scheduled"));
        assert!(!output.contains("')"), "unexpected output: {output}");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(status_display("IN_PLAY"), "live");
        assert_eq!(status_display("PAUSED"), "paused");
        assert_eq!(status_display("TIMED"), "timed");
        assert_eq!(status_display("SCHEDULED"), "scheduled");
        assert_eq!(status_display("FINISHED"), "finished");
        assert_eq!(status_display("postponed"), "postponed");
        assert_eq!(status_display(""), "unknown");
    }

    #[test]
    fn test_unfamiliar_payloads_yield_no_matches() {
        // none of these may produce an error the CLI would have to show a user
        for buf in [
            r#"{"foo": "bar"}"#,
            r#""just a string""#,
            "[]",
            "null",
            r#"{"matches": [1, 2, 3]}"#,
        ] {
            let data: TeamMatchesResponse =
                serde_json::from_str(buf).unwrap_or_else(|e| panic!("{buf} failed to parse: {e}"));

            assert!(data.next_match(matches_today()).is_none());
        }
    }

    #[test]
    fn test_unfamiliar_payloads_yield_no_teams() {
        for buf in [r#"{"foo": "bar"}"#, "[]", "null"] {
            let data: TeamsResponse =
                serde_json::from_str(buf).unwrap_or_else(|e| panic!("{buf} failed to parse: {e}"));

            assert!(data.search_teams("liverpool").is_empty());
            assert!(data.competition_name.is_empty());
        }
    }

    #[test]
    fn test_bad_match_entry_does_not_drop_its_neighbours() {
        // one unreadable fixture must not cost us the rest of the team's list
        let buf = r#"{
            "matches": [
                {"id": "not-a-number", "homeTeam": "not-an-object"},
                {"id": 7, "utcDate": "2026-09-13T14:00:00Z", "status": "TIMED", "minute": null,
                 "score": null,
                 "homeTeam": {"id": 64, "name": "Liverpool FC", "shortName": "Liverpool"},
                 "awayTeam": {"id": 65, "name": "Manchester City FC", "shortName": "Man City"}}
            ]
        }"#;
        let data: TeamMatchesResponse = serde_json::from_str(buf).expect("should parse");

        let m = data.next_match(matches_today()).expect("Liverpool fixture");
        assert_eq!(m.id, 7);
        assert_eq!(m.score(), (0, 0));
    }

    #[test]
    fn test_unknown_status_displays_unknown() {
        let buf = r#"{
            "matches": [
                {"id": 8, "utcDate": "2026-09-13T14:00:00Z", "status": "SOMETHING_NEW",
                 "homeTeam": {"id": 64, "name": "Liverpool FC", "shortName": "Liverpool"},
                 "awayTeam": {"id": 65, "name": "Manchester City FC", "shortName": "Man City"}}
            ]
        }"#;
        let data: TeamMatchesResponse = serde_json::from_str(buf).expect("should parse");
        let m = &data.matches[0];

        assert!(m.get_current_fixtures().contains("Status: unknown"));
    }
}
