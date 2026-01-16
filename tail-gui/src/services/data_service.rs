//! 数据服务 - 封装异步数据查询

use std::sync::Arc;
use tokio::runtime::Handle;
use tail_core::{services::*, DashboardData, StatsData, CategoryManagementData, GoalProgress};

/// GUI 数据服务入口
pub struct DataService {
    runtime: Handle,
    usage_service: Arc<UsageServiceImpl>,
    category_service: Arc<CategoryServiceImpl>,
    goal_service: Arc<GoalServiceImpl>,
}

impl DataService {
    /// 创建新的数据服务
    pub fn new(
        runtime: Handle,
        usage_service: Arc<UsageServiceImpl>,
        category_service: Arc<CategoryServiceImpl>,
        goal_service: Arc<GoalServiceImpl>,
    ) -> Self {
        Self {
            runtime,
            usage_service,
            category_service,
            goal_service,
        }
    }

    /// 获取使用统计服务
    pub fn usage(&self) -> &UsageServiceImpl {
        &self.usage_service
    }

    /// 获取分类服务
    pub fn category(&self) -> &CategoryServiceImpl {
        &self.category_service
    }

    /// 获取目标服务
    pub fn goal(&self) -> &GoalServiceImpl {
        &self.goal_service
    }

    /// 异步获取仪表板数据
    pub fn get_dashboard_data_blocking(&self) -> Result<DashboardData, String> {
        self.runtime
            .block_on(self.usage_service.get_dashboard_data())
            .map_err(|e| e.to_string())
    }

    /// 异步获取统计数据
    pub fn get_stats_data_blocking(
        &self,
        state: &tail_core::models::TimeNavigationState,
    ) -> Result<StatsData, String> {
        self.runtime
            .block_on(self.usage_service.get_stats_data(state))
            .map_err(|e| e.to_string())
    }

    /// 异步获取分类管理数据
    pub fn get_category_management_data_blocking(
        &self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> Result<CategoryManagementData, String> {
        self.runtime
            .block_on(self.category_service.get_category_management_data(start, end))
            .map_err(|e| e.to_string())
    }

    /// 异步获取所有目标进度
    pub fn get_all_goal_progress_blocking(&self) -> Result<Vec<GoalProgress>, String> {
        self.runtime
            .block_on(self.goal_service.get_all_goal_progress())
            .map_err(|e| e.to_string())
    }
}
