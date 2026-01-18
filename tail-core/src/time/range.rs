//! 时间范围计算
//!
//! 提供准确的时间范围计算，确保边界一致性
//!
//! # 设计原则
//!
//! 1. **明确的边界**: 所有时间范围都是闭区间 [start, end]
//! 2. **本地时间优先**: 所有计算基于本地时间，存储时再转为UTC
//! 3. **周一起始**: 一周从周一开始，到周日结束

use chrono::{DateTime, Datelike, Local, NaiveDate, NaiveTime, Utc};

/// 时间范围
///
/// 表示一个闭区间的时间范围 [start, end]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeRange {
    /// 开始时间（UTC）
    pub start: DateTime<Utc>,
    /// 结束时间（UTC）
    pub end: DateTime<Utc>,
}

impl TimeRange {
    /// 创建新的时间范围
    ///
    /// # Panics
    /// 如果 start > end
    pub fn new(start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        assert!(start <= end, "时间范围的开始必须早于或等于结束");
        Self { start, end }
    }

    /// 检查是否包含某个时间点
    pub fn contains(&self, timestamp: DateTime<Utc>) -> bool {
        timestamp >= self.start && timestamp <= self.end
    }

    /// 计算时间跨度（秒）
    pub fn duration_seconds(&self) -> i64 {
        (self.end - self.start).num_seconds().max(0)
    }

    /// 转换为本地时间的字符串表示
    pub fn to_local_string(&self) -> String {
        let start_local = self.start.with_timezone(&Local);
        let end_local = self.end.with_timezone(&Local);
        format!(
            "{} ~ {}",
            start_local.format("%Y-%m-%d %H:%M"),
            end_local.format("%Y-%m-%d %H:%M")
        )
    }
}

/// 时间范围计算器
///
/// 提供各种时间范围的计算方法
pub struct TimeRangeCalculator;

impl TimeRangeCalculator {
    /// 获取今天的范围（本地时间）
    ///
    /// 返回 [今天 00:00:00, 今天 23:59:59.999]
    pub fn today() -> TimeRange {
        let now = Local::now();
        let start = now
            .date_naive()
            .and_time(NaiveTime::from_hms_milli_opt(0, 0, 0, 0).unwrap())
            .and_local_timezone(Local)
            .unwrap()
            .with_timezone(&Utc);
        let end = now
            .date_naive()
            .and_time(NaiveTime::from_hms_milli_opt(23, 59, 59, 999).unwrap())
            .and_local_timezone(Local)
            .unwrap()
            .with_timezone(&Utc);
        TimeRange::new(start, end)
    }

    /// 获取昨天的范围（本地时间）
    pub fn yesterday() -> TimeRange {
        let now = Local::now();
        let yesterday = now.date_naive() - chrono::Duration::days(1);
        let start = yesterday
            .and_time(NaiveTime::from_hms_milli_opt(0, 0, 0, 0).unwrap())
            .and_local_timezone(Local)
            .unwrap()
            .with_timezone(&Utc);
        let end = yesterday
            .and_time(NaiveTime::from_hms_milli_opt(23, 59, 59, 999).unwrap())
            .and_local_timezone(Local)
            .unwrap()
            .with_timezone(&Utc);
        TimeRange::new(start, end)
    }

    /// 获取本周的范围（本地时间，周一到周日）
    ///
    /// 返回 [本周一 00:00:00, 本周日 23:59:59.999]
    pub fn this_week() -> TimeRange {
        let now = Local::now();
        let weekday_offset = now.date_naive().weekday().num_days_from_monday() as i64;
        let monday = now.date_naive() - chrono::Duration::days(weekday_offset);
        let sunday = monday + chrono::Duration::days(6);

        let start = monday
            .and_time(NaiveTime::from_hms_milli_opt(0, 0, 0, 0).unwrap())
            .and_local_timezone(Local)
            .unwrap()
            .with_timezone(&Utc);
        let end = sunday
            .and_time(NaiveTime::from_hms_milli_opt(23, 59, 59, 999).unwrap())
            .and_local_timezone(Local)
            .unwrap()
            .with_timezone(&Utc);
        TimeRange::new(start, end)
    }

    /// 获取本月的范围（本地时间）
    ///
    /// 返回 [本月1日 00:00:00, 本月最后一天 23:59:59.999]
    pub fn this_month() -> TimeRange {
        let now = Local::now();
        Self::month_range(now.year(), now.month())
    }

    /// 获取本年的范围（本地时间）
    ///
    /// 返回 [本年1月1日 00:00:00, 本年12月31日 23:59:59.999]
    pub fn this_year() -> TimeRange {
        let now = Local::now();
        Self::year_range(now.year())
    }

    /// 获取指定日期的范围
    ///
    /// 返回 [该日 00:00:00, 该日 23:59:59.999]
    pub fn day(date: NaiveDate) -> TimeRange {
        let start = date
            .and_time(NaiveTime::from_hms_milli_opt(0, 0, 0, 0).unwrap())
            .and_local_timezone(Local)
            .unwrap()
            .with_timezone(&Utc);
        let end = date
            .and_time(NaiveTime::from_hms_milli_opt(23, 59, 59, 999).unwrap())
            .and_local_timezone(Local)
            .unwrap()
            .with_timezone(&Utc);
        TimeRange::new(start, end)
    }

    /// 获取指定月份的范围
    ///
    /// 返回 [该月1日 00:00:00, 该月最后一天 23:59:59.999]
    pub fn month_range(year: i32, month: u32) -> TimeRange {
        let first_day = NaiveDate::from_ymd_opt(year, month, 1).unwrap();

        // 计算月末日期
        let last_day = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap() - chrono::Duration::days(1)
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap() - chrono::Duration::days(1)
        };

        let start = first_day
            .and_time(NaiveTime::from_hms_milli_opt(0, 0, 0, 0).unwrap())
            .and_local_timezone(Local)
            .unwrap()
            .with_timezone(&Utc);
        let end = last_day
            .and_time(NaiveTime::from_hms_milli_opt(23, 59, 59, 999).unwrap())
            .and_local_timezone(Local)
            .unwrap()
            .with_timezone(&Utc);
        TimeRange::new(start, end)
    }

    /// 获取指定年份的范围
    ///
    /// 返回 [该年1月1日 00:00:00, 该年12月31日 23:59:59.999]
    pub fn year_range(year: i32) -> TimeRange {
        let first_day = NaiveDate::from_ymd_opt(year, 1, 1).unwrap();
        let last_day = NaiveDate::from_ymd_opt(year, 12, 31).unwrap();

        let start = first_day
            .and_time(NaiveTime::from_hms_milli_opt(0, 0, 0, 0).unwrap())
            .and_local_timezone(Local)
            .unwrap()
            .with_timezone(&Utc);
        let end = last_day
            .and_time(NaiveTime::from_hms_milli_opt(23, 59, 59, 999).unwrap())
            .and_local_timezone(Local)
            .unwrap()
            .with_timezone(&Utc);
        TimeRange::new(start, end)
    }

    /// 获取指定月份中第几周的范围
    ///
    /// 返回 [该周周一 00:00:00, 该周周日 23:59:59.999]
    ///
    /// # 参数
    /// - `year`: 年份
    /// - `month`: 月份（1-12）
    /// - `week`: 周数（从1开始）
    ///
    /// # 注意
    /// 周的计算基于该月1日是周几来确定：
    /// - 如果1日是周三，则第1周从1日开始到周日结束
    /// - 第2周从下一个周一开始
    pub fn week_in_month(year: i32, month: u32, week: u32) -> TimeRange {
        let first_day = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        let first_weekday = first_day.weekday().num_days_from_monday();

        // 计算该周的第一天（周一）
        // week=1: 如果1日不是周一，则第1周从1日开始
        // week>1: 从对应的周一开始
        let week_start = if week == 1 {
            first_day
        } else {
            // 计算第2周及之后的周一开始
            let days_to_first_monday = if first_weekday == 0 {
                0 // 1日就是周一
            } else {
                7 - first_weekday as i64 // 下一个周一
            };
            let first_monday = first_day + chrono::Duration::days(days_to_first_monday);
            let target_week_offset = (week as i64 - 2) * 7;
            first_monday + chrono::Duration::days(target_week_offset)
        };

        // 周日 = 周一 + 6天
        let week_end = week_start + chrono::Duration::days(6);

        // 确保不超出该月范围
        let month_end = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap() - chrono::Duration::days(1)
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap() - chrono::Duration::days(1)
        };

        let actual_end = week_end.min(month_end);

        let start = week_start
            .and_time(NaiveTime::from_hms_milli_opt(0, 0, 0, 0).unwrap())
            .and_local_timezone(Local)
            .unwrap()
            .with_timezone(&Utc);
        let end = actual_end
            .and_time(NaiveTime::from_hms_milli_opt(23, 59, 59, 999).unwrap())
            .and_local_timezone(Local)
            .unwrap()
            .with_timezone(&Utc);
        TimeRange::new(start, end)
    }

    /// 计算某日期是该月的第几周
    ///
    /// # 返回
    /// 周数（从1开始）
    pub fn week_of_month(year: i32, month: u32, day: u32) -> u32 {
        let _date = NaiveDate::from_ymd_opt(year, month, day).unwrap();
        let first_day = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        let first_weekday = first_day.weekday().num_days_from_monday();
        let day_offset = day as i64 - 1;

        // 第1周：从1日到第一个周日
        // 第2周及之后：从周一开始
        let days_in_first_week = 7 - first_weekday as i64;

        if day_offset < days_in_first_week {
            1
        } else {
            let remaining_days = day_offset - days_in_first_week;
            2 + (remaining_days / 7) as u32
        }
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

    /// 获取最近N天的范围（包括今天）
    pub fn last_n_days(n: u32) -> TimeRange {
        let now = Local::now();
        let end = now
            .date_naive()
            .and_time(NaiveTime::from_hms_milli_opt(23, 59, 59, 999).unwrap())
            .and_local_timezone(Local)
            .unwrap()
            .with_timezone(&Utc);
        let start_date = now.date_naive() - chrono::Duration::days(n as i64 - 1);
        let start = start_date
            .and_time(NaiveTime::from_hms_milli_opt(0, 0, 0, 0).unwrap())
            .and_local_timezone(Local)
            .unwrap()
            .with_timezone(&Utc);
        TimeRange::new(start, end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Weekday;

    #[test]
    fn test_today_range() {
        let range = TimeRangeCalculator::today();
        assert!(range.start < range.end);
        // 验证范围跨度约为24小时
        let duration = range.duration_seconds();
        assert!(duration >= 86399 && duration <= 86401); // 考虑闰秒
    }

    #[test]
    fn test_month_range() {
        let range = TimeRangeCalculator::month_range(2024, 2);
        // 2024年是闰年，2月有29天
        assert_eq!(range.duration_seconds() / 86400, 28); // 29天 - 1 = 28个完整的24小时
    }

    #[test]
    fn test_year_range() {
        let range = TimeRangeCalculator::year_range(2024);
        // 2024年是闰年，366天
        assert_eq!(range.duration_seconds() / 86400, 365); // 366天 - 1 = 365
    }

    #[test]
    fn test_days_in_month() {
        assert_eq!(TimeRangeCalculator::days_in_month(2024, 2), 29); // 闰年
        assert_eq!(TimeRangeCalculator::days_in_month(2023, 2), 28); // 平年
        assert_eq!(TimeRangeCalculator::days_in_month(2024, 1), 31);
        assert_eq!(TimeRangeCalculator::days_in_month(2024, 4), 30);
    }

    #[test]
    fn test_week_of_month() {
        // 2024年1月1日是周一
        assert_eq!(TimeRangeCalculator::week_of_month(2024, 1, 1), 1); // 周一
        assert_eq!(TimeRangeCalculator::week_of_month(2024, 1, 7), 1); // 周日
        assert_eq!(TimeRangeCalculator::week_of_month(2024, 1, 8), 2); // 下周一
    }

    #[test]
    fn test_range_contains() {
        let range = TimeRangeCalculator::day(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
        let start = range.start;
        let end = range.end;
        let mid = start + chrono::Duration::seconds(3600);

        assert!(range.contains(start));
        assert!(range.contains(end));
        assert!(range.contains(mid));

        let outside = start - chrono::Duration::seconds(1);
        assert!(!range.contains(outside));
    }

    #[test]
    fn test_this_week_is_monday_to_sunday() {
        let range = TimeRangeCalculator::this_week();
        let start_local = range.start.with_timezone(&Local);
        let end_local = range.end.with_timezone(&Local);

        // 验证开始是周一
        assert_eq!(start_local.weekday(), Weekday::Mon);
        // 验证结束是周日
        assert_eq!(end_local.weekday(), Weekday::Sun);
    }
}
