//! AFK 事件仓储实现

use crate::errors::{DbError, DbResult};
use crate::models::AfkEvent;
use crate::traits::AfkEventRepository;
use crate::db::pool::DbPool;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rusqlite::params;

/// AFK 事件仓储实现
pub struct AfkEventRepositoryImpl {
    pool: DbPool,
}

impl AfkEventRepositoryImpl {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    fn insert_sync(&self, event: &AfkEvent) -> DbResult<i64> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT INTO afk_events (start_time, end_time, duration_secs)
             VALUES (?1, ?2, ?3)",
            params![event.start_time, event.end_time, event.duration_secs],
        )?;
        Ok(conn.last_insert_rowid())
    }

    fn update_end_sync(
        &self,
        id: i64,
        end_time: DateTime<Utc>,
        duration_secs: i64,
    ) -> DbResult<()> {
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE afk_events SET end_time = ?1, duration_secs = ?2 WHERE id = ?3",
            params![end_time, duration_secs, id],
        )?;
        Ok(())
    }

    fn get_by_time_range_sync(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> DbResult<Vec<AfkEvent>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, start_time, end_time, duration_secs
             FROM afk_events
             WHERE start_time >= ?1 AND start_time <= ?2
             ORDER BY start_time ASC",
        )?;

        let events = stmt
            .query_map(params![start, end], |row| {
                Ok(AfkEvent {
                    id: Some(row.get(0)?),
                    start_time: row.get(1)?,
                    end_time: row.get(2)?,
                    duration_secs: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(events)
    }
}

#[async_trait]
impl AfkEventRepository for AfkEventRepositoryImpl {
    async fn insert(&self, event: &AfkEvent) -> DbResult<i64> {
        let repo = self.clone();
        let event = event.clone();
        tokio::task::spawn_blocking(move || repo.insert_sync(&event))
            .await
            .map_err(|e| DbError::Validation(format!("Task join error: {}", e)))?
    }

    async fn update_end(
        &self,
        id: i64,
        end_time: DateTime<Utc>,
        duration_secs: i64,
    ) -> DbResult<()> {
        let repo = self.clone();
        tokio::task::spawn_blocking(move || repo.update_end_sync(id, end_time, duration_secs))
            .await
            .map_err(|e| DbError::Validation(format!("Task join error: {}", e)))?
    }

    async fn get_by_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> DbResult<Vec<AfkEvent>> {
        let repo = self.clone();
        tokio::task::spawn_blocking(move || repo.get_by_time_range_sync(start, end))
            .await
            .map_err(|e| DbError::Validation(format!("Task join error: {}", e)))?
    }
}

impl Clone for AfkEventRepositoryImpl {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
        }
    }
}
