//! 分类使用查询实现

use crate::errors::{DbError, DbResult};
use crate::models::CategoryUsage;
use crate::traits::CategoryUsageQuery;
use crate::db::repositories::CategoryRepositoryImpl;
use crate::db::pool::DbPool;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::sync::Arc;

/// 分类使用查询实现
pub struct CategoryUsageQueryImpl {
    category_repo: CategoryRepositoryImpl,
}

impl CategoryUsageQueryImpl {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self {
            category_repo: CategoryRepositoryImpl::new(pool),
        }
    }

    fn get_category_usage_sync(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> DbResult<Vec<CategoryUsage>> {
        self.category_repo.get_category_usage_sync(start, end)
    }
}

#[async_trait]
impl CategoryUsageQuery for CategoryUsageQueryImpl {
    async fn get_category_usage(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> DbResult<Vec<CategoryUsage>> {
        let query = self.clone();
        tokio::task::spawn_blocking(move || query.get_category_usage_sync(start, end))
            .await
            .map_err(|e| DbError::Validation(format!("Task join error: {}", e)))?
    }
}

impl Clone for CategoryUsageQueryImpl {
    fn clone(&self) -> Self {
        Self {
            category_repo: self.category_repo.clone(),
        }
    }
}
