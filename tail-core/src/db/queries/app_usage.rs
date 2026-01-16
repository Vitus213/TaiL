//! 应用使用查询实现

use crate::db::pool::DbPool;
use crate::db::repositories::WindowEventRepositoryImpl;
use crate::errors::{DbError, DbResult};
use crate::models::AppUsage;
use crate::traits::AppUsageQuery;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::sync::Arc;

/// 应用使用查询实现
pub struct AppUsageQueryImpl {
    window_event_repo: WindowEventRepositoryImpl,
}

impl AppUsageQueryImpl {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self {
            window_event_repo: WindowEventRepositoryImpl::new((*pool).clone()),
        }
    }

    fn get_app_usage_sync(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> DbResult<Vec<AppUsage>> {
        use crate::models::WindowEvent;

        // 获取所有窗口事件
        let events = self.window_event_repo.get_by_time_range_sync(start, end)?;

        // 按应用名称分组并计算总时长
        let mut app_map: std::collections::HashMap<String, (i64, Vec<WindowEvent>)> =
            std::collections::HashMap::new();

        for event in events {
            let entry = app_map
                .entry(event.app_name.clone())
                .or_insert((0, Vec::new()));
            entry.0 += event.duration_secs;
            entry.1.push(event);
        }

        // 转换为 AppUsage 并按总时长排序
        let mut usages: Vec<AppUsage> = app_map
            .into_iter()
            .map(|(app_name, (total_seconds, window_events))| AppUsage {
                app_name,
                total_seconds,
                window_events,
            })
            .collect();

        usages.sort_by(|a, b| b.total_seconds.cmp(&a.total_seconds));

        Ok(usages)
    }
}

#[async_trait]
impl AppUsageQuery for AppUsageQueryImpl {
    async fn get_app_usage(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> DbResult<Vec<AppUsage>> {
        let query = self.clone();
        tokio::task::spawn_blocking(move || query.get_app_usage_sync(start, end))
            .await
            .map_err(|e| DbError::Validation(format!("Task join error: {}", e)))?
    }
}

impl Clone for AppUsageQueryImpl {
    fn clone(&self) -> Self {
        Self {
            window_event_repo: self.window_event_repo.clone(),
        }
    }
}
