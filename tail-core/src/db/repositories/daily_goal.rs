//! 每日目标仓储实现

use crate::errors::{DbError, DbResult};
use crate::models::DailyGoal;
use crate::traits::DailyGoalRepository;
use crate::db::pool::DbPool;
use async_trait::async_trait;
use chrono::{Local, Utc};
use rusqlite::params;

/// 每日目标仓储实现
pub struct DailyGoalRepositoryImpl {
    pool: DbPool,
}

impl DailyGoalRepositoryImpl {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    fn upsert_sync(&self, goal: &DailyGoal) -> DbResult<i64> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT INTO daily_goals (app_name, max_minutes, notify_enabled)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(app_name) DO UPDATE SET
                max_minutes = excluded.max_minutes,
                notify_enabled = excluded.notify_enabled",
            params![goal.app_name, goal.max_minutes, goal.notify_enabled],
        )?;
        Ok(conn.last_insert_rowid())
    }

    fn get_all_sync(&self) -> DbResult<Vec<DailyGoal>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, app_name, max_minutes, notify_enabled
             FROM daily_goals
             ORDER BY app_name ASC",
        )?;

        let goals = stmt
            .query_map([], |row| {
                Ok(DailyGoal {
                    id: Some(row.get(0)?),
                    app_name: row.get(1)?,
                    max_minutes: row.get(2)?,
                    notify_enabled: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(goals)
    }

    fn delete_sync(&self, app_name: &str) -> DbResult<()> {
        let conn = self.pool.get()?;
        conn.execute(
            "DELETE FROM daily_goals WHERE app_name = ?1",
            params![app_name],
        )?;
        Ok(())
    }

    fn get_today_usage_sync(&self, app_name: &str) -> DbResult<i64> {
        let conn = self.pool.get()?;

        // 获取今天的开始时间（使用本地时间计算"今天"，然后转换为 UTC）
        let local_now = Local::now();
        let today_start = local_now
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .with_timezone(&Utc);

        let mut stmt = conn.prepare(
            "SELECT COALESCE(SUM(duration_secs), 0)
             FROM window_events
             WHERE app_name = ?1 AND timestamp >= ?2 AND is_afk = 0",
        )?;

        let total: i64 = stmt.query_row(params![app_name, today_start], |row| row.get(0))?;

        Ok(total)
    }
}

#[async_trait]
impl DailyGoalRepository for DailyGoalRepositoryImpl {
    async fn upsert(&self, goal: &DailyGoal) -> DbResult<i64> {
        let repo = self.clone();
        let goal = goal.clone();
        tokio::task::spawn_blocking(move || repo.upsert_sync(&goal))
            .await
            .map_err(|e| DbError::Validation(format!("Task join error: {}", e)))?
    }

    async fn get_all(&self) -> DbResult<Vec<DailyGoal>> {
        let repo = self.clone();
        tokio::task::spawn_blocking(move || repo.get_all_sync())
            .await
            .map_err(|e| DbError::Validation(format!("Task join error: {}", e)))?
    }

    async fn delete(&self, app_name: &str) -> DbResult<()> {
        let repo = self.clone();
        let app_name = app_name.to_string();
        tokio::task::spawn_blocking(move || repo.delete_sync(&app_name))
            .await
            .map_err(|e| DbError::Validation(format!("Task join error: {}", e)))?
    }

    async fn get_today_usage(&self, app_name: &str) -> DbResult<i64> {
        let repo = self.clone();
        let app_name = app_name.to_string();
        tokio::task::spawn_blocking(move || repo.get_today_usage_sync(&app_name))
            .await
            .map_err(|e| DbError::Validation(format!("Task join error: {}", e)))?
    }
}

impl Clone for DailyGoalRepositoryImpl {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
        }
    }
}
