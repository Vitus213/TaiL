//! 时间序列聚合算法
//!
//! 将原始事件数据按时间粒度聚合为时间槽。

use chrono::{Datelike, Local};
use std::collections::HashMap;
use std::fmt;

use super::time_event::TimeEvent;

/// 时间粒度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeGranularity {
    /// 分钟粒度 (60个桶)
    Minute,
    /// 小时粒度 (24个桶)
    Hour,
    /// 天粒度 (7个桶)
    Day,
    /// 周粒度 (动态)
    Week,
    /// 月粒度 (12个桶)
    Month,
    /// 年粒度
    Year,
}

impl TimeGranularity {
    /// 获取该粒度的桶数量
    pub fn bucket_count(&self) -> usize {
        match self {
            TimeGranularity::Minute => 60,
            TimeGranularity::Hour => 24,
            TimeGranularity::Day => 7,
            TimeGranularity::Week => 6,
            TimeGranularity::Month => 12,
            TimeGranularity::Year => 10,
        }
    }

    /// 获取默认标签
    pub fn default_label(&self, index: usize) -> String {
        match self {
            TimeGranularity::Minute => format!("{}分", index),
            TimeGranularity::Hour => format!("{}时", index),
            TimeGranularity::Day => {
                ["周一", "周二", "周三", "周四", "周五", "周六", "周日"][index].to_string()
            }
            TimeGranularity::Week => format!("第{}周", index + 1),
            TimeGranularity::Month => {
                [
                    "1月", "2月", "3月", "4月", "5月", "6月", "7月", "8月", "9月", "10月",
                    "11月", "12月",
                ][index]
                .to_string()
            }
            TimeGranularity::Year => format!("{}年", index),
        }
    }
}

impl fmt::Display for TimeGranularity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TimeGranularity::Minute => write!(f, "分钟"),
            TimeGranularity::Hour => write!(f, "小时"),
            TimeGranularity::Day => write!(f, "天"),
            TimeGranularity::Week => write!(f, "周"),
            TimeGranularity::Month => write!(f, "月"),
            TimeGranularity::Year => write!(f, "年"),
        }
    }
}

/// 聚合桶
#[derive(Debug, Clone)]
pub struct Bucket {
    /// 桶标签
    pub label: String,
    /// 桶索引
    pub index: usize,
    /// 总持续时间（秒）
    pub total_seconds: i64,
    /// 应用分组数据
    pub app_breakdown: HashMap<String, i64>,
}

impl Bucket {
    /// 创建新桶
    pub fn new(label: impl Into<String>, index: usize) -> Self {
        Self {
            label: label.into(),
            index,
            total_seconds: 0,
            app_breakdown: HashMap::new(),
        }
    }

    /// 添加事件到桶
    pub fn add_event(&mut self, app_name: &str, duration_secs: i64) {
        self.total_seconds += duration_secs;
        *self.app_breakdown.entry(app_name.to_string()).or_insert(0) += duration_secs;
    }

    /// 获取指定应用的持续时间
    pub fn get_app_duration(&self, app_name: &str) -> i64 {
        self.app_breakdown.get(app_name).copied().unwrap_or(0)
    }

    /// 获取标签
    pub fn label(&self) -> &str {
        &self.label
    }

    /// 获取索引
    pub fn index(&self) -> usize {
        self.index
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.total_seconds == 0
    }
}

/// 聚合结果
#[derive(Debug, Clone)]
pub struct AggregationResult {
    /// 时间粒度
    pub granularity: TimeGranularity,
    /// 桶列表
    pub buckets: Vec<Bucket>,
    /// 总持续时间（秒）
    pub total_seconds: i64,
}

impl AggregationResult {
    /// 创建新的聚合结果
    pub fn new(granularity: TimeGranularity) -> Self {
        Self {
            granularity,
            buckets: Vec::new(),
            total_seconds: 0,
        }
    }

    /// 添加桶
    pub fn add_bucket(&mut self, bucket: Bucket) {
        self.total_seconds += bucket.total_seconds;
        self.buckets.push(bucket);
    }

    /// 设置桶列表
    pub fn set_buckets(&mut self, buckets: Vec<Bucket>) {
        self.total_seconds = buckets.iter().map(|b| b.total_seconds).sum();
        self.buckets = buckets;
    }

    /// 获取总持续时间
    pub fn total_duration(&self) -> i64 {
        self.total_seconds
    }

    /// 获取前 N 个应用
    pub fn top_apps(&self, limit: usize) -> Vec<(String, i64)> {
        let mut all_apps: HashMap<String, i64> = HashMap::new();

        for bucket in &self.buckets {
            for (app, duration) in &bucket.app_breakdown {
                *all_apps.entry(app.clone()).or_insert(0) += duration;
            }
        }

        let mut apps: Vec<_> = all_apps.into_iter().collect();
        apps.sort_by(|a, b| b.1.cmp(&a.1));
        apps.truncate(limit);
        apps
    }

    /// 按应用分组聚合
    pub fn by_app(&self) -> HashMap<String, i64> {
        let mut result: HashMap<String, i64> = HashMap::new();

        for bucket in &self.buckets {
            for (app, duration) in &bucket.app_breakdown {
                *result.entry(app.clone()).or_insert(0) += duration;
            }
        }

        result
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.total_seconds == 0
    }

    /// 获取最大值
    pub fn max_bucket_value(&self) -> i64 {
        self.buckets
            .iter()
            .map(|b| b.total_seconds)
            .max()
            .unwrap_or(0)
    }
}

/// 时间序列分析器 - 领域服务
///
/// 提供时间序列数据的聚合功能
pub struct TimeSeriesAnalyzer;

impl TimeSeriesAnalyzer {
    /// 聚合时间序列数据
    ///
    /// # 参数
    /// - `events`: 事件列表
    /// - `granularity`: 时间粒度
    /// - `range`: 可选的时间范围过滤器
    ///
    /// # 返回
    /// 聚合结果
    pub fn aggregate(
        events: &[TimeEvent],
        granularity: TimeGranularity,
        range: Option<&TimeRange>,
    ) -> AggregationResult {
        // 先过滤时间范围和 AFK 事件
        let filtered: Vec<&TimeEvent> = events
            .iter()
            .filter(|e| !e.is_afk)
            .filter(|e| range.map_or(true, |r| r.contains(&e.timestamp)))
            .collect();

        match granularity {
            TimeGranularity::Minute => Self::aggregate_by_minute(&filtered),
            TimeGranularity::Hour => Self::aggregate_by_hour(&filtered),
            TimeGranularity::Day => Self::aggregate_by_day(&filtered),
            TimeGranularity::Week => Self::aggregate_by_week(&filtered),
            TimeGranularity::Month => Self::aggregate_by_month(&filtered),
            TimeGranularity::Year => Self::aggregate_by_year(&filtered),
        }
    }

    /// 按分钟聚合（60个桶）
    fn aggregate_by_minute(events: &[&TimeEvent]) -> AggregationResult {
        let mut result = AggregationResult::new(TimeGranularity::Minute);

        // 预创建60个桶
        for i in 0..60 {
            result.add_bucket(Bucket::new(format!("{}分", i), i));
        }

        for event in events {
            let minute = event.minute_index();
            if minute < result.buckets.len() {
                result.buckets[minute].add_event(event.app_name.as_str(), event.duration_secs());
            }
        }

        // 重新计算总时长
        result.total_seconds = result.buckets.iter().map(|b| b.total_seconds).sum();

        result
    }

    /// 按小时聚合（24个桶）
    fn aggregate_by_hour(events: &[&TimeEvent]) -> AggregationResult {
        let mut result = AggregationResult::new(TimeGranularity::Hour);

        // 预创建24个桶
        for i in 0..24 {
            result.add_bucket(Bucket::new(format!("{}时", i), i));
        }

        for event in events {
            let hour = event.hour_index();
            if hour < result.buckets.len() {
                result.buckets[hour].add_event(event.app_name.as_str(), event.duration_secs());
            }
        }

        // 重新计算总时长（因为桶的时长在 add_event 后更新了）
        result.total_seconds = result.buckets.iter().map(|b| b.total_seconds).sum();

        result
    }

    /// 按天聚合（7个桶，周一到周日）
    fn aggregate_by_day(events: &[&TimeEvent]) -> AggregationResult {
        let labels = ["周一", "周二", "周三", "周四", "周五", "周六", "周日"];
        let mut result = AggregationResult::new(TimeGranularity::Day);

        for (i, label) in labels.iter().enumerate() {
            result.add_bucket(Bucket::new(*label, i));
        }

        for event in events {
            let weekday = event.weekday_index();
            if weekday < result.buckets.len() {
                result.buckets[weekday]
                    .add_event(event.app_name.as_str(), event.duration_secs());
            }
        }

        // 重新计算总时长
        result.total_seconds = result.buckets.iter().map(|b| b.total_seconds).sum();

        result
    }

    /// 按周聚合（动态桶数）
    fn aggregate_by_week(events: &[&TimeEvent]) -> AggregationResult {
        let mut result = AggregationResult::new(TimeGranularity::Week);
        let mut week_map: HashMap<u32, Bucket> = HashMap::new();

        for event in events {
            let local = event.timestamp.with_timezone(&Local);
            let week = Self::week_of_month(local.year(), local.month(), local.day());

            week_map
                .entry(week)
                .or_insert_with(|| Bucket::new(format!("第{}周", week), (week - 1) as usize))
                .add_event(event.app_name.as_str(), event.duration_secs());
        }

        let mut weeks: Vec<_> = week_map.into_values().collect();
        weeks.sort_by_key(|b| b.index);
        result.set_buckets(weeks);

        result
    }

    /// 按月聚合（12个桶）
    fn aggregate_by_month(events: &[&TimeEvent]) -> AggregationResult {
        let labels = [
            "1月", "2月", "3月", "4月", "5月", "6月", "7月", "8月", "9月", "10月", "11月",
            "12月",
        ];
        let mut result = AggregationResult::new(TimeGranularity::Month);

        for (i, label) in labels.iter().enumerate() {
            result.add_bucket(Bucket::new(*label, i));
        }

        for event in events {
            let month = event.month_index();
            if month >= 1 && month <= 12 {
                result.buckets[month - 1]
                    .add_event(event.app_name.as_str(), event.duration_secs());
            }
        }

        // 重新计算总时长
        result.total_seconds = result.buckets.iter().map(|b| b.total_seconds).sum();

        result
    }

    /// 按年聚合（动态桶数）
    fn aggregate_by_year(events: &[&TimeEvent]) -> AggregationResult {
        let mut result = AggregationResult::new(TimeGranularity::Year);
        let mut year_map: HashMap<i32, Bucket> = HashMap::new();

        for event in events {
            let year = event.timestamp.with_timezone(&Local).year();

            year_map
                .entry(year)
                .or_insert_with(|| Bucket::new(format!("{}年", year), 0))
                .add_event(event.app_name.as_str(), event.duration_secs());
        }

        let mut years: Vec<_> = year_map.into_values().collect();
        years.sort_by_key(|b| b.label.clone());
        result.set_buckets(years);

        result
    }

    /// 计算某日是该月的第几周
    ///
    /// 规则：每月1日开始为第一周，每7天一周
    fn week_of_month(_year: i32, _month: u32, day: u32) -> u32 {
        ((day - 1) / 7) + 1
    }

    /// 计算使用趋势
    ///
    /// 比较两个时间段的使用情况
    pub fn calculate_trend(
        current: &AggregationResult,
        previous: &AggregationResult,
    ) -> TrendAnalysis {
        let current_total = current.total_seconds;
        let previous_total = previous.total_seconds;

        let change = if previous_total > 0 {
            ((current_total - previous_total) as f64 / previous_total as f64) * 100.0
        } else {
            0.0
        };

        let direction = if change > 10.0 {
            TrendDirection::Increasing
        } else if change < -10.0 {
            TrendDirection::Decreasing
        } else {
            TrendDirection::Stable
        };

        // 分析变化最大的应用
        let current_apps = current.by_app();
        let previous_apps = previous.by_app();

        let mut app_changes: Vec<(String, f64)> = current_apps
            .keys()
            .chain(previous_apps.keys())
            .map(|app| {
                let current_val = *current_apps.get(app).unwrap_or(&0);
                let previous_val = *previous_apps.get(app).unwrap_or(&0);

                let app_change = if previous_val > 0 {
                    ((current_val - previous_val) as f64 / previous_val as f64) * 100.0
                } else {
                    0.0
                };

                (app.clone(), app_change)
            })
            .collect();

        app_changes.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        TrendAnalysis {
            direction,
            change_percent: change,
            top_increasing: app_changes
                .iter()
                .filter(|(_, c)| *c > 0.0)
                .take(5)
                .map(|(a, c)| (a.clone(), *c))
                .collect(),
            top_decreasing: app_changes
                .iter()
                .filter(|(_, c)| *c < 0.0)
                .take(5)
                .map(|(a, c)| (a.clone(), *c))
                .collect(),
        }
    }
}

/// 趋势方向
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrendDirection {
    Increasing,
    Stable,
    Decreasing,
}

/// 趋势分析结果
#[derive(Debug, Clone)]
pub struct TrendAnalysis {
    /// 趋势方向
    pub direction: TrendDirection,
    /// 变化百分比
    pub change_percent: f64,
    /// 增加最多的应用
    pub top_increasing: Vec<(String, f64)>,
    /// 减少最多的应用
    pub top_decreasing: Vec<(String, f64)>,
}

// 为 TimeRange 添加引用支持
use super::time_event::TimeRange;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_events() -> Vec<TimeEvent> {
        // 使用 UTC 时间创建测试数据
        let base = chrono::NaiveDate::from_ymd_opt(2024, 1, 15)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap()
            .and_utc();

        vec![
            TimeEvent::new(base, "firefox", 3600).unwrap(),
            TimeEvent::new(base + chrono::Duration::seconds(3600), "vscode", 7200).unwrap(),
            TimeEvent::new(base + chrono::Duration::seconds(10800), "terminal", 1800).unwrap(),
            TimeEvent::new(base + chrono::Duration::seconds(12600), "firefox", 900).unwrap(),
        ]
    }

    #[test]
    fn test_aggregate_by_hour() {
        let events = create_test_events();
        let result = TimeSeriesAnalyzer::aggregate(&events, TimeGranularity::Hour, None);

        assert_eq!(result.buckets.len(), 24);

        // 总时长应该是 3600 + 7200 + 1800 + 900 = 13500 秒
        assert_eq!(result.total_seconds, 13500);

        // 第一个事件应该被聚合到某个桶
        let total_in_buckets: i64 = result.buckets.iter().map(|b| b.total_seconds).sum();
        assert_eq!(total_in_buckets, 13500);
    }

    #[test]
    fn test_aggregate_by_day() {
        let events = create_test_events();
        let result = TimeSeriesAnalyzer::aggregate(&events, TimeGranularity::Day, None);

        assert_eq!(result.buckets.len(), 7);

        // 所有事件应该在同一个桶中（同一天）
        let non_empty_buckets: Vec<_> = result.buckets.iter().filter(|b| b.total_seconds > 0).collect();
        assert_eq!(non_empty_buckets.len(), 1);

        // 总时长应该是 12600 秒 (3600+7200+1800，不包括重复的 firefox)
        let total_in_non_empty: i64 = non_empty_buckets.iter().map(|b| b.total_seconds).sum();
        assert_eq!(total_in_non_empty, 13500); // 所有事件的和
    }

    #[test]
    fn test_top_apps() {
        let events = create_test_events();
        let result = TimeSeriesAnalyzer::aggregate(&events, TimeGranularity::Hour, None);

        let top = result.top_apps(3);

        // vscode 应该排第一 (7200秒)
        assert_eq!(top[0].0, "vscode");
        assert_eq!(top[0].1, 7200);

        // firefox 第二 (3600+900=4500秒)
        assert_eq!(top[1].0, "firefox");
        assert_eq!(top[1].1, 4500);
    }

    #[test]
    fn test_aggregate_filters_afk() {
        let base = chrono::Utc::now();
        let mut events = create_test_events();

        // 添加 AFK 事件
        events.push(TimeEvent::afk(base, 3600).unwrap());

        let result = TimeSeriesAnalyzer::aggregate(&events, TimeGranularity::Hour, None);

        // AFK 事件应该被过滤掉
        assert_eq!(result.total_seconds, 13500); // 不包含 AFK 的 3600
    }

    #[test]
    fn test_aggregate_with_time_range() {
        let events = create_test_events();

        // 只包含前两个小时
        let start = "2024-01-15T10:00:00Z"
            .parse::<chrono::DateTime<chrono::Utc>>()
            .unwrap();
        let end = "2024-01-15T13:00:00Z"
            .parse::<chrono::DateTime<chrono::Utc>>()
            .unwrap();
        let range = TimeRange::new(start, end).unwrap();

        let result =
            TimeSeriesAnalyzer::aggregate(&events, TimeGranularity::Hour, Some(&range));

        // 只应该有前三个事件
        assert_eq!(result.total_seconds, 12600);
    }

    #[test]
    fn test_week_of_month() {
        assert_eq!(TimeSeriesAnalyzer::week_of_month(2024, 1, 1), 1);
        assert_eq!(TimeSeriesAnalyzer::week_of_month(2024, 1, 7), 1);
        assert_eq!(TimeSeriesAnalyzer::week_of_month(2024, 1, 8), 2);
        assert_eq!(TimeSeriesAnalyzer::week_of_month(2024, 1, 15), 3);
    }

    #[test]
    fn test_trend_analysis() {
        let current_events = create_test_events();
        let previous_events = {
            let mut events = create_test_events();
            // 修改时长
            events[0] = TimeEvent::new(
                events[0].timestamp - chrono::Duration::days(7),
                "firefox",
                1800,
            )
            .unwrap();
            events
        };

        let current =
            TimeSeriesAnalyzer::aggregate(&current_events, TimeGranularity::Hour, None);
        let previous =
            TimeSeriesAnalyzer::aggregate(&previous_events, TimeGranularity::Hour, None);

        let trend = TimeSeriesAnalyzer::calculate_trend(&current, &previous);

        // firefox 使用时间增加了 (3600 -> 4500)
        assert!(!trend.top_increasing.is_empty());
    }
}
