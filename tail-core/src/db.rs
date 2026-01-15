//! TaiL Core - 数据库访问层

use chrono::{DateTime, Utc, Local, Datelike};
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

        // 分类表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS categories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                icon TEXT NOT NULL,
                color TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // 应用-分类关联表（多对多关系）
        conn.execute(
            "CREATE TABLE IF NOT EXISTS app_categories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                app_name TEXT NOT NULL,
                category_id INTEGER NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE CASCADE,
                UNIQUE(app_name, category_id)
            )",
            [],
        )?;

        // 索引
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_app_categories_app ON app_categories(app_name)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_app_categories_category ON app_categories(category_id)",
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

        tracing::debug!("get_app_usage 查询范围: {} 到 {}", start, end);

        // 首先获取所有窗口事件
        let mut events_stmt = conn.prepare(
            "SELECT id, timestamp, app_name, window_title, workspace, duration_secs, is_afk
             FROM window_events
             WHERE timestamp >= ?1 AND timestamp <= ?2 AND is_afk = 0
             ORDER BY timestamp ASC",
        )?;

        let all_events: Vec<WindowEvent> = events_stmt.query_map(params![start, end], |row| {
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

        tracing::debug!("get_app_usage 查询到 {} 条事件", all_events.len());

        // 按应用名称分组并计算总时长
        let mut app_map: std::collections::HashMap<String, (i64, Vec<WindowEvent>)> = std::collections::HashMap::new();
        
        for event in all_events {
            let entry = app_map.entry(event.app_name.clone()).or_insert((0, Vec::new()));
            entry.0 += event.duration_secs;
            entry.1.push(event);
        }

        // 调试日志：输出每个应用的统计
        for (app_name, (total, events)) in &app_map {
            tracing::debug!("应用 '{}': 总时长 {} 秒, 事件数 {}", app_name, total, events.len());
        }

        // 转换为 AppUsage 并按总时长排序
        let mut usages: Vec<AppUsage> = app_map
            .into_iter()
            .map(|(app_name, (total_seconds, window_events))| AppUsage {
                app_name,
                total_seconds,
                window_events,
            })
            .collect();

        usages.sort_by(|a, b| b.total_seconds.cmp(&a.total_seconds));

        Ok(usages)
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
        
        // 获取今天的开始时间（使用本地时间计算"今天"，然后转换为 UTC）
        let local_now = Local::now();
        let today_start = local_now.date_naive()
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

    // ==================== 分类相关操作 ====================

    /// 插入新分类
    pub fn insert_category(&self, category: &Category) -> Result<i64, DbError> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT INTO categories (name, icon, color) VALUES (?1, ?2, ?3)",
            params![category.name, category.icon, category.color],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// 更新分类
    pub fn update_category(&self, category: &Category) -> Result<(), DbError> {
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE categories SET name = ?1, icon = ?2, color = ?3 WHERE id = ?4",
            params![category.name, category.icon, category.color, category.id],
        )?;
        Ok(())
    }

    /// 删除分类
    pub fn delete_category(&self, id: i64) -> Result<(), DbError> {
        let conn = self.pool.get()?;
        conn.execute("DELETE FROM categories WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// 获取所有分类
    pub fn get_categories(&self) -> Result<Vec<Category>, DbError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, icon, color FROM categories ORDER BY name ASC",
        )?;

        let categories = stmt.query_map([], |row| {
            Ok(Category {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                icon: row.get(2)?,
                color: row.get(3)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(categories)
    }

    /// 根据 ID 获取分类
    pub fn get_category_by_id(&self, id: i64) -> Result<Option<Category>, DbError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, icon, color FROM categories WHERE id = ?1",
        )?;

        let result = stmt.query_row(params![id], |row| {
            Ok(Category {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                icon: row.get(2)?,
                color: row.get(3)?,
            })
        });

        match result {
            Ok(category) => Ok(Some(category)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DbError::Sqlite(e)),
        }
    }

    /// 将应用添加到分类
    pub fn add_app_to_category(&self, app_name: &str, category_id: i64) -> Result<(), DbError> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT OR IGNORE INTO app_categories (app_name, category_id) VALUES (?1, ?2)",
            params![app_name, category_id],
        )?;
        Ok(())
    }

    /// 从分类中移除应用
    pub fn remove_app_from_category(&self, app_name: &str, category_id: i64) -> Result<(), DbError> {
        let conn = self.pool.get()?;
        conn.execute(
            "DELETE FROM app_categories WHERE app_name = ?1 AND category_id = ?2",
            params![app_name, category_id],
        )?;
        Ok(())
    }

    /// 获取应用所属的所有分类
    pub fn get_app_categories(&self, app_name: &str) -> Result<Vec<Category>, DbError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT c.id, c.name, c.icon, c.color
             FROM categories c
             INNER JOIN app_categories ac ON c.id = ac.category_id
             WHERE ac.app_name = ?1
             ORDER BY c.name ASC",
        )?;

        let categories = stmt.query_map(params![app_name], |row| {
            Ok(Category {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                icon: row.get(2)?,
                color: row.get(3)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(categories)
    }

    /// 获取分类下的所有应用名称
    pub fn get_category_apps(&self, category_id: i64) -> Result<Vec<String>, DbError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT app_name FROM app_categories WHERE category_id = ?1 ORDER BY app_name ASC",
        )?;

        let apps = stmt.query_map(params![category_id], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(apps)
    }

    /// 获取分类使用统计
    pub fn get_category_usage(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<CategoryUsage>, DbError> {
        let conn = self.pool.get()?;

        // 获取所有分类
        let categories = self.get_categories()?;
        
        let mut result = Vec::new();

        for category in categories {
            let category_id = category.id.unwrap();
            
            // 获取该分类下的所有应用
            let apps = self.get_category_apps(category_id)?;
            
            if apps.is_empty() {
                result.push(CategoryUsage {
                    category,
                    total_seconds: 0,
                    app_count: 0,
                    apps: Vec::new(),
                });
                continue;
            }

            // 构建 IN 子句的占位符
            let placeholders: Vec<String> = apps.iter().enumerate()
                .map(|(i, _)| format!("?{}", i + 3))
                .collect();
            let in_clause = placeholders.join(", ");

            // 查询该分类下所有应用的使用时间
            let query = format!(
                "SELECT app_name, COALESCE(SUM(duration_secs), 0) as total
                 FROM window_events
                 WHERE timestamp >= ?1 AND timestamp <= ?2
                   AND is_afk = 0
                   AND app_name IN ({})
                 GROUP BY app_name
                 ORDER BY total DESC",
                in_clause
            );

            let mut stmt = conn.prepare(&query)?;
            
            // 构建参数
            let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
            params_vec.push(Box::new(start));
            params_vec.push(Box::new(end));
            for app in &apps {
                params_vec.push(Box::new(app.clone()));
            }
            
            let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter()
                .map(|p| p.as_ref())
                .collect();

            let app_usages_with_time: Vec<AppUsageInCategory> = stmt.query_map(params_refs.as_slice(), |row| {
                Ok(AppUsageInCategory {
                    app_name: row.get(0)?,
                    total_seconds: row.get(1)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

            // 创建一个包含所有分类应用的列表，包括没有使用记录的应用
            let mut all_app_usages: Vec<AppUsageInCategory> = Vec::new();
            let apps_with_time: std::collections::HashSet<String> = app_usages_with_time.iter()
                .map(|a| a.app_name.clone())
                .collect();
            
            // 先添加有使用记录的应用
            all_app_usages.extend(app_usages_with_time);
            
            // 再添加没有使用记录的应用（时间为0）
            for app_name in &apps {
                if !apps_with_time.contains(app_name) {
                    all_app_usages.push(AppUsageInCategory {
                        app_name: app_name.clone(),
                        total_seconds: 0,
                    });
                }
            }

            let total_seconds: i64 = all_app_usages.iter().map(|a| a.total_seconds).sum();

            result.push(CategoryUsage {
                category,
                total_seconds,
                app_count: all_app_usages.len(),
                apps: all_app_usages,
            });
        }

        // 按总时长排序
        result.sort_by(|a, b| b.total_seconds.cmp(&a.total_seconds));

        Ok(result)
    }

    /// 设置应用的分类（替换所有现有分类）
    pub fn set_app_categories(&self, app_name: &str, category_ids: &[i64]) -> Result<(), DbError> {
        let conn = self.pool.get()?;
        
        // 先删除该应用的所有分类关联
        conn.execute(
            "DELETE FROM app_categories WHERE app_name = ?1",
            params![app_name],
        )?;
        
        // 添加新的分类关联
        for category_id in category_ids {
            conn.execute(
                "INSERT INTO app_categories (app_name, category_id) VALUES (?1, ?2)",
                params![app_name, category_id],
            )?;
        }
        
        Ok(())
    }

    /// 获取所有已记录的应用名称（用于分类管理）
    pub fn get_all_app_names(&self) -> Result<Vec<String>, DbError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT DISTINCT app_name FROM window_events ORDER BY app_name ASC",
        )?;

        let apps = stmt.query_map([], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(apps)
    }

    // ==================== 层级式时间统计查询 ====================

    /// 获取按年份汇总的使用统计（显示最近 N 年）
    pub fn get_yearly_usage(&self, years: i32) -> Result<Vec<PeriodUsage>, DbError> {
        let conn = self.pool.get()?;
        let current_year = Local::now().year();
        
        let mut result = Vec::new();
        
        for i in 0..years {
            let year = current_year - i;
            let year_start = chrono::NaiveDate::from_ymd_opt(year, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_local_timezone(Local)
                .unwrap()
                .with_timezone(&Utc);
            let year_end = chrono::NaiveDate::from_ymd_opt(year, 12, 31)
                .unwrap()
                .and_hms_opt(23, 59, 59)
                .unwrap()
                .and_local_timezone(Local)
                .unwrap()
                .with_timezone(&Utc);
            
            let mut stmt = conn.prepare(
                "SELECT COALESCE(SUM(duration_secs), 0)
                 FROM window_events
                 WHERE timestamp >= ?1 AND timestamp <= ?2 AND is_afk = 0",
            )?;
            
            let total: i64 = stmt.query_row(params![year_start, year_end], |row| row.get(0))?;
            
            result.push(PeriodUsage {
                label: format!("{}年", year),
                index: year,
                total_seconds: total,
            });
        }
        
        // 反转使其从旧到新排列
        result.reverse();
        
        Ok(result)
    }

    /// 获取某年按月份汇总的使用统计
    pub fn get_monthly_usage(&self, year: i32) -> Result<Vec<PeriodUsage>, DbError> {
        use chrono::Datelike;
        
        let conn = self.pool.get()?;
        let mut result = Vec::new();
        
        for month in 1..=12 {
            let month_start = chrono::NaiveDate::from_ymd_opt(year, month, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_local_timezone(Local)
                .unwrap()
                .with_timezone(&Utc);
            
            // 计算月末
            let next_month = if month == 12 {
                chrono::NaiveDate::from_ymd_opt(year + 1, 1, 1)
            } else {
                chrono::NaiveDate::from_ymd_opt(year, month + 1, 1)
            }.unwrap();
            let last_day = next_month.pred_opt().unwrap();
            
            let month_end = last_day
                .and_hms_opt(23, 59, 59)
                .unwrap()
                .and_local_timezone(Local)
                .unwrap()
                .with_timezone(&Utc);
            
            let mut stmt = conn.prepare(
                "SELECT COALESCE(SUM(duration_secs), 0)
                 FROM window_events
                 WHERE timestamp >= ?1 AND timestamp <= ?2 AND is_afk = 0",
            )?;
            
            let total: i64 = stmt.query_row(params![month_start, month_end], |row| row.get(0))?;
            
            result.push(PeriodUsage {
                label: format!("{}月", month),
                index: month as i32,
                total_seconds: total,
            });
        }
        
        Ok(result)
    }

    /// 获取某年某月按周汇总的使用统计
    pub fn get_weekly_usage(&self, year: i32, month: u32) -> Result<Vec<PeriodUsage>, DbError> {
        use chrono::Datelike;
        
        let conn = self.pool.get()?;
        let mut result = Vec::new();
        
        // 获取该月的第一天和最后一天
        let first_day = chrono::NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        let next_month = if month == 12 {
            chrono::NaiveDate::from_ymd_opt(year + 1, 1, 1)
        } else {
            chrono::NaiveDate::from_ymd_opt(year, month + 1, 1)
        }.unwrap();
        let last_day = next_month.pred_opt().unwrap();
        
        // 按周分组
        let mut week_num = 1;
        let mut current_day = first_day;
        
        while current_day <= last_day {
            // 计算本周的开始和结束
            let week_start = current_day;
            let mut week_end = current_day;
            
            // 找到本周的最后一天（周日或月末）
            while week_end.weekday() != chrono::Weekday::Sun && week_end < last_day {
                week_end = week_end.succ_opt().unwrap();
            }
            
            let start_dt = week_start
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_local_timezone(Local)
                .unwrap()
                .with_timezone(&Utc);
            let end_dt = week_end
                .and_hms_opt(23, 59, 59)
                .unwrap()
                .and_local_timezone(Local)
                .unwrap()
                .with_timezone(&Utc);
            
            let mut stmt = conn.prepare(
                "SELECT COALESCE(SUM(duration_secs), 0)
                 FROM window_events
                 WHERE timestamp >= ?1 AND timestamp <= ?2 AND is_afk = 0",
            )?;
            
            let total: i64 = stmt.query_row(params![start_dt, end_dt], |row| row.get(0))?;
            
            result.push(PeriodUsage {
                label: format!("第{}周", week_num),
                index: week_num,
                total_seconds: total,
            });
            
            // 移动到下一周
            current_day = week_end.succ_opt().unwrap();
            week_num += 1;
        }
        
        Ok(result)
    }

    /// 获取某年某月某周按天汇总的使用统计
    pub fn get_daily_usage_for_week(&self, year: i32, month: u32, week: u32) -> Result<Vec<PeriodUsage>, DbError> {
        use chrono::Datelike;
        
        let conn = self.pool.get()?;
        let mut result = Vec::new();
        
        // 获取该月的第一天
        let first_day = chrono::NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        let next_month = if month == 12 {
            chrono::NaiveDate::from_ymd_opt(year + 1, 1, 1)
        } else {
            chrono::NaiveDate::from_ymd_opt(year, month + 1, 1)
        }.unwrap();
        let last_day = next_month.pred_opt().unwrap();
        
        // 找到指定周的开始日期
        let mut current_day = first_day;
        let mut current_week = 1u32;
        
        while current_week < week && current_day <= last_day {
            // 跳到下一周
            while current_day.weekday() != chrono::Weekday::Sun && current_day < last_day {
                current_day = current_day.succ_opt().unwrap();
            }
            current_day = current_day.succ_opt().unwrap_or(current_day);
            current_week += 1;
        }
        
        // 收集该周的每一天
        let week_start = current_day;
        let mut day = week_start;
        
        let weekday_names = ["周一", "周二", "周三", "周四", "周五", "周六", "周日"];
        
        loop {
            if day > last_day {
                break;
            }
            
            let start_dt = day
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_local_timezone(Local)
                .unwrap()
                .with_timezone(&Utc);
            let end_dt = day
                .and_hms_opt(23, 59, 59)
                .unwrap()
                .and_local_timezone(Local)
                .unwrap()
                .with_timezone(&Utc);
            
            let mut stmt = conn.prepare(
                "SELECT COALESCE(SUM(duration_secs), 0)
                 FROM window_events
                 WHERE timestamp >= ?1 AND timestamp <= ?2 AND is_afk = 0",
            )?;
            
            let total: i64 = stmt.query_row(params![start_dt, end_dt], |row| row.get(0))?;
            
            let weekday_idx = day.weekday().num_days_from_monday() as usize;
            result.push(PeriodUsage {
                label: weekday_names[weekday_idx].to_string(),
                index: day.day() as i32,
                total_seconds: total,
            });
            
            // 如果是周日或月末，结束
            if day.weekday() == chrono::Weekday::Sun || day >= last_day {
                break;
            }
            
            day = day.succ_opt().unwrap();
        }
        
        Ok(result)
    }

    /// 获取某天按小时汇总的使用统计
    pub fn get_hourly_usage(&self, year: i32, month: u32, day: u32) -> Result<Vec<PeriodUsage>, DbError> {
        let conn = self.pool.get()?;
        let mut result = Vec::new();
        
        let date = chrono::NaiveDate::from_ymd_opt(year, month, day).unwrap();
        
        for hour in 0..24 {
            let hour_start = date
                .and_hms_opt(hour, 0, 0)
                .unwrap()
                .and_local_timezone(Local)
                .unwrap()
                .with_timezone(&Utc);
            let hour_end = date
                .and_hms_opt(hour, 59, 59)
                .unwrap()
                .and_local_timezone(Local)
                .unwrap()
                .with_timezone(&Utc);
            
            let mut stmt = conn.prepare(
                "SELECT COALESCE(SUM(duration_secs), 0)
                 FROM window_events
                 WHERE timestamp >= ?1 AND timestamp <= ?2 AND is_afk = 0",
            )?;
            
            let total: i64 = stmt.query_row(params![hour_start, hour_end], |row| row.get(0))?;
            
            result.push(PeriodUsage {
                label: format!("{}时", hour),
                index: hour as i32,
                total_seconds: total,
            });
        }
        
        Ok(result)
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
