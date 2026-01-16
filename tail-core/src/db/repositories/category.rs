//! 分类仓储实现

use crate::errors::{DbError, DbResult};
use crate::models::{AppUsageInCategory, Category, CategoryUsage};
use crate::traits::CategoryRepository;
use crate::db::pool::DbPool;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rusqlite::params;
use std::sync::Arc;

/// 分类仓储实现
pub struct CategoryRepositoryImpl {
    pool: Arc<DbPool>,
}

impl CategoryRepositoryImpl {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }

    fn insert_sync(&self, category: &Category) -> DbResult<i64> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT INTO categories (name, icon, color) VALUES (?1, ?2, ?3)",
            params![category.name, category.icon, category.color],
        )?;
        Ok(conn.last_insert_rowid())
    }

    fn update_sync(&self, category: &Category) -> DbResult<()> {
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE categories SET name = ?1, icon = ?2, color = ?3 WHERE id = ?4",
            params![category.name, category.icon, category.color, category.id],
        )?;
        Ok(())
    }

    fn delete_sync(&self, id: i64) -> DbResult<()> {
        let conn = self.pool.get()?;
        conn.execute("DELETE FROM categories WHERE id = ?1", params![id])?;
        Ok(())
    }

    fn get_all_sync(&self) -> DbResult<Vec<Category>> {
        let conn = self.pool.get()?;
        let mut stmt =
            conn.prepare("SELECT id, name, icon, color FROM categories ORDER BY name ASC")?;

        let categories = stmt
            .query_map([], |row| {
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

    fn get_by_id_sync(&self, id: i64) -> DbResult<Option<Category>> {
        let conn = self.pool.get()?;
        let mut stmt =
            conn.prepare("SELECT id, name, icon, color FROM categories WHERE id = ?1")?;

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
            Err(e) => Err(DbError::from(e)),
        }
    }

    fn get_app_categories_sync(&self, app_name: &str) -> DbResult<Vec<Category>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT c.id, c.name, c.icon, c.color
             FROM categories c
             INNER JOIN app_categories ac ON c.id = ac.category_id
             WHERE ac.app_name = ?1
             ORDER BY c.name ASC",
        )?;

        let categories = stmt
            .query_map(params![app_name], |row| {
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

    fn get_category_apps_sync(&self, category_id: i64) -> DbResult<Vec<String>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT app_name FROM app_categories WHERE category_id = ?1 ORDER BY app_name ASC",
        )?;

        let apps = stmt
            .query_map(params![category_id], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(apps)
    }

    fn add_app_to_category_sync(&self, app_name: &str, category_id: i64) -> DbResult<()> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT OR IGNORE INTO app_categories (app_name, category_id) VALUES (?1, ?2)",
            params![app_name, category_id],
        )?;
        Ok(())
    }

    fn remove_app_from_category_sync(&self, app_name: &str, category_id: i64) -> DbResult<()> {
        let conn = self.pool.get()?;
        conn.execute(
            "DELETE FROM app_categories WHERE app_name = ?1 AND category_id = ?2",
            params![app_name, category_id],
        )?;
        Ok(())
    }

    fn set_app_categories_sync(&self, app_name: &str, category_ids: &[i64]) -> DbResult<()> {
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

    fn get_all_app_names_sync(&self) -> DbResult<Vec<String>> {
        let conn = self.pool.get()?;
        let mut stmt =
            conn.prepare("SELECT DISTINCT app_name FROM window_events ORDER BY app_name ASC")?;

        let apps = stmt
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(apps)
    }

    /// 获取分类使用统计（辅助方法，供查询模块使用）
    pub fn get_category_usage_sync(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> DbResult<Vec<CategoryUsage>> {
        let conn = self.pool.get()?;
        let categories = self.get_all_sync()?;

        let mut result = Vec::new();

        for category in categories {
            let category_id = category.id.unwrap();
            let apps = self.get_category_apps_sync(category_id)?;

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
            let placeholders: Vec<String> = apps
                .iter()
                .enumerate()
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
            let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(start), Box::new(end)];
            for app in &apps {
                params_vec.push(Box::new(app.clone()));
            }

            let params_refs: Vec<&dyn rusqlite::ToSql> =
                params_vec.iter().map(|p| p.as_ref()).collect();

            let app_usages_with_time: Vec<AppUsageInCategory> = stmt
                .query_map(params_refs.as_slice(), |row| {
                    Ok(AppUsageInCategory {
                        app_name: row.get(0)?,
                        total_seconds: row.get(1)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;

            // 创建一个包含所有分类应用的列表，包括没有使用记录的应用
            let mut all_app_usages: Vec<AppUsageInCategory> = Vec::new();
            let apps_with_time: std::collections::HashSet<String> = app_usages_with_time
                .iter()
                .map(|a| a.app_name.clone())
                .collect();

            all_app_usages.extend(app_usages_with_time);

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
}

#[async_trait]
impl CategoryRepository for CategoryRepositoryImpl {
    async fn insert(&self, category: &Category) -> DbResult<i64> {
        let repo = self.clone();
        let category = category.clone();
        tokio::task::spawn_blocking(move || repo.insert_sync(&category))
            .await
            .map_err(|e| DbError::Validation(format!("Task join error: {}", e)))?
    }

    async fn update(&self, category: &Category) -> DbResult<()> {
        let repo = self.clone();
        let category = category.clone();
        tokio::task::spawn_blocking(move || repo.update_sync(&category))
            .await
            .map_err(|e| DbError::Validation(format!("Task join error: {}", e)))?
    }

    async fn delete(&self, id: i64) -> DbResult<()> {
        let repo = self.clone();
        tokio::task::spawn_blocking(move || repo.delete_sync(id))
            .await
            .map_err(|e| DbError::Validation(format!("Task join error: {}", e)))?
    }

    async fn get_all(&self) -> DbResult<Vec<Category>> {
        let repo = self.clone();
        tokio::task::spawn_blocking(move || repo.get_all_sync())
            .await
            .map_err(|e| DbError::Validation(format!("Task join error: {}", e)))?
    }

    async fn get_by_id(&self, id: i64) -> DbResult<Option<Category>> {
        let repo = self.clone();
        tokio::task::spawn_blocking(move || repo.get_by_id_sync(id))
            .await
            .map_err(|e| DbError::Validation(format!("Task join error: {}", e)))?
    }

    async fn get_app_categories(&self, app_name: &str) -> DbResult<Vec<Category>> {
        let repo = self.clone();
        let app_name = app_name.to_string();
        tokio::task::spawn_blocking(move || repo.get_app_categories_sync(&app_name))
            .await
            .map_err(|e| DbError::Validation(format!("Task join error: {}", e)))?
    }

    async fn get_category_apps(&self, category_id: i64) -> DbResult<Vec<String>> {
        let repo = self.clone();
        tokio::task::spawn_blocking(move || repo.get_category_apps_sync(category_id))
            .await
            .map_err(|e| DbError::Validation(format!("Task join error: {}", e)))?
    }

    async fn add_app_to_category(&self, app_name: &str, category_id: i64) -> DbResult<()> {
        let repo = self.clone();
        let app_name = app_name.to_string();
        tokio::task::spawn_blocking(move || repo.add_app_to_category_sync(&app_name, category_id))
            .await
            .map_err(|e| DbError::Validation(format!("Task join error: {}", e)))?
    }

    async fn remove_app_from_category(
        &self,
        app_name: &str,
        category_id: i64,
    ) -> DbResult<()> {
        let repo = self.clone();
        let app_name = app_name.to_string();
        tokio::task::spawn_blocking(move || {
            repo.remove_app_from_category_sync(&app_name, category_id)
        })
        .await
        .map_err(|e| DbError::Validation(format!("Task join error: {}", e)))?
    }

    async fn set_app_categories(&self, app_name: &str, category_ids: &[i64]) -> DbResult<()> {
        let repo = self.clone();
        let app_name = app_name.to_string();
        let category_ids = category_ids.to_vec();
        tokio::task::spawn_blocking(move || repo.set_app_categories_sync(&app_name, &category_ids))
            .await
            .map_err(|e| DbError::Validation(format!("Task join error: {}", e)))?
    }

    async fn get_all_app_names(&self) -> DbResult<Vec<String>> {
        let repo = self.clone();
        tokio::task::spawn_blocking(move || repo.get_all_app_names_sync())
            .await
            .map_err(|e| DbError::Validation(format!("Task join error: {}", e)))?
    }
}

impl Clone for CategoryRepositoryImpl {
    fn clone(&self) -> Self {
        Self {
            pool: Arc::clone(&self.pool),
        }
    }
}
