//! TaiL Core - 数据库访问层

use chrono::{DateTime, Utc};
use rusqlite::params;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::path::Path;

use crate::models::*;

/// 数据库错误类型
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("Connection pool error: {0}")]
    Pool(#[from] r2d2::Error),

    #[error("Database not found: {0}")]
    NotFound(String),
}

/// 数据库配置
#[derive(Debug, Clone)]
pub struct DbConfig {
    pub path: String,
}

impl Default for DbConfig {
    fn default() -> Self {
        let xdg_data = std::env::var("XDG_DATA_HOME")
            .unwrap_or_else(|_| format!("{}/.local/share", std::env::var("HOME").unwrap()));
        let db_path = format!("{}/tail/tail.db", xdg_data);

        // 确保目录存在
        if let Some(parent) = Path::new(&db_path).parent() {
            std::fs::create_dir_all(parent).ok();
        }

        Self { path: db_path }
    }
}

/// 数据库连接池
pub type DbPool = Pool<SqliteConnectionManager>;

/// 数据库仓库
pub struct Repository {
    pool: DbPool,
}

impl Repository {
    /// 创建新的数据库连接
    pub fn new(config: &DbConfig) -> Result<Self, DbError> {
        let manager = SqliteConnectionManager::file(&config.path);
        let pool = Pool::builder()
            .max_size(10)
            .build(manager)?;

        let repo = Self { pool };
        repo.init_schema()?;

        Ok(repo)
    }

    /// 从连接池创建
    pub fn with_pool(pool: DbPool) -> Self {
        Self { pool }
    }

    /// 初始化数据库 schema
    fn init_schema(&self) -> Result<(), DbError> {
        let conn = self.pool.get()?;

        // 窗口事件表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS window_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp DATETIME NOT NULL,
                app_name TEXT NOT NULL,
                window_title TEXT,
                workspace TEXT,
                duration_secs INTEGER NOT NULL DEFAULT 0,
                is_afk BOOLEAN NOT NULL DEFAULT 0
            )",
            [],
        )?;

        // 索引
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_window_events_timestamp
             ON window_events(timestamp)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_window_events_app
             ON window_events(app_name)",
            [],
        )?;

        // AFK 事件表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS afk_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                start_time DATETIME NOT NULL,
                end_time DATETIME,
                duration_secs INTEGER NOT NULL DEFAULT 0
            )",
            [],
        )?;

        // 每日目标表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS daily_goals (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                app_name TEXT NOT NULL UNIQUE,
                max_minutes INTEGER NOT NULL,
                notify_enabled BOOLEAN NOT NULL DEFAULT 1
            )",
            [],
        )?;

        Ok(())
    }

    /// 插入窗口事件
    pub fn insert_window_event(&self, event: &WindowEvent) -> Result<i64, DbError> {
        let conn = self.pool.get()?;

        conn.execute(
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
        )?;

        Ok(conn.last_insert_rowid())
    }

    /// 获取指定时间范围内的窗口事件
    pub fn get_window_events(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<WindowEvent>, DbError> {
        let conn = self.pool.get()?;

        let mut stmt = conn.prepare(
            "SELECT id, timestamp, app_name, window_title, workspace, duration_secs, is_afk
             FROM window_events
             WHERE timestamp >= ?1 AND timestamp <= ?2
             ORDER BY timestamp ASC",
        )?;

        let events = stmt.query_map(params![start, end], |row| {
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

    /// 获取应用使用统计
    pub fn get_app_usage(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<AppUsage>, DbError> {
        let conn = self.pool.get()?;

        let mut stmt = conn.prepare(
            "SELECT app_name, SUM(duration_secs) as total_seconds
             FROM window_events
             WHERE timestamp >= ?1 AND timestamp <= ?2 AND is_afk = 0
             GROUP BY app_name
             ORDER BY total_seconds DESC",
        )?;

        let usages = stmt.query_map(params![start, end], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(usages
            .into_iter()
            .map(|(app_name, total_seconds)| AppUsage {
                app_name,
                total_seconds,
                window_events: vec![],
            })
            .collect())
    }

    /// 更新窗口事件的时长
    pub fn update_window_event_duration(&self, id: i64, duration_secs: i64) -> Result<(), DbError> {
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE window_events SET duration_secs = ?1 WHERE id = ?2",
            params![duration_secs, id],
        )?;
        Ok(())
    }

    /// 插入 AFK 事件
    pub fn insert_afk_event(&self, event: &AfkEvent) -> Result<i64, DbError> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT INTO afk_events (start_time, end_time, duration_secs)
             VALUES (?1, ?2, ?3)",
            params![event.start_time, event.end_time, event.duration_secs],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// 更新 AFK 事件结束时间
    pub fn update_afk_event_end(&self, id: i64, end_time: DateTime<Utc>, duration_secs: i64) -> Result<(), DbError> {
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE afk_events SET end_time = ?1, duration_secs = ?2 WHERE id = ?3",
            params![end_time, duration_secs, id],
        )?;
        Ok(())
    }

    /// 获取指定时间范围内的 AFK 事件
    pub fn get_afk_events(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<AfkEvent>, DbError> {
        let conn = self.pool.get()?;

        let mut stmt = conn.prepare(
            "SELECT id, start_time, end_time, duration_secs
             FROM afk_events
             WHERE start_time >= ?1 AND start_time <= ?2
             ORDER BY start_time ASC",
        )?;

        let events = stmt.query_map(params![start, end], |row| {
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

    /// 插入或更新每日目标
    pub fn upsert_daily_goal(&self, goal: &DailyGoal) -> Result<i64, DbError> {
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

    /// 获取所有每日目标
    pub fn get_daily_goals(&self) -> Result<Vec<DailyGoal>, DbError> {
        let conn = self.pool.get()?;

        let mut stmt = conn.prepare(
            "SELECT id, app_name, max_minutes, notify_enabled
             FROM daily_goals
             ORDER BY app_name ASC",
        )?;

        let goals = stmt.query_map([], |row| {
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

    /// 删除每日目标
    pub fn delete_daily_goal(&self, app_name: &str) -> Result<(), DbError> {
        let conn = self.pool.get()?;
        conn.execute(
            "DELETE FROM daily_goals WHERE app_name = ?1",
            params![app_name],
        )?;
        Ok(())
    }

    /// 获取今日某应用的总使用时长（秒）
    pub fn get_today_app_usage(&self, app_name: &str) -> Result<i64, DbError> {
        let conn = self.pool.get()?;
        
        // 获取今天的开始时间（UTC）
        let today_start = chrono::Utc::now()
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();

        let mut stmt = conn.prepare(
            "SELECT COALESCE(SUM(duration_secs), 0)
             FROM window_events
             WHERE app_name = ?1 AND timestamp >= ?2 AND is_afk = 0",
        )?;

        let total: i64 = stmt.query_row(params![app_name, today_start], |row| row.get(0))?;

        Ok(total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_repo() -> Repository {
        let config = DbConfig {
            path: ":memory:".to_string(),
        };
        Repository::new(&config).unwrap()
    }

    fn create_test_event(app_name: &str, duration: i64) -> WindowEvent {
        WindowEvent {
            id: None,
            timestamp: Utc::now(),
            app_name: app_name.to_string(),
            window_title: "Test Window".to_string(),
            workspace: "1".to_string(),
            duration_secs: duration,
            is_afk: false,
        }
    }

    #[test]
    fn test_insert_window_event() {
        let repo = create_test_repo();
        let event = create_test_event("firefox", 120);

        let id = repo.insert_window_event(&event).unwrap();
        assert!(id > 0);
    }

    #[test]
    fn test_get_window_events() {
        let repo = create_test_repo();
        
        // 插入测试数据
        let event1 = create_test_event("firefox", 120);
        let event2 = create_test_event("kitty", 60);
        
        repo.insert_window_event(&event1).unwrap();
        repo.insert_window_event(&event2).unwrap();

        // 查询
        let start = Utc::now() - chrono::Duration::hours(1);
        let end = Utc::now() + chrono::Duration::hours(1);
        let events = repo.get_window_events(start, end).unwrap();

        assert_eq!(events.len(), 2);
    }

    #[test]
    fn test_get_app_usage() {
        let repo = create_test_repo();
        
        // 插入多个相同应用的事件
        repo.insert_window_event(&create_test_event("firefox", 120)).unwrap();
        repo.insert_window_event(&create_test_event("firefox", 180)).unwrap();
        repo.insert_window_event(&create_test_event("kitty", 60)).unwrap();

        let start = Utc::now() - chrono::Duration::hours(1);
        let end = Utc::now() + chrono::Duration::hours(1);
        let usage = repo.get_app_usage(start, end).unwrap();

        assert_eq!(usage.len(), 2);
        
        // firefox 应该排在第一位（总时长最长）
        assert_eq!(usage[0].app_name, "firefox");
        assert_eq!(usage[0].total_seconds, 300); // 120 + 180
    }

    #[test]
    fn test_update_window_event_duration() {
        let repo = create_test_repo();
        let event = create_test_event("firefox", 0);
        
        let id = repo.insert_window_event(&event).unwrap();
        repo.update_window_event_duration(id, 300).unwrap();

        let start = Utc::now() - chrono::Duration::hours(1);
        let end = Utc::now() + chrono::Duration::hours(1);
        let events = repo.get_window_events(start, end).unwrap();

        assert_eq!(events[0].duration_secs, 300);
    }

    #[test]
    fn test_afk_events() {
        let repo = create_test_repo();
        
        let afk_event = AfkEvent {
            id: None,
            start_time: Utc::now(),
            end_time: None,
            duration_secs: 0,
        };

        let id = repo.insert_afk_event(&afk_event).unwrap();
        assert!(id > 0);

        // 更新结束时间
        let end_time = Utc::now();
        repo.update_afk_event_end(id, end_time, 300).unwrap();

        let start = Utc::now() - chrono::Duration::hours(1);
        let end = Utc::now() + chrono::Duration::hours(1);
        let events = repo.get_afk_events(start, end).unwrap();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].duration_secs, 300);
    }

    #[test]
    fn test_daily_goals() {
        let repo = create_test_repo();
        
        let goal = DailyGoal {
            id: None,
            app_name: "firefox".to_string(),
            max_minutes: 120,
            notify_enabled: true,
        };

        repo.upsert_daily_goal(&goal).unwrap();

        let goals = repo.get_daily_goals().unwrap();
        assert_eq!(goals.len(), 1);
        assert_eq!(goals[0].app_name, "firefox");
        assert_eq!(goals[0].max_minutes, 120);

        // 测试更新
        let updated_goal = DailyGoal {
            id: None,
            app_name: "firefox".to_string(),
            max_minutes: 180,
            notify_enabled: false,
        };
        repo.upsert_daily_goal(&updated_goal).unwrap();

        let goals = repo.get_daily_goals().unwrap();
        assert_eq!(goals.len(), 1); // 应该还是1个，因为是更新
        assert_eq!(goals[0].max_minutes, 180);

        // 测试删除
        repo.delete_daily_goal("firefox").unwrap();
        let goals = repo.get_daily_goals().unwrap();
        assert_eq!(goals.len(), 0);
    }

    #[test]
    fn test_get_today_app_usage() {
        let repo = create_test_repo();
        
        // 插入今天的数据
        repo.insert_window_event(&create_test_event("firefox", 120)).unwrap();
        repo.insert_window_event(&create_test_event("firefox", 180)).unwrap();

        let total = repo.get_today_app_usage("firefox").unwrap();
        assert_eq!(total, 300);

        let total_kitty = repo.get_today_app_usage("kitty").unwrap();
        assert_eq!(total_kitty, 0);
    }
}
