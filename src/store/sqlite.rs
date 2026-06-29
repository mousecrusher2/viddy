use chrono::{DateTime, Local, Utc};
use color_eyre::Result;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::store::{Record, Store};
use crate::types::ExecutionId;

#[derive(Debug, Clone)]
pub struct SQLiteStore {
    conn: Arc<Mutex<Connection>>,
}

impl SQLiteStore {
    pub fn new(path: PathBuf, init: bool) -> Result<Self> {
        if init && path.exists() {
            std::fs::remove_file(&path)?;
        }

        // Pre-create the database file with owner-only permissions before SQLite
        // opens it, otherwise the backup is created world-readable (#195).
        #[cfg(unix)]
        if init {
            use std::os::unix::fs::OpenOptionsExt;
            std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o600)
                .open(&path)?;
        }

        let conn = Connection::open_with_flags(
            path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE | rusqlite::OpenFlags::SQLITE_OPEN_CREATE,
        )?;

        if init {
            conn.execute(
                "CREATE TABLE record (
                id INTEGER PRIMARY KEY,
                start_time TEXT NOT NULL,
                stdout BLOB NOT NULL,
                stderr BLOB NOT NULL,
                end_time TEXT NOT NULL,
                exit_code INTEGER NOT NULL,
                diff_add INTEGER,
                diff_delete INTEGER,
                previous_id INTEGER
            )",
                (),
            )?;

            conn.execute(
                "CREATE TABLE runtime_config (
                interval INTEGER NOT NULL,
                command TEXT NOT NULL
            )",
                (),
            )?;
        }

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }
}

impl Store for SQLiteStore {
    fn add_record(&mut self, record: Record) -> Result<()> {
        if let Ok(conn) = self.conn.lock() {
            conn.execute(
                "INSERT INTO record (
                id, start_time, stdout, stderr, end_time, exit_code, diff_add, diff_delete, previous_id
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                (
                    record.id,
                    record.start_time.to_utc().to_rfc3339(),
                    record.stdout,
                    record.stderr,
                    record.end_time.to_utc().to_rfc3339(),
                    record.exit_code,
                    record.diff.map(|(add, _delete)| i64::from(add)),
                    record.diff.map(|(_add, delete)| i64::from(delete)),
                    record.previous_id,
                ),
            )?;
            Ok(())
        } else {
            color_eyre::eyre::bail!("Failed to get connection")
        }
    }

    fn get_record(&self, id: ExecutionId) -> Result<Option<Record>> {
        if let Ok(conn) = self.conn.lock() {
            let r = conn.query_row("SELECT * FROM record WHERE id = ?1", [id], |row| {
                let start_time = row.get::<_, DateTime<Utc>>(1)?;
                let end_time = row.get::<_, DateTime<Utc>>(4)?;
                let diff_add: Option<u32> = row.get(6)?;
                let diff_delete: Option<u32> = row.get(7)?;
                let diff = diff_add.zip(diff_delete);
                Ok(Record {
                    id: row.get(0)?,
                    start_time: start_time.with_timezone(&Local),
                    stdout: row.get(2)?,
                    stderr: row.get(3)?,
                    end_time: end_time.with_timezone(&Local),
                    exit_code: row.get(5)?,
                    diff,
                    previous_id: row.get(8)?,
                })
            });

            match r {
                Ok(record) => Ok(Some(record)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e.into()),
            }
        } else {
            color_eyre::eyre::bail!("Failed to get connection")
        }
    }

    fn get_latest_id(&self) -> Result<Option<ExecutionId>> {
        if let Ok(conn) = self.conn.lock() {
            let r = conn.query_row(
                "SELECT id FROM record ORDER BY id DESC LIMIT 1",
                [],
                |row| row.get(0),
            );

            match r {
                Ok(id) => Ok(Some(id)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e.into()),
            }
        } else {
            color_eyre::eyre::bail!("Failed to get connection")
        }
    }

    fn get_records(&self) -> Result<Vec<Record>> {
        if let Ok(conn) = self.conn.lock() {
            let mut stmt = conn.prepare("SELECT * FROM record")?;
            let records = stmt
                .query_map([], |row| {
                    let start_time = row.get::<_, DateTime<Utc>>(1)?;
                    let end_time = row.get::<_, DateTime<Utc>>(4)?;
                    let diff_add: Option<u32> = row.get(6)?;
                    let diff_delete: Option<u32> = row.get(7)?;
                    let diff = diff_add.zip(diff_delete);
                    Ok(Record {
                        id: row.get(0)?,
                        start_time: start_time.with_timezone(&Local),
                        stdout: row.get(2)?,
                        stderr: row.get(3)?,
                        end_time: end_time.with_timezone(&Local),
                        exit_code: row.get(5)?,
                        diff,
                        previous_id: row.get(8)?,
                    })
                })?
                .collect::<rusqlite::Result<Vec<Record>>>()?;
            Ok(records)
        } else {
            color_eyre::eyre::bail!("Failed to get connection")
        }
    }

    fn get_runtime_config(&self) -> Result<Option<crate::store::RuntimeConfig>> {
        if let Ok(conn) = self.conn.lock() {
            let r = conn.query_row(
                "SELECT * FROM runtime_config ORDER BY ROWID DESC LIMIT 1",
                [],
                |row| {
                    Ok(crate::store::RuntimeConfig {
                        interval: row.get(0)?,
                        command: row.get(1)?,
                    })
                },
            );

            match r {
                Ok(config) => Ok(Some(config)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e.into()),
            }
        } else {
            color_eyre::eyre::bail!("Failed to get connection")
        }
    }

    fn set_runtime_config(&mut self, config: crate::store::RuntimeConfig) -> Result<()> {
        if let Ok(conn) = self.conn.lock() {
            conn.execute(
                "INSERT INTO runtime_config (interval, command) VALUES (?1, ?2)",
                (config.interval, config.command),
            )?;
            Ok(())
        } else {
            color_eyre::eyre::bail!("Failed to get connection")
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone as _;

    use super::*;
    use crate::store::RuntimeConfig;

    fn local_time(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        min: u32,
        sec: u32,
    ) -> DateTime<Local> {
        Utc.with_ymd_and_hms(year, month, day, hour, min, sec)
            .unwrap()
            .with_timezone(&Local)
    }

    fn record() -> Record {
        Record {
            id: ExecutionId(1),
            start_time: local_time(2026, 1, 2, 3, 4, 5),
            stdout: b"stdout".to_vec(),
            stderr: b"stderr".to_vec(),
            end_time: local_time(2026, 1, 2, 3, 4, 6),
            exit_code: 7,
            diff: Some((8, 9)),
            previous_id: Some(ExecutionId(0)),
        }
    }

    #[test]
    fn round_trips_record_and_runtime_config() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let mut store = SQLiteStore::new(temp_dir.path().join("history.db"), true)?;
        let record = record();

        store.add_record(record.clone())?;
        store.set_runtime_config(RuntimeConfig {
            interval: 1000,
            command: "echo hello".to_string(),
        })?;

        let restored = store.get_record(record.id)?.unwrap();
        assert_eq!(restored.id, record.id);
        assert_eq!(restored.start_time, record.start_time);
        assert_eq!(restored.stdout, record.stdout);
        assert_eq!(restored.stderr, record.stderr);
        assert_eq!(restored.end_time, record.end_time);
        assert_eq!(restored.exit_code, record.exit_code);
        assert_eq!(restored.diff, record.diff);
        assert_eq!(restored.previous_id, record.previous_id);
        assert_eq!(store.get_latest_id()?, Some(record.id));

        let runtime_config = store.get_runtime_config()?.unwrap();
        assert_eq!(runtime_config.interval, 1000);
        assert_eq!(runtime_config.command, "echo hello");

        Ok(())
    }

    #[test]
    fn integer_timestamps_are_rejected() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let mut store = SQLiteStore::new(temp_dir.path().join("history.db"), true)?;

        store.add_record(record())?;
        {
            let conn = store.conn.lock().unwrap();
            conn.execute(
                "UPDATE record SET start_time = 0, end_time = 0 WHERE id = ?1",
                [ExecutionId(1)],
            )?;
        }

        assert!(store.get_record(ExecutionId(1)).is_err());

        Ok(())
    }
}
