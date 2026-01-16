//! 统计页面状态管理

use std::sync::Arc;
use std::time::Duration;
use tail_core::{models::TimeNavigationState, StatsData};
use crate::services::{CacheService, DataService};
use chrono::Datelike;

/// 统计页面状态存储
pub struct StatsStore {
    /// 数据服务
    data_service: Arc<DataService>,
    /// 缓存的统计数据
    cache: CacheService<StatsData>,
    /// 当前时间导航状态
    pub time_state: TimeNavigationState,
}

impl StatsStore {
    /// 创建新的统计页面状态存储
    pub fn new(data_service: Arc<DataService>) -> Self {
        let current_year = chrono::Local::now().year();
        Self {
            data_service,
            cache: CacheService::new(Duration::from_secs(5)),
            time_state: TimeNavigationState::new(current_year),
        }
    }

    /// 获取统计数据（使用缓存）
    pub fn get_data(&mut self) -> Result<&StatsData, String> {
        self.cache
            .get_or_refresh(|| self.data_service.get_stats_data_blocking(&self.time_state))
    }

    /// 刷新统计数据
    pub fn refresh(&mut self) -> Result<(), String> {
        self.cache.invalidate();
        self.get_data().map(|_| ())
    }

    /// 使缓存失效
    pub fn invalidate(&mut self) {
        self.cache.invalidate();
    }

    /// 设置时间导航状态
    pub fn set_time_state(&mut self, state: TimeNavigationState) {
        if self.time_state != state {
            self.time_state = state;
            self.cache.invalidate();
        }
    }
}
