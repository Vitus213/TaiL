//! 仪表板状态管理

use std::sync::Arc;
use std::time::Duration;
use tail_core::DashboardData;
use crate::services::{CacheService, DataService};

/// 仪表板状态存储
pub struct DashboardStore {
    /// 数据服务
    data_service: Arc<DataService>,
    /// 缓存的仪表板数据
    cache: CacheService<DashboardData>,
}

impl DashboardStore {
    /// 创建新的仪表板状态存储
    pub fn new(data_service: Arc<DataService>) -> Self {
        Self {
            data_service,
            cache: CacheService::new(Duration::from_secs(5)),
        }
    }

    /// 获取仪表板数据（使用缓存）
    pub fn get_data(&mut self) -> Result<&DashboardData, String> {
        self.cache.get_or_refresh(|| self.data_service.get_dashboard_data_blocking())
    }

    /// 刷新仪表板数据
    pub fn refresh(&mut self) -> Result<(), String> {
        self.cache.invalidate();
        self.get_data().map(|_| ())
    }

    /// 使缓存失效
    pub fn invalidate(&mut self) {
        self.cache.invalidate();
    }

    /// 获取总使用时长
    pub fn get_total_seconds(&self) -> i64 {
        self.cache
            .get()
            .map(|d| d.app_usage.iter().map(|u| u.total_seconds).sum())
            .unwrap_or(0)
    }

    /// 获取活跃应用数量
    pub fn get_app_count(&self) -> usize {
        self.cache
            .get()
            .map(|d| d.app_usage.iter().filter(|u| !u.app_name.is_empty()).count())
            .unwrap_or(0)
    }
}
