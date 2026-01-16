//! 应用状态管理

use crate::services::DataService;
use std::sync::Arc;

/// 应用状态存储
pub struct AppStore {
    /// 数据服务
    pub data: Arc<DataService>,
    /// 是否需要刷新
    needs_refresh: bool,
}

impl AppStore {
    /// 创建新的应用状态存储
    pub fn new(data: Arc<DataService>) -> Self {
        Self {
            data,
            needs_refresh: true,
        }
    }

    /// 标记需要刷新
    pub fn mark_dirty(&mut self) {
        self.needs_refresh = true;
    }

    /// 检查是否需要刷新
    pub fn is_dirty(&self) -> bool {
        self.needs_refresh
    }

    /// 清除刷新标记
    pub fn mark_clean(&mut self) {
        self.needs_refresh = false;
    }

    /// 获取数据服务引用
    pub fn data_service(&self) -> &DataService {
        &self.data
    }
}
