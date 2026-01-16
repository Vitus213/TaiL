//! TaiL Core - 统一错误类型

use rusqlite::Error as SqliteError;
use r2d2::Error as PoolError;

/// 数据库错误类型
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] SqliteError),

    #[error("Connection pool error: {0}")]
    Pool(#[from] PoolError),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Database connection closed")]
    ConnectionClosed,
}

/// DbResult 类型别名
pub type DbResult<T> = Result<T, DbError>;
