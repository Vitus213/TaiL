//! 使用统计服务实现

use crate::errors::DbResult;
use crate::models::*;
use crate::traits::{AppUsageQuery, CategoryUsageQuery, TimeStatsQuery};
use crate::db::queries::{AppUsageQueryImpl, CategoryUsageQueryImpl, TimeStatsQueryImpl};
use crate::db::pool::DbPool;
use async_trait::async_trait;
use chrono::{DateTime, Local, Utc};
use std::sync::Arc;

/// 仪表板数据
#[derive(Debug, Clone)]
pub struct DashboardData {
    /// 应用使用统计
    pub app_usage: Vec<AppUsage>,
    /// 每日目标
    pub daily_goals: Vec<DailyGoal>,
    /// 统计时间
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

/// 统计数据
#[derive(Debug, Clone)]
pub struct StatsData {
    /// 应用使用统计
    pub app_usage: Vec<AppUsage>,
    /// 分类使用统计
    pub category_usage: Vec<CategoryUsage>,
    /// 时间段数据
    pub period_usage: Vec<PeriodUsage>,
    /// 统计时间范围
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

/// 使用统计服务实现（聚合接口）
pub struct UsageServiceImpl {
    app_usage_query: AppUsageQueryImpl,
    category_usage_query: CategoryUsageQueryImpl,
    time_stats_query: TimeStatsQueryImpl,
}

impl UsageServiceImpl {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self {
            app_usage_query: AppUsageQueryImpl::new(Arc::clone(&pool)),
            category_usage_query: CategoryUsageQueryImpl::new(Arc::clone(&pool)),
            time_stats_query: TimeStatsQueryImpl::new(pool),
        }
    }

    /// 获取仪表板数据
    pub async fn get_dashboard_data(&self) -> DbResult<DashboardData> {
        let local_now = Local::now();
        let today_start = local_now
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .with_timezone(&Utc);
        let now = Utc::now();

        let app_usage = self.get_app_usage(today_start, now).await?;

        Ok(DashboardData {
            app_usage,
            daily_goals: Vec::new(), // 由 GoalService 提供
            start: today_start,
            end: now,
        })
    }

    /// 获取统计数据（根据时间导航状态）
    pub async fn get_stats_data(&self, state: &TimeNavigationState) -> DbResult<StatsData> {
        let time_range = state.to_time_range();
        let (start, end) = match time_range {
            TimeRange::Today => {
                let local_now = Local::now();
                let today_start = local_now
                    .date_naive()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_local_timezone(Local)
                    .unwrap()
                    .with_timezone(&Utc);
                (today_start, Utc::now())
            }
            TimeRange::Yesterday => {
                let local_yesterday = Local::now() - chrono::Duration::days(1);
                let yesterday_start = local_yesterday
                    .date_naive()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_local_timezone(Local)
                    .unwrap()
                    .with_timezone(&Utc);
                let yesterday_end = local_yesterday
                    .date_naive()
                    .and_hms_opt(23, 59, 59)
                    .unwrap()
                    .and_local_timezone(Local)
                    .unwrap()
                    .with_timezone(&Utc);
                (yesterday_start, yesterday_end)
            }
            TimeRange::Custom(s, e) => (s, e),
            _ => {
                let local_now = Local::now();
                let today_start = local_now
                    .date_naive()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_local_timezone(Local)
                    .unwrap()
                    .with_timezone(&Utc);
                (today_start, Utc::now())
            }
        };

        let app_usage = self.get_app_usage(start, end).await?;
        let category_usage = self.get_category_usage(start, end).await?;

        // 根据导航状态获取时间段数据
        let period_usage = match state.level {
            crate::models::TimeNavigationLevel::Year => {
                self.get_yearly_usage(state.selected_year).await?
            }
            crate::models::TimeNavigationLevel::Month => {
                self.get_monthly_usage(state.selected_year).await?
            }
            crate::models::TimeNavigationLevel::Week => {
                let month = state.selected_month.unwrap_or(1);
                self.get_weekly_usage(state.selected_year, month).await?
            }
            crate::models::TimeNavigationLevel::Day => {
                let month = state.selected_month.unwrap_or(1);
                let week = state.selected_week.unwrap_or(1);
                self.get_daily_usage_for_week(state.selected_year, month, week)
                    .await?
            }
            crate::models::TimeNavigationLevel::Hour => {
                let month = state.selected_month.unwrap_or(1);
                let day = state.selected_day.unwrap_or(1);
                self.get_hourly_usage(state.selected_year, month, day).await?
            }
        };

        Ok(StatsData {
            app_usage,
            category_usage,
            period_usage,
            start,
            end,
        })
    }
}

#[async_trait]
impl AppUsageQuery for UsageServiceImpl {
    async fn get_app_usage(
        &self,
        start: chrono::DateTime<Utc>,
        end: chrono::DateTime<Utc>,
    ) -> DbResult<Vec<AppUsage>> {
        self.app_usage_query.get_app_usage(start, end).await
    }
}

#[async_trait]
impl CategoryUsageQuery for UsageServiceImpl {
    async fn get_category_usage(
        &self,
        start: chrono::DateTime<Utc>,
        end: chrono::DateTime<Utc>,
    ) -> DbResult<Vec<CategoryUsage>> {
        self.category_usage_query.get_category_usage(start, end).await
    }
}

#[async_trait]
impl TimeStatsQuery for UsageServiceImpl {
    async fn get_yearly_usage(&self, years: i32) -> DbResult<Vec<PeriodUsage>> {
        self.time_stats_query.get_yearly_usage(years).await
    }

    async fn get_monthly_usage(&self, year: i32) -> DbResult<Vec<PeriodUsage>> {
        self.time_stats_query.get_monthly_usage(year).await
    }

    async fn get_weekly_usage(&self, year: i32, month: u32) -> DbResult<Vec<PeriodUsage>> {
        self.time_stats_query.get_weekly_usage(year, month).await
    }

    async fn get_daily_usage_for_week(
        &self,
        year: i32,
        month: u32,
        week: u32,
    ) -> DbResult<Vec<PeriodUsage>> {
        self.time_stats_query
            .get_daily_usage_for_week(year, month, week)
            .await
    }

    async fn get_hourly_usage(
        &self,
        year: i32,
        month: u32,
        day: u32,
    ) -> DbResult<Vec<PeriodUsage>> {
        self.time_stats_query.get_hourly_usage(year, month, day).await
    }
}

impl Clone for UsageServiceImpl {
    fn clone(&self) -> Self {
        Self {
            app_usage_query: self.app_usage_query.clone(),
            category_usage_query: self.category_usage_query.clone(),
            time_stats_query: self.time_stats_query.clone(),
        }
    }
}
