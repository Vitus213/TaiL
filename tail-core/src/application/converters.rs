//! 数据模型转换器
//!
//! 在新的 DTO 和旧模型之间转换

use chrono::{DateTime, Datelike, TimeZone, Utc};
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
                    // 注意：这里我们丢失了详细的窗口标题等信息
                    // 如果需要完整信息，应该从原始事件数据中获取
                    window_events.push(WindowEvent {
                        id: None,
                        timestamp: estimate_time_for_bucket(&view.time_range, bucket.index, TimeGranularity::Hour),
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
                        timestamp: estimate_time_for_bucket(&view.time_range, bucket.index, view.period_breakdown.granularity),
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

/// 根据桶索引和时间范围估计时间戳
fn estimate_time_for_bucket(_time_range_str: &str, bucket_index: usize, granularity: TimeGranularity) -> DateTime<Utc> {
    let now = Utc::now();

    match granularity {
        TimeGranularity::Hour => {
            // 小时粒度：bucket_index 是小时 (0-23)
            let local_now = chrono::Local::now();
            let start_of_day = local_now
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
            let local_now = chrono::Local::now();
            let weekday = local_now.weekday().num_days_from_monday();
            let days_diff = bucket_index as i64 - weekday as i64;
            now + chrono::Duration::days(days_diff)
        }
        TimeGranularity::Month => {
            // 月粒度：bucket_index 是月份 (0-11)
            let local_now = chrono::Local::now();
            let current_year = local_now.year();
            chrono::Utc.with_ymd_and_hms(current_year, bucket_index as u32 + 1, 1, 12, 0, 0)
                .unwrap()
        }
        TimeGranularity::Minute => {
            // 分钟粒度
            now + chrono::Duration::seconds(bucket_index as i64 * 60)
        }
        TimeGranularity::Week | TimeGranularity::Year => now,
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
