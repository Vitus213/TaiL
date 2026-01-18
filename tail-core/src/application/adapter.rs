//! 仓储适配器
//!
//! 将旧的 Repository 适配到新的 EventRepositoryPort 接口

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::sync::Arc;

use crate::application::ports::EventRepositoryPort;
use crate::domain::TimeEvent;
use crate::models::{WindowEvent, AfkEvent};
use crate::traits::{WindowEventRepository, AfkEventRepository};

/// Repository 适配器
///
/// 将旧的仓储接口适配到新的端口接口
pub struct RepositoryAdapter {
    window_events: Arc<dyn WindowEventRepository + Send + Sync>,
    afk_events: Arc<dyn AfkEventRepository + Send + Sync>,
}

impl RepositoryAdapter {
    /// 从旧的 Repository 创建适配器
    pub fn from_repository(repo: Arc<crate::Repository>) -> Self {
        Self {
            window_events: Arc::new(repo.window_events()),
            afk_events: Arc::new(repo.afk_events()),
        }
    }
}

#[async_trait]
impl EventRepositoryPort for RepositoryAdapter {
    async fn save(&self, event: &TimeEvent) -> Result<(), crate::application::ports::RepoError> {
        if event.is_afk {
            // AFK 事件转换为 AfkEvent
            let afk_event = AfkEvent {
                id: None,
                start_time: event.timestamp,
                end_time: Some(event.timestamp + chrono::Duration::seconds(event.duration_secs())),
                duration_secs: event.duration_secs(),
            };
            self.afk_events.insert(&afk_event).await
                .map_err(|e| crate::application::ports::RepoError::ConnectionFailed(e.to_string()))?;
        } else {
            // 窗口事件转换为 WindowEvent
            let window_event = WindowEvent {
                id: None,
                timestamp: event.timestamp,
                app_name: event.app_name.as_str().to_string(),
                window_title: event.window_title.clone().unwrap_or_default(),
                workspace: event.workspace.clone().unwrap_or_default(),
                duration_secs: event.duration_secs(),
                is_afk: false,
            };
            self.window_events.insert(&window_event).await
                .map_err(|e| crate::application::ports::RepoError::ConnectionFailed(e.to_string()))?;
        }
        Ok(())
    }

    async fn save_batch(&self, events: &[TimeEvent]) -> Result<(), crate::application::ports::RepoError> {
        for event in events {
            self.save(event).await?;
        }
        Ok(())
    }

    async fn find_by_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<TimeEvent>, crate::application::ports::RepoError> {
        // 获取窗口事件
        let window_events = self.window_events.get_by_time_range(start, end).await
            .map_err(|e| crate::application::ports::RepoError::ConnectionFailed(e.to_string()))?;

        // 获取 AFK 事件
        let afk_events = self.afk_events.get_by_time_range(start, end).await
            .map_err(|e| crate::application::ports::RepoError::ConnectionFailed(e.to_string()))?;

        let mut result = Vec::new();

        // 转换窗口事件（过滤掉应用名称为空的事件）
        for we in window_events {
            // 跳过应用名称为空的事件（可能是无效数据）
            if we.app_name.trim().is_empty() {
                continue;
            }
            let mut evt = TimeEvent::new(we.timestamp, we.app_name, we.duration_secs)
                .map_err(|e| crate::application::ports::RepoError::Domain(e))?;
            if !we.window_title.is_empty() {
                evt = evt.with_window_title(we.window_title);
            }
            if !we.workspace.is_empty() {
                evt = evt.with_workspace(we.workspace);
            }
            result.push(evt);
        }

        // 转换 AFK 事件
        for ae in afk_events {
            let evt = TimeEvent::afk(ae.start_time, ae.duration_secs)
                .map_err(|e| crate::application::ports::RepoError::Domain(e))?;
            result.push(evt);
        }

        // 按时间戳排序
        result.sort_by_key(|e| e.timestamp);

        Ok(result)
    }

    async fn find_by_app(
        &self,
        app_name: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<TimeEvent>, crate::application::ports::RepoError> {
        let all_events = self.find_by_range(start, end).await?;
        let filtered: Vec<TimeEvent> = all_events
            .into_iter()
            .filter(|e| e.app_name.as_str() == app_name)
            .collect();
        Ok(filtered)
    }
}
