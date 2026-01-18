//! SQLite 事件仓储适配器
//!
//! 实现 `EventRepositoryPort` 接口

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rusqlite::params;
use std::sync::Arc;

use crate::application::ports::{EventRepositoryPort, RepoError};
use crate::domain::TimeEvent;

/// SQLite 事件仓储
pub struct SqliteEventRepository {
    pool: Arc<crate::db::DbPool>,
}

impl SqliteEventRepository {
    pub fn new(pool: Arc<crate::db::DbPool>) -> Self {
        Self { pool }
    }

    pub async fn from_config(config: &crate::db::Config) -> Result<Self, RepoError> {
        let pool = crate::db::create_pool(config)?;
        Ok(Self { pool: Arc::new(pool) })
    }

    pub async fn from_path(path: &str) -> Result<Self, RepoError> {
        let config = crate::db::Config {
            path: path.to_string(),
            ..Default::default()
        };
        Self::from_config(&config).await
    }

    pub async fn in_memory() -> Result<Self, RepoError> {
        let pool = crate::db::create_pool(&crate::db::Config {
            path: ":memory:".to_string(),
        })?;

        crate::db::init_schema(&pool)?;

        Ok(Self { pool: Arc::new(pool) })
    }
}

#[async_trait]
impl EventRepositoryPort for SqliteEventRepository {
    async fn save(&self, event: &TimeEvent) -> Result<(), RepoError> {
        let pool = self.pool.clone();
        let timestamp = event.timestamp;
        let app_name = event.app_name.as_str().to_string();
        let duration_secs = event.duration_secs();
        let window_title = event.window_title.clone();
        let workspace = event.workspace.clone();
        let is_afk = event.is_afk;

        tokio::task::spawn_blocking(move || {
            let conn = pool.get()?;

            // 简单插入，如果重复会忽略（因为 timestamp + app_name 组合可能重复）
            conn.execute(
                "INSERT INTO window_events (timestamp, app_name, window_title, workspace, duration_secs, is_afk)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![timestamp, app_name, window_title, workspace, duration_secs, is_afk],
            )?;

            Ok::<(), RepoError>(())
        })
        .await
        .map_err(|e| RepoError::ConnectionFailed(e.to_string()))?
    }

    async fn save_batch(&self, events: &[TimeEvent]) -> Result<(), RepoError> {
        let pool = self.pool.clone();
        let events: Vec<(DateTime<Utc>, String, i64, bool)> = events
            .iter()
            .map(|e| (e.timestamp, e.app_name.as_str().to_string(), e.duration_secs(), e.is_afk))
            .collect();

        tokio::task::spawn_blocking(move || {
            let conn = pool.get()?;
            let tx = conn.unchecked_transaction()?;

            for (timestamp, app_name, duration_secs, is_afk) in &events {
                tx.execute(
                    "INSERT INTO window_events (timestamp, app_name, duration_secs, is_afk)
                     VALUES (?1, ?2, ?3, ?4)
                     ON CONFLICT(timestamp) DO UPDATE SET
                     duration_secs = duration_secs + excluded.duration_secs",
                    params![timestamp, app_name, duration_secs, is_afk],
                )?;
            }

            tx.commit()?;
            Ok::<(), RepoError>(())
        })
        .await
        .map_err(|e| RepoError::ConnectionFailed(e.to_string()))?
    }

    async fn find_by_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<TimeEvent>, RepoError> {
        let pool = self.pool.clone();

        tokio::task::spawn_blocking(move || {
            let conn = pool.get()?;

            let mut stmt = conn.prepare_cached(
                "SELECT timestamp, app_name, duration_secs, is_afk, window_title, workspace
                 FROM window_events
                 WHERE timestamp >= ?1 AND timestamp <= ?2
                 ORDER BY timestamp",
            )?;

            let rows = stmt.query_map(params![start, end], |row| {
                Ok((
                    row.get::<_, DateTime<Utc>>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, bool>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, Option<String>>(5)?,
                ))
            })?;

            let mut events = Vec::new();
            for row in rows {
                let (timestamp, app_name, duration_secs, is_afk, window_title, workspace) = row?;

                if is_afk {
                    events.push(TimeEvent::afk(timestamp, duration_secs)?);
                } else {
                    let mut evt = TimeEvent::new(timestamp, app_name, duration_secs)?;
                    if let Some(title) = window_title {
                        evt = evt.with_window_title(title);
                    }
                    if let Some(ws) = workspace {
                        evt = evt.with_workspace(ws);
                    }
                    events.push(evt);
                }
            }

            Ok(events)
        })
        .await
        .map_err(|e| RepoError::ConnectionFailed(e.to_string()))?
    }

    async fn find_by_app(
        &self,
        app_name: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<TimeEvent>, RepoError> {
        let pool = self.pool.clone();
        let app_name = app_name.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = pool.get()?;

            let mut stmt = conn.prepare_cached(
                "SELECT timestamp, app_name, duration_secs, is_afk, window_title, workspace
                 FROM window_events
                 WHERE app_name = ?1 AND timestamp >= ?2 AND timestamp <= ?3
                 ORDER BY timestamp",
            )?;

            let rows = stmt.query_map(params![app_name, start, end], |row| {
                Ok((
                    row.get::<_, DateTime<Utc>>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, bool>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, Option<String>>(5)?,
                ))
            })?;

            let mut events = Vec::new();
            for row in rows {
                let (timestamp, _app_name, duration_secs, is_afk, window_title, workspace) = row?;

                if is_afk {
                    events.push(TimeEvent::afk(timestamp, duration_secs)?);
                } else {
                    let mut evt = TimeEvent::new(timestamp, app_name.clone(), duration_secs)?;
                    if let Some(title) = window_title {
                        evt = evt.with_window_title(title);
                    }
                    if let Some(ws) = workspace {
                        evt = evt.with_workspace(ws);
                    }
                    events.push(evt);
                }
            }

            Ok(events)
        })
        .await
        .map_err(|e| RepoError::ConnectionFailed(e.to_string()))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sqlite_repo() {
        let repo = SqliteEventRepository::in_memory().await.unwrap();
        let event = TimeEvent::new(Utc::now(), "test", 100).unwrap();
        repo.save(&event).await.unwrap();
    }
}
