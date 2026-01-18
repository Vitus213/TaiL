//! 数据模型转换器
//!
//! 在新的 DTO 和旧模型之间转换

use chrono::{DateTime, Datelike, TimeZone, Timelike, Utc};
use crate::models::{AppUsage, WindowEvent, Category as OldCategory};
use crate::application::ports::{DashboardView, StatsView, Category};
use crate::domain::aggregation::TimeGranularity;

/// 将 DashboardView 转换为 Vec<AppUsage>
pub fn dashboard_view_to_app_usage(view: DashboardView) -> Vec<AppUsage> {
    let mut result = Vec::new();

    for item in view.top_apps {
        // 获取该应用在各个时间桶中的数据
        let mut window_events = Vec::new();

        // 从 hourly_breakdown 中提取该应用的事件数据
        for bucket in &view.hourly_breakdown.buckets {
            if let Some(&duration) = bucket.app_breakdown.get(&item.app_name) {
                if duration > 0 {
                    // 创建一个代表性的窗口事件
                    // 使用 range_start 和桶索引来计算准确的时间戳
                    window_events.push(WindowEvent {
                        id: None,
                        timestamp: estimate_time_for_bucket(view.range_start, bucket.index, TimeGranularity::Hour),
                        app_name: item.app_name.clone(),
                        window_title: String::new(),
                        workspace: String::new(),
                        duration_secs: duration,
                        is_afk: false,
                    });
                }
            }
        }

        result.push(AppUsage {
            app_name: item.app_name,
            total_seconds: item.duration_secs,
            window_events,
        });
    }

    result
}

/// 将 StatsView 转换为 Vec<AppUsage>
pub fn stats_view_to_app_usage(view: StatsView) -> Vec<AppUsage> {
    let mut result = Vec::new();

    for item in view.app_breakdown {
        let mut window_events = Vec::new();

        for bucket in &view.period_breakdown.buckets {
            if let Some(&duration) = bucket.app_breakdown.get(&item.app_name) {
                if duration > 0 {
                    window_events.push(WindowEvent {
                        id: None,
                        timestamp: estimate_time_for_bucket(view.range_start, bucket.index, view.period_breakdown.granularity),
                        app_name: item.app_name.clone(),
                        window_title: String::new(),
                        workspace: String::new(),
                        duration_secs: duration,
                        is_afk: false,
                    });
                }
            }
        }

        result.push(AppUsage {
            app_name: item.app_name,
            total_seconds: item.duration_secs,
            window_events,
        });
    }

    result
}

/// 根据桶索引和时间范围开始时间计算时间戳
fn estimate_time_for_bucket(range_start: DateTime<Utc>, bucket_index: usize, granularity: TimeGranularity) -> DateTime<Utc> {
    match granularity {
        TimeGranularity::Hour => {
            // 小时粒度：bucket_index 是小时 (0-23)
            // 计算当天开始时间
            let local_start = range_start.with_timezone(&chrono::Local);
            let start_of_day = local_start
                .date_naive()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_local_timezone(chrono::Local)
                .unwrap()
                .with_timezone(&Utc);
            start_of_day + chrono::Duration::seconds(bucket_index as i64 * 3600)
        }
        TimeGranularity::Day => {
            // 天粒度：bucket_index 是星期 (0-6, 周一到周日)
            let local_start = range_start.with_timezone(&chrono::Local);
            let weekday = local_start.weekday().num_days_from_monday();
            let start_of_week = local_start
                .date_naive()
                - chrono::Duration::days(weekday as i64)
                - chrono::Duration::days(weekday as i64);  // 确保到周一
            let start_of_week = start_of_week
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_local_timezone(chrono::Local)
                .unwrap()
                .with_timezone(&Utc);
            start_of_week + chrono::Duration::days(bucket_index as i64)
        }
        TimeGranularity::Month => {
            // 月粒度：bucket_index 是月份 (0-11)
            let local_start = range_start.with_timezone(&chrono::Local);
            let current_year = local_start.year();
            chrono::Utc.with_ymd_and_hms(current_year, bucket_index as u32 + 1, 1, 12, 0, 0)
                .unwrap()
        }
        TimeGranularity::Minute => {
            // 分钟粒度
            let local_start = range_start.with_timezone(&chrono::Local);
            let start_of_hour = local_start
                .date_naive()
                .and_hms_opt(local_start.hour(), 0, 0)
                .unwrap()
                .and_local_timezone(chrono::Local)
                .unwrap()
                .with_timezone(&Utc);
            start_of_hour + chrono::Duration::seconds(bucket_index as i64 * 60)
        }
        TimeGranularity::Week | TimeGranularity::Year => range_start,
    }
}

/// 将新的 Category 转换为旧的 Category
pub fn category_to_old(cat: Category) -> OldCategory {
    OldCategory {
        id: Some(cat.id),
        name: cat.name,
        color: cat.color,
        icon: cat.icon,
    }
}

/// 将旧的 Category 转换为新的 Category
pub fn category_from_old(old: OldCategory) -> Category {
    Category {
        id: old.id.unwrap_or(0),
        name: old.name,
        color: old.color,
        icon: old.icon,
    }
}
