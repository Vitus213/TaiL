//! 应用层端口定义
//!
//! 定义应用层的输入端口（Input Ports），这些端口由适配器（GUI、CLI）调用。

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::domain::{
    aggregation::AggregationResult, navigation::NavigationPath, time_event::TimeEvent,
};

// ============================================================================
// 查询端口
// ============================================================================

/// 统计查询端口 - 输入端口
///
/// GUI/CLI/TUI 等适配器通过此端口查询统计数据
#[async_trait]
pub trait StatsQueryPort: Send + Sync {
    /// 获取仪表板数据（今日统计）
    async fn get_dashboard(&self) -> Result<DashboardView, AppError>;

    /// 获取统计数据（根据导航路径）
    async fn get_stats(&self, navigation: &NavigationPath) -> Result<StatsView, AppError>;

    /// 获取趋势分析
    async fn get_trend(&self) -> Result<TrendView, AppError>;
}

/// 分类管理端口 - 输入端口
#[async_trait]
pub trait CategoryManagementPort: Send + Sync {
    /// 设置应用的分类
    async fn set_app_categories(
        &self,
        app_name: &str,
        category_ids: Vec<i64>,
    ) -> Result<(), AppError>;

    /// 获取所有分类
    async fn get_categories(&self) -> Result<Vec<Category>, AppError>;

    /// 创建新分类
    async fn create_category(&self, name: &str, icon: &str) -> Result<i64, AppError>;

    /// 删除分类
    async fn delete_category(&self, id: i64) -> Result<(), AppError>;
}

// ============================================================================
// 仓储端口 - 输出端口（由基础设施实现）
// ============================================================================

/// 事件仓储端口 - 输出端口
///
/// 由 SQLite 等基础设施实现
#[async_trait]
pub trait EventRepositoryPort: Send + Sync {
    /// 保存事件
    async fn save(&self, event: &TimeEvent) -> Result<(), RepoError>;

    /// 批量保存事件
    async fn save_batch(&self, events: &[TimeEvent]) -> Result<(), RepoError>;

    /// 查找指定时间范围内的事件
    async fn find_by_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<TimeEvent>, RepoError>;

    /// 查找指定应用的事件
    async fn find_by_app(
        &self,
        app_name: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<TimeEvent>, RepoError>;
}

// ============================================================================
// 视图模型（View Models）
// ============================================================================

/// 仪表板视图模型
#[derive(Debug, Clone)]
pub struct DashboardView {
    /// 时间范围
    pub time_range: String,
    /// 总时长（格式化）
    pub total_duration: String,
    /// 总时长（秒）
    pub total_seconds: i64,
    /// 应用使用排行
    pub top_apps: Vec<AppUsageItem>,
    /// 小时分布聚合结果
    pub hourly_breakdown: AggregationResult,
}

/// 应用使用项
#[derive(Debug, Clone)]
pub struct AppUsageItem {
    pub app_name: String,
    pub duration: String,
    pub duration_secs: i64,
    pub percentage: f32,
}

/// 统计视图模型
#[derive(Debug, Clone)]
pub struct StatsView {
    /// 导航面包屑
    pub breadcrumb: String,
    /// 时间段分布
    pub period_breakdown: AggregationResult,
    /// 应用排行
    pub app_breakdown: Vec<AppUsageItem>,
    /// 时间范围
    pub time_range: String,
}

/// 趋势视图模型
#[derive(Debug, Clone)]
pub struct TrendView {
    /// 趋势方向
    pub direction: TrendDirection,
    /// 变化百分比
    pub change_percent: f64,
    /// 增加最多的应用
    pub top_increasing: Vec<(String, f64)>,
    /// 减少最多的应用
    pub top_decreasing: Vec<(String, f64)>,
}

/// 趋势方向
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrendDirection {
    Increasing,
    Stable,
    Decreasing,
}

/// 分类
#[derive(Debug, Clone)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub icon: String,
    pub color: Option<String>,
}

// ============================================================================
// 错误类型
// ============================================================================

/// 应用层错误
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("领域错误: {0}")]
    Domain(#[from] crate::domain::DomainError),

    #[error("仓储错误: {0}")]
    Repository(String),

    #[error("未找到: {0}")]
    NotFound(String),

    #[error("验证错误: {0}")]
    Validation(String),
}

/// 仓储错误
#[derive(Debug, thiserror::Error)]
pub enum RepoError {
    #[error("数据库连接失败: {0}")]
    ConnectionFailed(String),

    #[error("查询执行失败: {0}")]
    QueryFailed(String),

    #[error("序列化错误: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("SQLite 错误: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("数据库错误: {0}")]
    Database(#[from] crate::errors::DbError),

    #[error("连接池错误: {0}")]
    PoolError(#[from] r2d2::Error),

    #[error("领域错误: {0}")]
    Domain(#[from] crate::domain::DomainError),
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 格式化时长
pub fn format_duration(seconds: i64) -> String {
    if seconds < 60 {
        format!("{}秒", seconds)
    } else if seconds < 3600 {
        format!("{}分{}秒", seconds / 60, seconds % 60)
    } else {
        format!("{}时{}分", seconds / 3600, (seconds % 3600) / 60)
    }
}

/// 解析时长字符串（从格式化后的字符串）
pub fn parse_duration(s: &str) -> Option<i64> {
    // 只包含秒
    if let Some(rest) = s.strip_suffix("秒") {
        if !rest.contains('时') && !rest.contains('分') {
            return rest.trim().parse().ok();
        }
    }

    // 包含时和分
    if s.contains('时') && s.contains('分') {
        let parts: Vec<_> = s.split(&['时', '分'][..]).collect();
        if parts.len() >= 2 {
            let hours: i64 = parts[0].parse().ok()?;
            let minutes: i64 = parts[1].parse().ok()?;
            return Some(hours * 3600 + minutes * 60);
        }
    }

    // 包含分和秒
    if s.contains('分') && s.contains('秒') {
        let rest = s.trim_end_matches('秒');
        let parts: Vec<_> = rest.split('分').collect();
        if parts.len() == 2 {
            let minutes: i64 = parts[0].parse().ok()?;
            let seconds: i64 = parts[1].parse().ok()?;
            return Some(minutes * 60 + seconds);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(30), "30秒");
        assert_eq!(format_duration(90), "1分30秒");
        assert_eq!(format_duration(3661), "1时1分");
    }

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("30秒"), Some(30));
        assert_eq!(parse_duration("1分30秒"), Some(90));
    }
}
