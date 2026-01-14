//! TaiL Core - 数据模型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 窗口事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowEvent {
    pub id: Option<i64>,
    pub timestamp: DateTime<Utc>,
    pub app_name: String,
    pub window_title: String,
    pub workspace: String,
    pub duration_secs: i64,
    pub is_afk: bool,
}

/// AFK 事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AfkEvent {
    pub id: Option<i64>,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_secs: i64,
}

/// 每日目标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyGoal {
    pub id: Option<i64>,
    pub app_name: String,
    pub max_minutes: i32,
    pub notify_enabled: bool,
}

/// 应用使用统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUsage {
    pub app_name: String,
    pub total_seconds: i64,
    pub window_events: Vec<WindowEvent>,
}

/// 时间范围
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TimeRange {
    Today,
    Yesterday,
    Last7Days,
    Last30Days,
    Custom(DateTime<Utc>, DateTime<Utc>),
}
