//! TaiL Core - 核心 Trait 定义

use crate::errors::{DbError, DbResult};
use crate::models::*;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

// ============================================================================
// 窗口事件仓储
// ============================================================================

/// 窗口事件仓储
#[async_trait]
pub trait WindowEventRepository: Send + Sync {
    /// 插入窗口事件
    async fn insert(&self, event: &WindowEvent) -> DbResult<i64>;

    /// 获取时间范围内的窗口事件
    async fn get_by_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> DbResult<Vec<WindowEvent>>;

    /// 更新窗口事件时长
    async fn update_duration(&self, id: i64, duration_secs: i64) -> DbResult<()>;
}

// ============================================================================
// AFK 事件仓储
// ============================================================================

/// AFK 事件仓储
#[async_trait]
pub trait AfkEventRepository: Send + Sync {
    /// 插入 AFK 事件
    async fn insert(&self, event: &AfkEvent) -> DbResult<i64>;

    /// 更新 AFK 事件结束时间
    async fn update_end(
        &self,
        id: i64,
        end_time: DateTime<Utc>,
        duration_secs: i64,
    ) -> DbResult<()>;

    /// 获取时间范围内的 AFK 事件
    async fn get_by_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> DbResult<Vec<AfkEvent>>;
}

// ============================================================================
// 每日目标仓储
// ============================================================================

/// 每日目标仓储
#[async_trait]
pub trait DailyGoalRepository: Send + Sync {
    /// 插入或更新每日目标
    async fn upsert(&self, goal: &DailyGoal) -> DbResult<i64>;

    /// 获取所有每日目标
    async fn get_all(&self) -> DbResult<Vec<DailyGoal>>;

    /// 删除每日目标
    async fn delete(&self, app_name: &str) -> DbResult<()>;

    /// 获取今日某应用的总使用时长
    async fn get_today_usage(&self, app_name: &str) -> DbResult<i64>;
}

// ============================================================================
// 分类仓储
// ============================================================================

/// 分类仓储
#[async_trait]
pub trait CategoryRepository: Send + Sync {
    /// 插入新分类
    async fn insert(&self, category: &Category) -> DbResult<i64>;

    /// 更新分类
    async fn update(&self, category: &Category) -> DbResult<()>;

    /// 删除分类
    async fn delete(&self, id: i64) -> DbResult<()>;

    /// 获取所有分类
    async fn get_all(&self) -> DbResult<Vec<Category>>;

    /// 根据 ID 获取分类
    async fn get_by_id(&self, id: i64) -> DbResult<Option<Category>>;

    /// 获取应用所属的所有分类
    async fn get_app_categories(&self, app_name: &str) -> DbResult<Vec<Category>>;

    /// 获取分类下的所有应用名称
    async fn get_category_apps(&self, category_id: i64) -> DbResult<Vec<String>>;

    /// 将应用添加到分类
    async fn add_app_to_category(&self, app_name: &str, category_id: i64) -> DbResult<()>;

    /// 从分类中移除应用
    async fn remove_app_from_category(
        &self,
        app_name: &str,
        category_id: i64,
    ) -> DbResult<()>;

    /// 设置应用的分类（替换所有现有分类）
    async fn set_app_categories(&self, app_name: &str, category_ids: &[i64]) -> DbResult<()>;

    /// 获取所有已记录的应用名称
    async fn get_all_app_names(&self) -> DbResult<Vec<String>>;
}

// ============================================================================
// 别名仓储
// ============================================================================

/// 别名仓储
#[async_trait]
pub trait AliasRepository: Send + Sync {
    /// 设置应用别名
    async fn set(&self, app_name: &str, alias: &str) -> DbResult<()>;

    /// 获取应用别名
    async fn get(&self, app_name: &str) -> DbResult<Option<String>>;

    /// 获取所有应用别名
    async fn get_all(&self) -> DbResult<Vec<(String, String)>>;

    /// 删除应用别名
    async fn delete(&self, app_name: &str) -> DbResult<()>;
}

// ============================================================================
// 查询服务
// ============================================================================

/// 应用使用查询
#[async_trait]
pub trait AppUsageQuery: Send + Sync {
    /// 获取应用使用统计
    async fn get_app_usage(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> DbResult<Vec<AppUsage>>;
}

/// 分类使用查询
#[async_trait]
pub trait CategoryUsageQuery: Send + Sync {
    /// 获取分类使用统计
    async fn get_category_usage(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> DbResult<Vec<CategoryUsage>>;
}

/// 时间统计查询
#[async_trait]
pub trait TimeStatsQuery: Send + Sync {
    /// 获取按年份汇总的使用统计
    async fn get_yearly_usage(&self, years: i32) -> DbResult<Vec<PeriodUsage>>;

    /// 获取某年按月份汇总的使用统计
    async fn get_monthly_usage(&self, year: i32) -> DbResult<Vec<PeriodUsage>>;

    /// 获取某年某月按周汇总的使用统计
    async fn get_weekly_usage(&self, year: i32, month: u32) -> DbResult<Vec<PeriodUsage>>;

    /// 获取某年某月某周按天汇总的使用统计
    async fn get_daily_usage_for_week(
        &self,
        year: i32,
        month: u32,
        week: u32,
    ) -> DbResult<Vec<PeriodUsage>>;

    /// 获取某天按小时汇总的使用统计
    async fn get_hourly_usage(
        &self,
        year: i32,
        month: u32,
        day: u32,
    ) -> DbResult<Vec<PeriodUsage>>;
}
