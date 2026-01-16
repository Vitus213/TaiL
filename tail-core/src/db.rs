//! TaiL Core - 数据库模块（重构后）
//!
//! 提供模块化的数据库访问层，通过仓储模式和服务层实现高内聚低耦合。

pub mod pool;
pub mod queries;
pub mod repositories;

use pool::{create_pool, init_schema};
use std::sync::Arc;

use crate::services::{CategoryServiceImpl, GoalServiceImpl, UsageServiceImpl};

// 重新导出 pool 模块的内容
pub use pool::DbConfig as Config;
pub use pool::DbPool;

// ============================================================================
// Repository - 模块化数据库入口
// ============================================================================

/// 模块化数据库入口
///
/// 提供访问各个仓储和服务的方法。
pub struct Repository {
    pool: Arc<DbPool>,
}

impl Repository {
    /// 创建新的数据库连接
    pub fn new(config: &Config) -> Result<Self, crate::errors::DbError> {
        let pool = Arc::new(create_pool(config)?);
        init_schema(&pool)?;
        Ok(Self { pool })
    }

    /// 从连接池创建
    pub fn with_pool(pool: DbPool) -> Self {
        Self {
            pool: Arc::new(pool),
        }
    }

    /// 获取原始连接池（供内部使用）
    pub fn pool(&self) -> Arc<DbPool> {
        Arc::clone(&self.pool)
    }

    // ========================================================================
    // 仓储访问
    // ========================================================================

    /// 获取窗口事件仓储
    pub fn window_events(&self) -> repositories::WindowEventRepositoryImpl {
        repositories::WindowEventRepositoryImpl::new((*self.pool).clone())
    }

    /// 获取 AFK 事件仓储
    pub fn afk_events(&self) -> repositories::AfkEventRepositoryImpl {
        repositories::AfkEventRepositoryImpl::new((*self.pool).clone())
    }

    /// 获取每日目标仓储
    pub fn daily_goals(&self) -> repositories::DailyGoalRepositoryImpl {
        repositories::DailyGoalRepositoryImpl::new((*self.pool).clone())
    }

    /// 获取分类仓储
    pub fn categories(&self) -> repositories::CategoryRepositoryImpl {
        repositories::CategoryRepositoryImpl::new(Arc::clone(&self.pool))
    }

    /// 获取别名仓储
    pub fn aliases(&self) -> repositories::AliasRepositoryImpl {
        repositories::AliasRepositoryImpl::new(Arc::clone(&self.pool))
    }

    // ========================================================================
    // 服务层访问
    // ========================================================================

    /// 获取使用统计服务
    pub fn usage_service(&self) -> UsageServiceImpl {
        UsageServiceImpl::new(Arc::clone(&self.pool))
    }

    /// 获取分类服务
    pub fn category_service(&self) -> CategoryServiceImpl {
        CategoryServiceImpl::new(Arc::clone(&self.pool))
    }

    /// 获取目标服务
    pub fn goal_service(&self) -> GoalServiceImpl {
        GoalServiceImpl::new((*self.pool).clone())
    }
}

// ============================================================================
// 便捷 Trait 实现（让 Repository 可以直接作为 Trait 使用）
// ============================================================================

#[async_trait::async_trait]
impl crate::traits::WindowEventRepository for Repository {
    async fn insert(&self, event: &crate::models::WindowEvent) -> crate::errors::DbResult<i64> {
        self.window_events().insert(event).await
    }

    async fn get_by_time_range(
        &self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> crate::errors::DbResult<Vec<crate::models::WindowEvent>> {
        self.window_events().get_by_time_range(start, end).await
    }

    async fn update_duration(&self, id: i64, duration_secs: i64) -> crate::errors::DbResult<()> {
        self.window_events()
            .update_duration(id, duration_secs)
            .await
    }
}

#[async_trait::async_trait]
impl crate::traits::AfkEventRepository for Repository {
    async fn insert(&self, event: &crate::models::AfkEvent) -> crate::errors::DbResult<i64> {
        self.afk_events().insert(event).await
    }

    async fn update_end(
        &self,
        id: i64,
        end_time: chrono::DateTime<chrono::Utc>,
        duration_secs: i64,
    ) -> crate::errors::DbResult<()> {
        self.afk_events()
            .update_end(id, end_time, duration_secs)
            .await
    }

    async fn get_by_time_range(
        &self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> crate::errors::DbResult<Vec<crate::models::AfkEvent>> {
        self.afk_events().get_by_time_range(start, end).await
    }
}

#[async_trait::async_trait]
impl crate::traits::DailyGoalRepository for Repository {
    async fn upsert(&self, goal: &crate::models::DailyGoal) -> crate::errors::DbResult<i64> {
        self.daily_goals().upsert(goal).await
    }

    async fn get_all(&self) -> crate::errors::DbResult<Vec<crate::models::DailyGoal>> {
        self.daily_goals().get_all().await
    }

    async fn delete(&self, app_name: &str) -> crate::errors::DbResult<()> {
        self.daily_goals().delete(app_name).await
    }

    async fn get_today_usage(&self, app_name: &str) -> crate::errors::DbResult<i64> {
        self.daily_goals().get_today_usage(app_name).await
    }
}

#[async_trait::async_trait]
impl crate::traits::CategoryRepository for Repository {
    async fn insert(&self, category: &crate::models::Category) -> crate::errors::DbResult<i64> {
        self.categories().insert(category).await
    }

    async fn update(&self, category: &crate::models::Category) -> crate::errors::DbResult<()> {
        self.categories().update(category).await
    }

    async fn delete(&self, id: i64) -> crate::errors::DbResult<()> {
        self.categories().delete(id).await
    }

    async fn get_all(&self) -> crate::errors::DbResult<Vec<crate::models::Category>> {
        self.categories().get_all().await
    }

    async fn get_by_id(&self, id: i64) -> crate::errors::DbResult<Option<crate::models::Category>> {
        self.categories().get_by_id(id).await
    }

    async fn get_app_categories(
        &self,
        app_name: &str,
    ) -> crate::errors::DbResult<Vec<crate::models::Category>> {
        self.categories().get_app_categories(app_name).await
    }

    async fn get_category_apps(&self, category_id: i64) -> crate::errors::DbResult<Vec<String>> {
        self.categories().get_category_apps(category_id).await
    }

    async fn add_app_to_category(
        &self,
        app_name: &str,
        category_id: i64,
    ) -> crate::errors::DbResult<()> {
        self.categories()
            .add_app_to_category(app_name, category_id)
            .await
    }

    async fn remove_app_from_category(
        &self,
        app_name: &str,
        category_id: i64,
    ) -> crate::errors::DbResult<()> {
        self.categories()
            .remove_app_from_category(app_name, category_id)
            .await
    }

    async fn set_app_categories(
        &self,
        app_name: &str,
        category_ids: &[i64],
    ) -> crate::errors::DbResult<()> {
        self.categories()
            .set_app_categories(app_name, category_ids)
            .await
    }

    async fn get_all_app_names(&self) -> crate::errors::DbResult<Vec<String>> {
        self.categories().get_all_app_names().await
    }
}

#[async_trait::async_trait]
impl crate::traits::AliasRepository for Repository {
    async fn set(&self, app_name: &str, alias: &str) -> crate::errors::DbResult<()> {
        self.aliases().set(app_name, alias).await
    }

    async fn get(&self, app_name: &str) -> crate::errors::DbResult<Option<String>> {
        self.aliases().get(app_name).await
    }

    async fn get_all(&self) -> crate::errors::DbResult<Vec<(String, String)>> {
        self.aliases().get_all().await
    }

    async fn delete(&self, app_name: &str) -> crate::errors::DbResult<()> {
        self.aliases().delete(app_name).await
    }
}
