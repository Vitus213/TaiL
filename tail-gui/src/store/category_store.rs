//! 分类页面状态管理

use std::sync::Arc;
use std::time::Duration;
use tail_core::CategoryManagementData;
use crate::services::{CacheService, DataService};

/// 分类页面状态存储
pub struct CategoryStore {
    /// 数据服务
    data_service: Arc<DataService>,
    /// 缓存的分类管理数据
    cache: CacheService<CategoryManagementData>,
}

impl CategoryStore {
    /// 创建新的分类页面状态存储
    pub fn new(data_service: Arc<DataService>) -> Self {
        Self {
            data_service,
            cache: CacheService::new(Duration::from_secs(10)),
        }
    }

    /// 获取分类管理数据（使用缓存）
    pub fn get_data(&mut self) -> Result<&CategoryManagementData, String> {
        let data_service = Arc::clone(&self.data_service);
        self.cache.get_or_refresh(|| {
            // 获取今天的开始和结束时间
            let local_now = chrono::Local::now();
            let today_start = local_now
                .date_naive()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_local_timezone(chrono::Local)
                .unwrap()
                .with_timezone(&chrono::Utc);
            let now = chrono::Utc::now();

            data_service.get_category_management_data_blocking(today_start, now)
        })
    }

    /// 刷新分类管理数据
    pub fn refresh(&mut self) -> Result<(), String> {
        self.cache.invalidate();
        self.get_data().map(|_| ())
    }

    /// 使缓存失效
    pub fn invalidate(&mut self) {
        self.cache.invalidate();
    }
}
