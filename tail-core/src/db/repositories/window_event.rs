//! 窗口事件仓储实现

use crate::errors::{DbError, DbResult};
use crate::models::WindowEvent;
use crate::traits::WindowEventRepository;
use crate::db::pool::DbPool;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rusqlite::params;
use tracing::{debug, error};

/// 窗口事件仓储实现
pub struct WindowEventRepositoryImpl {
    pool: DbPool,
}

impl WindowEventRepositoryImpl {
    /// 创建新的仓储实例
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// 插入窗口事件（同步方法，供内部使用）
    fn insert_sync(&self, event: &WindowEvent) -> DbResult<i64> {
        let conn = self.pool.get()?;

        debug!(
            app_name = %event.app_name,
            window_title = %event.window_title,
            duration_secs = event.duration_secs,
            is_afk = event.is_afk,
            "插入窗口事件"
        );

        match conn.execute(
            "INSERT INTO window_events (timestamp, app_name, window_title, workspace, duration_secs, is_afk)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                event.timestamp,
                event.app_name,
                event.window_title,
                event.workspace,
                event.duration_secs,
                event.is_afk,
            ],
        ) {
            Ok(_) => {
                let id = conn.last_insert_rowid();
                debug!(event_id = id, "窗口事件插入成功");
                Ok(id)
            }
            Err(e) => {
                error!(
                    error = %e,
                    app_name = %event.app_name,
                    timestamp = %event.timestamp,
                    "插入窗口事件失败"
                );
                Err(DbError::from(e))
            }
        }
    }

    /// 获取时间范围内的窗口事件（同步方法，供内部使用）
    pub fn get_by_time_range_sync(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> DbResult<Vec<WindowEvent>> {
        let conn = self.pool.get()?;

        let mut stmt = conn.prepare(
            "SELECT id, timestamp, app_name, window_title, workspace, duration_secs, is_afk
             FROM window_events
             WHERE timestamp >= ?1 AND timestamp <= ?2
             ORDER BY timestamp ASC",
        )?;

        let events = stmt
            .query_map(params![start, end], |row| {
                Ok(WindowEvent {
                    id: Some(row.get(0)?),
                    timestamp: row.get(1)?,
                    app_name: row.get(2)?,
                    window_title: row.get(3)?,
                    workspace: row.get(4)?,
                    duration_secs: row.get(5)?,
                    is_afk: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(events)
    }

    /// 更新窗口事件时长（同步方法，供内部使用）
    fn update_duration_sync(&self, id: i64, duration_secs: i64) -> DbResult<()> {
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE window_events SET duration_secs = ?1 WHERE id = ?2",
            params![duration_secs, id],
        )?;
        Ok(())
    }
}

#[async_trait]
impl WindowEventRepository for WindowEventRepositoryImpl {
    async fn insert(&self, event: &WindowEvent) -> DbResult<i64> {
        let repo = self.clone();
        // 使用 tokio task pool 在异步上下文中执行同步代码
        let event = event.clone();
        tokio::task::spawn_blocking(move || repo.insert_sync(&event))
            .await
            .map_err(|e| DbError::Validation(format!("Task join error: {}", e)))?
    }

    async fn get_by_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> DbResult<Vec<WindowEvent>> {
        let repo = self.clone();
        tokio::task::spawn_blocking(move || repo.get_by_time_range_sync(start, end))
            .await
            .map_err(|e| DbError::Validation(format!("Task join error: {}", e)))?
    }

    async fn update_duration(&self, id: i64, duration_secs: i64) -> DbResult<()> {
        let repo = self.clone();
        tokio::task::spawn_blocking(move || repo.update_duration_sync(id, duration_secs))
            .await
            .map_err(|e| DbError::Validation(format!("Task join error: {}", e)))?
    }
}

impl Clone for WindowEventRepositoryImpl {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
        }
    }
}
