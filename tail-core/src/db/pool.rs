//! 数据库连接池管理

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use tracing::{error, info};

use crate::errors::{DbError, DbResult};

/// 数据库连接池类型
pub type DbPool = Pool<SqliteConnectionManager>;

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
        if let Some(parent) = std::path::Path::new(&db_path).parent() {
            std::fs::create_dir_all(parent).ok();
        }

        Self { path: db_path }
    }
}

/// 创建数据库连接池
pub fn create_pool(config: &DbConfig) -> DbResult<DbPool> {
    info!("正在初始化数据库连接池，路径: {}", config.path);

    let manager = SqliteConnectionManager::file(&config.path);
    let pool = Pool::builder().max_size(10).build(manager)?;

    info!("数据库连接池创建成功");
    Ok(pool)
}

/// 初始化数据库 schema
pub fn init_schema(pool: &DbPool) -> DbResult<()> {
    let conn = pool.get()?;

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

    // 应用-分类关联表
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

    // 应用别名表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS app_aliases (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            app_name TEXT NOT NULL UNIQUE,
            alias TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
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

    info!("数据库 schema 初始化完成");
    Ok(())
}
