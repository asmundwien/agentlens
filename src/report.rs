use std::collections::HashMap;

use chrono::{DateTime, Duration, NaiveDate, SecondsFormat, Utc};
use rusqlite::params;
use serde::Serialize;
use thiserror::Error;

use crate::model::{Client, Initiator};
use crate::storage::Store;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Query {
    since: Option<DateTime<Utc>>,
    until: Option<DateTime<Utc>>,
}

impl Query {
    pub fn parse(
        since: Option<&str>,
        until: Option<&str>,
        now: DateTime<Utc>,
    ) -> Result<Self, QueryError> {
        let since = since.map(|value| parse_since(value, now)).transpose()?;
        let until = until.map(parse_until).transpose()?;

        if matches!((&since, &until), (Some(since), Some(until)) if since > until) {
            return Err(QueryError::Reversed);
        }

        Ok(Self { since, until })
    }

    fn since_text(&self) -> Option<String> {
        self.since
            .map(|value| value.to_rfc3339_opts(SecondsFormat::AutoSi, true))
    }
    fn since_storage_text(&self) -> Option<String> {
        self.since
            .map(|value| value.to_rfc3339_opts(SecondsFormat::Nanos, true))
    }

    fn until_text(&self) -> Option<String> {
        self.until
            .map(|value| value.to_rfc3339_opts(SecondsFormat::AutoSi, true))
    }
    fn until_storage_text(&self) -> Option<String> {
        self.until
            .map(|value| value.to_rfc3339_opts(SecondsFormat::Nanos, true))
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum QueryError {
    #[error("invalid --since value; use 7d, an ISO date, or an RFC 3339 timestamp")]
    InvalidSince,
    #[error("invalid --until value; use an ISO date or an RFC 3339 timestamp")]
    InvalidUntil,
    #[error("--since must not be later than --until")]
    Reversed,
}

fn parse_since(value: &str, now: DateTime<Utc>) -> Result<DateTime<Utc>, QueryError> {
    if let Some(absolute) = parse_absolute(value) {
        return Ok(absolute);
    }
    parse_relative(value, now).ok_or(QueryError::InvalidSince)
}

fn parse_until(value: &str) -> Result<DateTime<Utc>, QueryError> {
    parse_absolute(value).ok_or(QueryError::InvalidUntil)
}

fn parse_absolute(value: &str) -> Option<DateTime<Utc>> {
    if let Ok(timestamp) = DateTime::parse_from_rfc3339(value) {
        return Some(timestamp.with_timezone(&Utc));
    }

    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .ok()?
        .and_hms_opt(0, 0, 0)
        .map(|timestamp| timestamp.and_utc())
}

fn parse_relative(value: &str, now: DateTime<Utc>) -> Option<DateTime<Utc>> {
    let (amount, unit) = value.split_at(value.len().checked_sub(1)?);
    let amount: i64 = amount.parse().ok()?;
    let seconds_per_unit = match unit {
        "s" => 1,
        "m" => 60,
        "h" => 60 * 60,
        "d" => 24 * 60 * 60,
        "w" => 7 * 24 * 60 * 60,
        _ => return None,
    };
    let seconds = amount.checked_mul(seconds_per_unit)?;
    now.checked_sub_signed(Duration::seconds(seconds))
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct BucketCount {
    pub client: Client,
    pub initiator: Initiator,
    pub observed_invocation_count: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SkillCount {
    pub skill: String,
    pub observed_invocation_count: u64,
    pub by_client_and_initiator: Vec<BucketCount>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ReportQuery {
    pub since: Option<String>,
    pub until: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Report {
    pub query: ReportQuery,
    pub observed_invocation_count: u64,
    pub skill_count: usize,
    pub skills: Vec<SkillCount>,
}

#[derive(Debug, Error)]
pub enum ReportError {
    #[error("failed to read Agentlens storage")]
    Storage(#[from] rusqlite::Error),
    #[error("Agentlens storage contains an invalid client or initiator")]
    InvalidStoredValue,
    #[error("an aggregate count exceeded the supported range")]
    CountOverflow,
}

impl Report {
    pub fn load(store: &Store, query: &Query) -> Result<Self, ReportError> {
        let since = query.since_text();
        let until = query.until_text();
        let storage_since = query.since_storage_text();
        let storage_until = query.until_storage_text();
        let mut statement = store.connection().prepare(
            "SELECT skill, client, initiator, COUNT(*)
             FROM usage_events
             WHERE (?1 IS NULL OR observed_at >= ?1)
               AND (?2 IS NULL OR observed_at < ?2)
             GROUP BY skill, client, initiator",
        )?;
        let rows = statement.query_map(
            params![storage_since.as_deref(), storage_until.as_deref()],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, u64>(3)?,
                ))
            },
        )?;

        let mut counts: HashMap<String, [u64; 4]> = HashMap::new();
        for row in rows {
            let (skill, client, initiator, count) = row?;
            let client = Client::from_db(&client).ok_or(ReportError::InvalidStoredValue)?;
            let initiator =
                Initiator::from_db(&initiator).ok_or(ReportError::InvalidStoredValue)?;
            let index = bucket_index(client, initiator);
            counts.entry(skill).or_insert([0; 4])[index] = count;
        }

        let mut skills = counts
            .into_iter()
            .map(|(skill, counts)| {
                let observed_invocation_count = counts
                    .iter()
                    .try_fold(0_u64, |total, count| total.checked_add(*count))
                    .ok_or(ReportError::CountOverflow)?;
                Ok(SkillCount {
                    skill,
                    observed_invocation_count,
                    by_client_and_initiator: bucket_counts(counts),
                })
            })
            .collect::<Result<Vec<_>, ReportError>>()?;
        skills.sort_by(|left, right| {
            right
                .observed_invocation_count
                .cmp(&left.observed_invocation_count)
                .then_with(|| left.skill.cmp(&right.skill))
        });

        let observed_invocation_count = skills.iter().try_fold(0_u64, |total, skill| {
            total.checked_add(skill.observed_invocation_count)
        });
        let observed_invocation_count =
            observed_invocation_count.ok_or(ReportError::CountOverflow)?;

        Ok(Self {
            query: ReportQuery { since, until },
            observed_invocation_count,
            skill_count: skills.len(),
            skills,
        })
    }

    pub fn render_table(&self) -> String {
        let mut lines = vec![
            scope_heading(&self.query),
            format!(
                "{} observed invocations across {} skills",
                self.observed_invocation_count, self.skill_count
            ),
            String::new(),
        ];

        if self.skills.is_empty() {
            lines.push("No observed invocations in this scope.".to_owned());
            return lines.join("\n");
        }

        for skill in &self.skills {
            let counts = &skill.by_client_and_initiator;
            lines.push(format!(
                "{}  {}",
                skill.skill, skill.observed_invocation_count
            ));
            lines.push(format!(
                "  OMP          user {:>2}   model {:>2}",
                counts[0].observed_invocation_count, counts[1].observed_invocation_count
            ));
            lines.push(format!(
                "  Claude Code  user {:>2}   model {:>2}",
                counts[2].observed_invocation_count, counts[3].observed_invocation_count
            ));
            lines.push(String::new());
        }
        while lines.last().is_some_and(String::is_empty) {
            lines.pop();
        }
        lines.join("\n")
    }
}

fn bucket_index(client: Client, initiator: Initiator) -> usize {
    match (client, initiator) {
        (Client::Omp, Initiator::User) => 0,
        (Client::Omp, Initiator::Model) => 1,
        (Client::ClaudeCode, Initiator::User) => 2,
        (Client::ClaudeCode, Initiator::Model) => 3,
    }
}

fn bucket_counts(counts: [u64; 4]) -> Vec<BucketCount> {
    [
        (Client::Omp, Initiator::User),
        (Client::Omp, Initiator::Model),
        (Client::ClaudeCode, Initiator::User),
        (Client::ClaudeCode, Initiator::Model),
    ]
    .into_iter()
    .zip(counts)
    .map(
        |((client, initiator), observed_invocation_count)| BucketCount {
            client,
            initiator,
            observed_invocation_count,
        },
    )
    .collect()
}

fn scope_heading(query: &ReportQuery) -> String {
    match (&query.since, &query.until) {
        (None, None) => "ALL RETAINED EVENTS".to_owned(),
        (Some(since), None) => format!("EVENTS SINCE {since} (INCLUSIVE)"),
        (None, Some(until)) => format!("EVENTS UNTIL {until} (EXCLUSIVE)"),
        (Some(since), Some(until)) => format!("EVENTS IN [{since}, {until})"),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use chrono::TimeZone;
    use uuid::Uuid;

    use super::*;
    use crate::model::UsageEvent;
    use crate::storage::Store;

    fn temporary_database() -> PathBuf {
        std::env::temp_dir().join(format!("agentlens-report-{}.sqlite3", Uuid::now_v7()))
    }

    fn timestamp(second: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 8, 23, 12, 0, second)
            .single()
            .unwrap()
    }

    #[test]
    fn applies_half_open_bounds_and_renders_fixed_bucket_order() {
        let path = temporary_database();
        let mut store = Store::open(&path).unwrap();
        for (second, client, skill, initiator) in [
            (0, Client::Omp, "zeta", Initiator::User),
            (1, Client::ClaudeCode, "alpha", Initiator::Model),
            (2, Client::Omp, "alpha", Initiator::Model),
            (3, Client::Omp, "excluded", Initiator::User),
        ] {
            let observed_at = timestamp(second)
                + if second == 1 {
                    Duration::milliseconds(500)
                } else {
                    Duration::zero()
                };
            let event = UsageEvent::at(client, skill.to_owned(), initiator, observed_at).unwrap();
            store.insert(&event).unwrap();
        }

        let query = Query::parse(
            Some("2026-08-23T12:00:01Z"),
            Some("2026-08-23T12:00:03Z"),
            timestamp(10),
        )
        .unwrap();
        let report = Report::load(&store, &query).unwrap();

        assert_eq!(report.observed_invocation_count, 2);
        assert_eq!(report.skill_count, 1);
        assert_eq!(report.skills[0].skill, "alpha");
        assert_eq!(
            report.skills[0]
                .by_client_and_initiator
                .iter()
                .map(|bucket| bucket.observed_invocation_count)
                .collect::<Vec<_>>(),
            [0, 1, 0, 1]
        );
        assert!(
            report
                .render_table()
                .contains("OMP          user  0   model  1")
        );

        drop(store);
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(path.with_extension("sqlite3-shm"));
        let _ = std::fs::remove_file(path.with_extension("sqlite3-wal"));
    }

    #[test]
    fn parses_dates_relative_since_and_equal_empty_window() {
        let now = timestamp(10);
        assert_eq!(
            Query::parse(Some("1d"), None, now).unwrap().since_text(),
            Some("2026-08-22T12:00:10Z".to_owned())
        );
        assert_eq!(
            Query::parse(Some("2026-08-23"), None, now)
                .unwrap()
                .since_text(),
            Some("2026-08-23T00:00:00Z".to_owned())
        );
        assert!(Query::parse(Some("1d"), Some("1d"), now).is_err());
        assert!(Query::parse(Some("2026-08-24"), Some("2026-08-23"), now).is_err());
        assert!(Query::parse(Some("2026-08-23"), Some("2026-08-23"), now).is_ok());
    }
}
