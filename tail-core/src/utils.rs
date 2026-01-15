//! TaiL Core - 工具函数模块
//!
//! 提供时间范围计算、时长格式化、数据过滤等通用工具

use chrono::Duration as ChronoDuration;
use chrono::{DateTime, Datelike, NaiveDate, NaiveTime, Utc};

use crate::models::AppUsage;

/// 时间范围计算工具
pub mod time_range {
    use super::*;

    /// 获取今天的开始时间（UTC）
    ///
    /// 返回今天 00:00:00 对应的 UTC 时间
    pub fn today_start() -> DateTime<Utc> {
        Utc::now()
            .date_naive()
            .and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
            .and_utc()
    }

    /// 获取今天的结束时间（UTC）
    ///
    /// 返回今天 23:59:59 对应的 UTC 时间
    pub fn today_end() -> DateTime<Utc> {
        Utc::now()
            .date_naive()
            .and_time(NaiveTime::from_hms_milli_opt(23, 59, 59, 999).unwrap())
            .and_utc()
    }

    /// 获取指定日期的开始时间（UTC）
    pub fn day_start(date: NaiveDate) -> DateTime<Utc> {
        date.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
            .and_utc()
    }

    /// 获取指定日期的结束时间（UTC）
    pub fn day_end(date: NaiveDate) -> DateTime<Utc> {
        date.and_time(NaiveTime::from_hms_milli_opt(23, 59, 59, 999).unwrap())
            .and_utc()
    }

    /// 获取指定月份的范围
    ///
    /// 返回 (月初 00:00:00, 月末 23:59:59.999)
    pub fn month_range(year: i32, month: u32) -> (DateTime<Utc>, DateTime<Utc>) {
        let start = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        // 计算月末日期
        let end_date = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap() - ChronoDuration::days(1)
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap() - ChronoDuration::days(1)
        };
        (day_start(start), day_end(end_date))
    }

    /// 获取指定年份的范围
    ///
    /// 返回 (1月1日 00:00:00, 12月31日 23:59:59)
    pub fn year_range(year: i32) -> (DateTime<Utc>, DateTime<Utc>) {
        let start = NaiveDate::from_ymd_opt(year, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(year, 12, 31).unwrap();
        (day_start(start), day_end(end))
    }

    /// 获取指定月份中第几周的范围
    ///
    /// 返回 (周一 00:00:00, 周日 23:59:59)
    pub fn week_range(year: i32, month: u32, week: u32) -> (DateTime<Utc>, DateTime<Utc>) {
        let first_day = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        let first_weekday = first_day.weekday().num_days_from_monday();

        // 计算该周的第一天（周一）
        let week_start_day = ((week - 1) * 7) as i64 - first_weekday as i64 + 1;
        let week_start = if week_start_day < 1 {
            first_day
        } else {
            first_day + ChronoDuration::days(week_start_day - 1)
        };

        let week_end = week_start + ChronoDuration::days(6);

        (day_start(week_start), day_end(week_end))
    }

    /// 获取指定月份的天数
    pub fn days_in_month(year: i32, month: u32) -> u32 {
        if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap()
        }
        .signed_duration_since(NaiveDate::from_ymd_opt(year, month, 1).unwrap())
        .num_days() as u32
    }

    /// 计算某日期是该月的第几周
    pub fn week_of_month(year: i32, month: u32, day: u32) -> u32 {
        let first_day = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        let first_weekday = first_day.weekday().num_days_from_monday();
        ((day + first_weekday - 1) / 7) + 1
    }
}

/// 时长格式化工具
pub mod duration {
    /// 时长格式风格
    #[derive(Debug, Clone, Copy)]
    pub enum DurationStyle {
        /// 简短格式：1h 30m
        Short,
        /// 完整格式：1h 30m 15s
        Full,
        /// 中文格式：1小时30分钟
        Chinese,
        /// 极简格式：1h
        Minimal,
    }

    /// 格式化秒数为可读时长
    ///
    /// # 示例
    /// ```
    /// use tail_core::utils::duration::{format_duration, DurationStyle};
    ///
    /// assert_eq!(format_duration(3665, DurationStyle::Short), "1h 1m");
    /// assert_eq!(format_duration(3665, DurationStyle::Full), "1h 1m 5s");
    /// assert_eq!(format_duration(3665, DurationStyle::Chinese), "1小时1分钟");
    /// assert_eq!(format_duration(3665, DurationStyle::Minimal), "1h");
    /// ```
    pub fn format_duration(seconds: i64, style: DurationStyle) -> String {
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        let secs = seconds % 60;

        match style {
            DurationStyle::Short => {
                if hours > 0 {
                    format!("{}h {}m", hours, minutes)
                } else if minutes > 0 {
                    format!("{}m {}s", minutes, secs)
                } else {
                    format!("{}s", secs)
                }
            }
            DurationStyle::Full => {
                if hours > 0 {
                    format!("{}h {}m {}s", hours, minutes, secs)
                } else if minutes > 0 {
                    format!("{}m {}s", minutes, secs)
                } else {
                    format!("{}s", secs)
                }
            }
            DurationStyle::Chinese => {
                if hours > 0 {
                    format!("{}小时{}分钟", hours, minutes)
                } else {
                    format!("{}分钟", minutes)
                }
            }
            DurationStyle::Minimal => {
                if hours > 0 {
                    format!("{}h", hours)
                } else {
                    format!("{}m", minutes)
                }
            }
        }
    }

    /// 从分钟格式化（用于目标进度等场景）
    pub fn format_minutes(minutes: i32, style: DurationStyle) -> String {
        format_duration(minutes as i64 * 60, style)
    }
}

/// 数据过滤工具
pub mod filter {
    use super::*;

    /// 过滤掉空名称的应用，返回引用
    pub fn filter_empty_apps(apps: &[AppUsage]) -> Vec<&AppUsage> {
        apps.iter().filter(|u| !u.app_name.is_empty()).collect()
    }

    /// 过滤掉空名称的应用，返回所有权
    pub fn filter_empty_apps_owned(apps: Vec<AppUsage>) -> Vec<AppUsage> {
        apps.into_iter()
            .filter(|u| !u.app_name.is_empty())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use duration::{format_duration, DurationStyle};
    use time_range::{days_in_month, month_range, year_range};

    #[test]
    fn test_format_duration_short() {
        assert_eq!(format_duration(0, DurationStyle::Short), "0s");
        assert_eq!(format_duration(59, DurationStyle::Short), "59s");
        assert_eq!(format_duration(60, DurationStyle::Short), "1m 0s");
        assert_eq!(format_duration(3665, DurationStyle::Short), "1h 1m");
    }

    #[test]
    fn test_format_duration_full() {
        assert_eq!(format_duration(3665, DurationStyle::Full), "1h 1m 5s");
        assert_eq!(format_duration(60, DurationStyle::Full), "1m 0s");
    }

    #[test]
    fn test_format_duration_chinese() {
        assert_eq!(format_duration(3665, DurationStyle::Chinese), "1小时1分钟");
        assert_eq!(format_duration(60, DurationStyle::Chinese), "1分钟");
    }

    #[test]
    fn test_format_duration_minimal() {
        assert_eq!(format_duration(3665, DurationStyle::Minimal), "1h");
        assert_eq!(format_duration(300, DurationStyle::Minimal), "5m");
    }

    #[test]
    fn test_year_range() {
        let (start, end) = year_range(2024);
        assert_eq!(start.year(), 2024);
        assert_eq!(start.month(), 1);
        assert_eq!(start.day(), 1);
        assert_eq!(end.year(), 2024);
        assert_eq!(end.month(), 12);
        assert_eq!(end.day(), 31);
    }

    #[test]
    fn test_month_range() {
        let (start, end) = month_range(2024, 2);
        assert_eq!(start.year(), 2024);
        assert_eq!(start.month(), 2);
        assert_eq!(start.day(), 1);
        assert_eq!(end.year(), 2024);
        assert_eq!(end.month(), 2);
        assert_eq!(end.day(), 29); // 2024 是闰年
    }

    #[test]
    fn test_days_in_month() {
        assert_eq!(days_in_month(2024, 2), 29); // 闰年
        assert_eq!(days_in_month(2023, 2), 28);
        assert_eq!(days_in_month(2024, 1), 31);
        assert_eq!(days_in_month(2024, 4), 30);
    }
}
