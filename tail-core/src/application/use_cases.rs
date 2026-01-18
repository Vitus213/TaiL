//! 用例层 - 业务逻辑编排
//!
//! 此模块包含应用层的用例实现，协调领域对象完成业务逻辑。

use async_trait::async_trait;

use crate::application::ports::{
    AppError, AppUsageItem, CategoryManagementPort, DashboardView, EventRepositoryPort,
    StatsQueryPort, StatsView, TrendView, format_duration,
};
use crate::domain::{
    TimeSeriesAnalyzer, NavigationPath, TimeRange,
    aggregation::{TimeGranularity, TrendDirection},
};

/// 统计查询用例
///
/// 编排领域对象完成统计查询功能
pub struct StatsQueryUseCase<R> {
    event_repo: R,
}

impl<R> StatsQueryUseCase<R>
where
    R: EventRepositoryPort + Send + Sync,
{
    /// 创建新的用例实例
    pub fn new(event_repo: R) -> Self {
        Self { event_repo }
    }

    /// 获取应用使用排行
    async fn get_app_ranking(&self, range: &TimeRange) -> Result<Vec<AppUsageItem>, AppError> {
        let events = self
            .event_repo
            .find_by_range(range.start, range.end)
            .await
            .map_err(|e| AppError::Repository(e.to_string()))?;

        // 使用领域服务聚合
        let result = TimeSeriesAnalyzer::aggregate(&events, TimeGranularity::Hour, Some(range));
        let total_seconds = result.total_duration();

        // 构建应用排行
        let mut app_usage: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
        for event in &events {
            if !event.is_afk {
                *app_usage
                    .entry(event.app_name.as_str().to_string())
                    .or_insert(0) += event.duration_secs();
            }
        }

        let mut items: Vec<_> = app_usage
            .into_iter()
            .map(|(name, secs)| {
                let percentage = if total_seconds > 0 {
                    (secs as f32 / total_seconds as f32) * 100.0
                } else {
                    0.0
                };
                AppUsageItem {
                    app_name: name.clone(),
                    duration: format_duration(secs),
                    duration_secs: secs,
                    percentage,
                }
            })
            .collect();

        items.sort_by(|a, b| b.duration_secs.cmp(&a.duration_secs));
        items.truncate(20);

        Ok(items)
    }
}

#[async_trait]
impl<R> StatsQueryPort for StatsQueryUseCase<R>
where
    R: EventRepositoryPort + Send + Sync,
{
    async fn get_dashboard(&self) -> Result<DashboardView, AppError> {
        // 暂时使用本周数据范围，以便在数据较少时也能显示内容
        let range = TimeRange::this_week();

        let events = self
            .event_repo
            .find_by_range(range.start, range.end)
            .await
            .map_err(|e| AppError::Repository(e.to_string()))?;

        // 使用领域服务聚合
        let hourly_breakdown =
            TimeSeriesAnalyzer::aggregate(&events, TimeGranularity::Hour, Some(&range));

        let total_seconds = hourly_breakdown.total_duration();

        // 获取应用排行
        let top_apps = self.get_app_ranking(&range).await?;

        Ok(DashboardView {
            time_range: range.to_string(),
            total_duration: format_duration(total_seconds),
            total_seconds,
            top_apps,
            hourly_breakdown,
        })
    }

    async fn get_stats(&self, navigation: &NavigationPath) -> Result<StatsView, AppError> {
        let range = navigation.current_range();
        let granularity = navigation.current_granularity();

        let events = self
            .event_repo
            .find_by_range(range.start, range.end)
            .await
            .map_err(|e| AppError::Repository(e.to_string()))?;

        // 使用领域服务聚合
        let period_breakdown =
            TimeSeriesAnalyzer::aggregate(&events, granularity, Some(&range));

        // 获取应用排行
        let top_apps = self.get_app_ranking(&range).await?;

        Ok(StatsView {
            breadcrumb: navigation.breadcrumb(),
            period_breakdown,
            app_breakdown: top_apps,
            time_range: range.to_string(),
        })
    }

    async fn get_trend(&self) -> Result<TrendView, AppError> {
        let current_range = TimeRange::this_week();
        let previous_range = {
            let start = current_range.start - chrono::Duration::days(7);
            let end = current_range.end - chrono::Duration::days(7);
            TimeRange::new(start, end).unwrap()
        };

        let current_events = self
            .event_repo
            .find_by_range(current_range.start, current_range.end)
            .await
            .map_err(|e| AppError::Repository(e.to_string()))?;

        let previous_events = self
            .event_repo
            .find_by_range(previous_range.start, previous_range.end)
            .await
            .map_err(|e| AppError::Repository(e.to_string()))?;

        let current_result =
            TimeSeriesAnalyzer::aggregate(&current_events, TimeGranularity::Day, Some(&current_range));
        let previous_result = TimeSeriesAnalyzer::aggregate(
            &previous_events,
            TimeGranularity::Day,
            Some(&previous_range),
        );

        let trend = TimeSeriesAnalyzer::calculate_trend(&current_result, &previous_result);

        Ok(TrendView {
            direction: match trend.direction {
                TrendDirection::Increasing => crate::application::ports::TrendDirection::Increasing,
                TrendDirection::Stable => crate::application::ports::TrendDirection::Stable,
                TrendDirection::Decreasing => crate::application::ports::TrendDirection::Decreasing,
            },
            change_percent: trend.change_percent,
            top_increasing: trend.top_increasing,
            top_decreasing: trend.top_decreasing,
        })
    }
}

/// 分类管理用例
///
/// 编排分类相关的业务逻辑
pub struct CategoryManagementUseCase<C> {
    #[allow(dead_code)]
    category_repo: C,
}

impl<C> CategoryManagementUseCase<C>
where
    C: CategoryManagementPort + Send + Sync,
{
    /// 创建新的用例实例
    pub fn new(category_repo: C) -> Self {
        Self { category_repo }
    }

    /// 获取所有应用及其分类
    pub async fn get_all_apps_with_categories(&self) -> Result<Vec<AppCategory>, AppError> {
        // 这个方法需要从仓储获取数据
        // 暂时返回空列表
        Ok(Vec::new())
    }
}

/// 应用分类
#[derive(Debug, Clone)]
pub struct AppCategory {
    pub app_name: String,
    pub category_id: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::TimeEvent;
    use crate::application::ports::RepoError;
    use chrono::{Utc, Duration as ChronoDuration};

    // Mock 实现
    struct MockEventRepository;

    #[async_trait]
    impl EventRepositoryPort for MockEventRepository {
        async fn save(&self, _event: &TimeEvent) -> Result<(), RepoError> {
            Ok(())
        }

        async fn save_batch(&self, _events: &[TimeEvent]) -> Result<(), RepoError> {
            Ok(())
        }

        async fn find_by_range(
            &self,
            start: chrono::DateTime<Utc>,
            _end: chrono::DateTime<Utc>,
        ) -> Result<Vec<TimeEvent>, RepoError> {
            // 使用时间范围的开始时间加上偏移，确保事件在范围内
            let base = start + ChronoDuration::seconds(3600); // 范围开始后1小时
            Ok(vec![
                TimeEvent::new(base, "firefox", 3600).unwrap(),
                TimeEvent::new(base + ChronoDuration::seconds(3600), "vscode", 7200).unwrap(),
            ])
        }

        async fn find_by_app(
            &self,
            _app_name: &str,
            _start: chrono::DateTime<Utc>,
            _end: chrono::DateTime<Utc>,
        ) -> Result<Vec<TimeEvent>, RepoError> {
            Ok(Vec::new())
        }
    }

    #[tokio::test]
    async fn test_get_dashboard() {
        let repo = MockEventRepository;
        let use_case = StatsQueryUseCase::new(repo);

        let dashboard = use_case.get_dashboard().await.unwrap();

        assert!(!dashboard.top_apps.is_empty());
        assert!(dashboard.total_seconds > 0);
    }

    #[tokio::test]
    async fn test_get_stats() {
        let repo = MockEventRepository;
        let use_case = StatsQueryUseCase::new(repo);

        let navigation = NavigationPath::new();
        let stats = use_case.get_stats(&navigation).await.unwrap();

        assert!(!stats.app_breakdown.is_empty());
    }
}
