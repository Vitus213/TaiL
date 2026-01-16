//! 缓存服务 - 通用缓存管理（支持 TTL）

use chrono::{DateTime, Utc};
use std::time::Duration;

/// 缓存服务
#[derive(Debug, Clone)]
pub struct CacheService<T> {
    data: Option<T>,
    last_refresh: Option<DateTime<Utc>>,
    ttl: Duration,
}

impl<T: Clone> CacheService<T> {
    /// 创建新的缓存服务
    pub fn new(ttl: Duration) -> Self {
        Self {
            data: None,
            last_refresh: None,
            ttl,
        }
    }

    /// 检查缓存是否有效
    pub fn is_valid(&self) -> bool {
        if let Some(last) = self.last_refresh {
            let elapsed = Utc::now() - last;
            elapsed.num_seconds() < self.ttl.as_secs() as i64
        } else {
            false
        }
    }

    /// 获取缓存数据或刷新
    pub fn get_or_refresh<F>(&mut self, fetch: F) -> Result<&T, String>
    where
        F: FnOnce() -> Result<T, String>,
    {
        if self.is_valid() {
            Ok(self.data.as_ref().unwrap())
        } else {
            let data = fetch()?;
            self.data = Some(data);
            self.last_refresh = Some(Utc::now());
            Ok(self.data.as_ref().unwrap())
        }
    }

    /// 直接设置缓存数据
    pub fn set(&mut self, data: T) {
        self.data = Some(data);
        self.last_refresh = Some(Utc::now());
    }

    /// 获取缓存数据（不检查有效性）
    pub fn get(&self) -> Option<&T> {
        self.data.as_ref()
    }

    /// 使缓存失效
    pub fn invalidate(&mut self) {
        self.data = None;
        self.last_refresh = None;
    }
}

impl<T: Clone> Default for CacheService<T> {
    fn default() -> Self {
        Self::new(Duration::from_secs(5))
    }
}
