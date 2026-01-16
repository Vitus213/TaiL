//! 别名仓储实现

use crate::errors::{DbError, DbResult};
use crate::db::pool::DbPool;
use crate::traits::AliasRepository;
use async_trait::async_trait;
use rusqlite::params;
use std::sync::Arc;

/// 别名仓储实现
pub struct AliasRepositoryImpl {
    pool: Arc<DbPool>,
}

impl AliasRepositoryImpl {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }

    fn set_sync(&self, app_name: &str, alias: &str) -> DbResult<()> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT OR REPLACE INTO app_aliases (app_name, alias) VALUES (?1, ?2)",
            params![app_name, alias],
        )?;
        Ok(())
    }

    fn get_sync(&self, app_name: &str) -> DbResult<Option<String>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT alias FROM app_aliases WHERE app_name = ?1")?;

        let result = stmt.query_row(params![app_name], |row| row.get(0))?;
        Ok(Some(result))
    }

    fn get_all_sync(&self) -> DbResult<Vec<(String, String)>> {
        let conn = self.pool.get()?;
        let mut stmt =
            conn.prepare("SELECT app_name, alias FROM app_aliases ORDER BY app_name ASC")?;

        let aliases = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(aliases)
    }

    fn delete_sync(&self, app_name: &str) -> DbResult<()> {
        let conn = self.pool.get()?;
        conn.execute(
            "DELETE FROM app_aliases WHERE app_name = ?1",
            params![app_name],
        )?;
        Ok(())
    }
}

#[async_trait]
impl AliasRepository for AliasRepositoryImpl {
    async fn set(&self, app_name: &str, alias: &str) -> DbResult<()> {
        let repo = self.clone();
        let app_name = app_name.to_string();
        let alias = alias.to_string();
        tokio::task::spawn_blocking(move || repo.set_sync(&app_name, &alias))
            .await
            .map_err(|e| DbError::Validation(format!("Task join error: {}", e)))?
    }

    async fn get(&self, app_name: &str) -> DbResult<Option<String>> {
        let repo = self.clone();
        let app_name = app_name.to_string();
        tokio::task::spawn_blocking(move || repo.get_sync(&app_name))
            .await
            .map_err(|e| DbError::Validation(format!("Task join error: {}", e)))?
    }

    async fn get_all(&self) -> DbResult<Vec<(String, String)>> {
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
}

impl Clone for AliasRepositoryImpl {
    fn clone(&self) -> Self {
        Self {
            pool: Arc::clone(&self.pool),
        }
    }
}
