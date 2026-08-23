use std::fmt;

use chrono::{DateTime, SecondsFormat, Utc};
use clap::ValueEnum;
use serde::Serialize;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
#[value(rename_all = "snake_case")]
pub enum Client {
    Omp,
    ClaudeCode,
}

impl Client {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Omp => "omp",
            Self::ClaudeCode => "claude_code",
        }
    }

    pub(crate) fn from_db(value: &str) -> Option<Self> {
        match value {
            "omp" => Some(Self::Omp),
            "claude_code" => Some(Self::ClaudeCode),
            _ => None,
        }
    }
}

impl fmt::Display for Client {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
#[value(rename_all = "snake_case")]
pub enum Initiator {
    User,
    Model,
}

impl Initiator {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Model => "model",
        }
    }

    pub(crate) fn from_db(value: &str) -> Option<Self> {
        match value {
            "user" => Some(Self::User),
            "model" => Some(Self::Model),
            _ => None,
        }
    }
}

impl fmt::Display for Initiator {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UsageEvent {
    pub event_id: Uuid,
    pub observed_at: String,
    pub client: Client,
    pub skill: String,
    pub initiator: Initiator,
}

impl UsageEvent {
    pub fn observed(client: Client, skill: String, initiator: Initiator) -> Option<Self> {
        Self::at(client, skill, initiator, Utc::now())
    }

    pub(crate) fn at(
        client: Client,
        skill: String,
        initiator: Initiator,
        observed_at: DateTime<Utc>,
    ) -> Option<Self> {
        if skill.trim().is_empty() {
            return None;
        }

        Some(Self {
            event_id: Uuid::now_v7(),
            observed_at: observed_at.to_rfc3339_opts(SecondsFormat::Nanos, true),
            client,
            skill,
            initiator,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collector_generates_uuid_v7_and_utc_observation_time() {
        let first = UsageEvent::observed(Client::Omp, "tdd".to_owned(), Initiator::User).unwrap();
        let second = UsageEvent::observed(Client::Omp, "tdd".to_owned(), Initiator::User).unwrap();

        assert_eq!(first.event_id.get_version_num(), 7);
        assert_ne!(first.event_id, second.event_id);
        assert!(first.observed_at.ends_with('Z'));
        assert!(DateTime::parse_from_rfc3339(&first.observed_at).is_ok());
    }
}
