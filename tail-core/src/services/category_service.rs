//! 分类服务实现

use crate::db::pool::DbPool;
use crate::db::repositories::CategoryRepositoryImpl;
use crate::errors::DbResult;
use crate::models::{Category, CategoryUsage};
use crate::traits::CategoryRepository;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::sync::Arc;

/// 分类管理数据
#[derive(Debug, Clone)]
pub struct CategoryManagementData {
    /// 所有分类
    pub categories: Vec<Category>,
    /// 分类使用统计
    pub category_usage: Vec<CategoryUsage>,
    /// 所有应用名称
    pub all_app_names: Vec<String>,
}

/// 分类服务实现
pub struct CategoryServiceImpl {
    category_repo: CategoryRepositoryImpl,
}

impl CategoryServiceImpl {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self {
            category_repo: CategoryRepositoryImpl::new(pool),
        }
    }

    /// 获取分类管理所需的所有数据
    pub async fn get_category_management_data(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> DbResult<CategoryManagementData> {
        let categories = self.category_repo.get_all().await?;
        let category_usage = self.category_repo.get_category_usage_sync(start, end)?;
        let all_app_names = self.category_repo.get_all_app_names().await?;

        Ok(CategoryManagementData {
            categories,
            category_usage,
            all_app_names,
        })
    }
}

#[async_trait]
impl CategoryRepository for CategoryServiceImpl {
    async fn insert(&self, category: &Category) -> DbResult<i64> {
        self.category_repo.insert(category).await
    }

    async fn update(&self, category: &Category) -> DbResult<()> {
        self.category_repo.update(category).await
    }

    async fn delete(&self, id: i64) -> DbResult<()> {
        self.category_repo.delete(id).await
    }

    async fn get_all(&self) -> DbResult<Vec<Category>> {
        self.category_repo.get_all().await
    }

    async fn get_by_id(&self, id: i64) -> DbResult<Option<Category>> {
        self.category_repo.get_by_id(id).await
    }

    async fn get_app_categories(&self, app_name: &str) -> DbResult<Vec<Category>> {
        self.category_repo.get_app_categories(app_name).await
    }

    async fn get_category_apps(&self, category_id: i64) -> DbResult<Vec<String>> {
        self.category_repo.get_category_apps(category_id).await
    }

    async fn add_app_to_category(&self, app_name: &str, category_id: i64) -> DbResult<()> {
        self.category_repo
            .add_app_to_category(app_name, category_id)
            .await
    }

    async fn remove_app_from_category(&self, app_name: &str, category_id: i64) -> DbResult<()> {
        self.category_repo
            .remove_app_from_category(app_name, category_id)
            .await
    }

    async fn set_app_categories(&self, app_name: &str, category_ids: &[i64]) -> DbResult<()> {
        self.category_repo
            .set_app_categories(app_name, category_ids)
            .await
    }

    async fn get_all_app_names(&self) -> DbResult<Vec<String>> {
        self.category_repo.get_all_app_names().await
    }
}

impl Clone for CategoryServiceImpl {
    fn clone(&self) -> Self {
        Self {
            category_repo: self.category_repo.clone(),
        }
    }
}
