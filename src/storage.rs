use std::path::{Path, PathBuf};
use std::time::Duration;

use rusqlite::{Connection, TransactionBehavior, params};
use thiserror::Error;

use crate::model::UsageEvent;

const SCHEMA_VERSION: i64 = 1;
const BUSY_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("the HOME environment variable is unavailable")]
    HomeUnavailable,
    #[error("failed to prepare Agentlens storage")]
    CreateDirectory(#[source] std::io::Error),
    #[error("Agentlens storage uses unsupported schema version {0}")]
    UnsupportedSchema(i64),
    #[error("Agentlens storage is unavailable")]
    Sqlite(#[from] rusqlite::Error),
}

pub fn default_database_path() -> Result<PathBuf, StorageError> {
    let home = std::env::var_os("HOME").ok_or(StorageError::HomeUnavailable)?;
    Ok(PathBuf::from(home)
        .join("Library")
        .join("Application Support")
        .join("Agentlens")
        .join("agentlens.sqlite3"))
}

pub struct Store {
    connection: Connection,
}

impl Store {
    pub fn open_default() -> Result<Self, StorageError> {
        Self::open(default_database_path()?)
    }

    pub fn open(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(StorageError::CreateDirectory)?;
        }

        let mut connection = Connection::open(path)?;
        connection.busy_timeout(BUSY_TIMEOUT)?;
        connection.query_row("PRAGMA journal_mode = WAL", [], |_| Ok(()))?;
        connection.pragma_update(None, "synchronous", "NORMAL")?;

        let version: i64 = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
        match version {
            0 => {
                let transaction =
                    connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
                transaction.execute_batch(
                    "CREATE TABLE IF NOT EXISTS usage_events (
                        event_id TEXT PRIMARY KEY NOT NULL,
                        observed_at TEXT NOT NULL,
                        client TEXT NOT NULL CHECK (client IN ('omp', 'claude_code')),
                        skill TEXT NOT NULL CHECK (length(skill) > 0),
                        initiator TEXT NOT NULL CHECK (initiator IN ('user', 'model'))
                    );
                    CREATE INDEX IF NOT EXISTS usage_events_observed_at
                        ON usage_events(observed_at);
                    PRAGMA user_version = 1;",
                )?;
                transaction.commit()?;
            }
            SCHEMA_VERSION => {}
            other => return Err(StorageError::UnsupportedSchema(other)),
        }

        Ok(Self { connection })
    }

    pub fn insert(&mut self, event: &UsageEvent) -> Result<(), StorageError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        transaction.execute(
            "INSERT INTO usage_events (event_id, observed_at, client, skill, initiator)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                event.event_id.to_string(),
                event.observed_at,
                event.client.as_str(),
                event.skill,
                event.initiator.as_str(),
            ],
        )?;
        transaction.commit()?;
        Ok(())
    }

    pub(crate) fn connection(&self) -> &Connection {
        &self.connection
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Barrier};
    use std::thread;

    use chrono::Utc;
    use uuid::Uuid;

    use super::*;
    use crate::model::{Client, Initiator, UsageEvent};

    fn temporary_database() -> PathBuf {
        std::env::temp_dir().join(format!("agentlens-{}.sqlite3", Uuid::now_v7()))
    }

    #[test]
    fn persists_exactly_the_five_allowlisted_fields() {
        let path = temporary_database();
        let mut store = Store::open(&path).unwrap();
        let event = UsageEvent::at(
            Client::Omp,
            "Case:Sensitive".to_owned(),
            Initiator::Model,
            Utc::now(),
        )
        .unwrap();
        store.insert(&event).unwrap();

        let columns: Vec<String> = store
            .connection()
            .prepare("PRAGMA table_info(usage_events)")
            .unwrap()
            .query_map([], |row| row.get(1))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(
            columns,
            ["event_id", "observed_at", "client", "skill", "initiator"]
        );
        assert_eq!(
            store
                .connection()
                .query_row("SELECT skill FROM usage_events", [], |row| row
                    .get::<_, String>(0))
                .unwrap(),
            "Case:Sensitive"
        );

        drop(store);
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(path.with_extension("sqlite3-shm"));
        let _ = std::fs::remove_file(path.with_extension("sqlite3-wal"));
    }

    #[test]
    fn accepts_concurrent_one_shot_writers() {
        const WRITERS: usize = 64;
        let path = temporary_database();
        drop(Store::open(&path).unwrap());
        let barrier = Arc::new(Barrier::new(WRITERS));

        let handles: Vec<_> = (0..WRITERS)
            .map(|index| {
                let path = path.clone();
                let barrier = Arc::clone(&barrier);
                thread::spawn(move || {
                    barrier.wait();
                    let mut store = Store::open(path).unwrap();
                    let event = UsageEvent::observed(
                        Client::Omp,
                        format!("skill-{index}"),
                        Initiator::Model,
                    )
                    .unwrap();
                    store.insert(&event).unwrap();
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        let store = Store::open(&path).unwrap();
        let count: i64 = store
            .connection()
            .query_row("SELECT COUNT(*) FROM usage_events", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, WRITERS as i64);

        drop(store);
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(path.with_extension("sqlite3-shm"));
        let _ = std::fs::remove_file(path.with_extension("sqlite3-wal"));
    }
}
