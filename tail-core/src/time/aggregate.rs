//! 时间聚合计算
//!
//! 将原始事件数据按时间粒度聚合为时间槽
//!
//! # 设计原则
//!
//! 1. **准确聚合**: 确保聚合结果与原始数据一致
//! 2. **时间范围过滤**: 支持按时间范围过滤事件
//! 3. **总时长计算**: total_seconds 必须反映实际聚合的数据

use chrono::{DateTime, Datelike, Local, Timelike, Utc};
use std::collections::HashMap;

use crate::models::AppUsage;
use crate::time::range::TimeRange;
use crate::time::types::{TimeGranularity, TimeSlot, TimeSlots};

/// 分组模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupMode {
    /// 按应用分组
    ByApp,
    /// 按分类分组
    ByCategory,
}

/// 时间聚合器
///
/// 负责将原始事件数据按时间粒度聚合
pub struct TimeAggregator<'a> {
    /// 应用使用数据
    app_usage: &'a [AppUsage],
    /// 时间范围过滤器
    time_range: Option<TimeRange>,
    /// 分组模式
    group_mode: GroupMode,
}

impl<'a> TimeAggregator<'a> {
    /// 创建新的聚合器
    pub fn new(app_usage: &'a [AppUsage]) -> Self {
        Self {
            app_usage,
            time_range: None,
            group_mode: GroupMode::ByApp,
        }
    }

    /// 设置时间范围过滤
    pub fn with_time_range(mut self, range: TimeRange) -> Self {
        self.time_range = Some(range);
        self
    }

    /// 设置分组模式
    pub fn with_group_mode(mut self, mode: GroupMode) -> Self {
        self.group_mode = mode;
        self
    }

    /// 按日聚合（24小时）
    ///
    /// 返回 24 个时间槽，每个代表一小时
    pub fn aggregate_by_day(&self) -> TimeSlots {
        let mut slots = Self::create_slots_by_granularity(TimeGranularity::Day);

        let mut total_seconds = 0i64;

        for usage in self.app_usage {
            if usage.app_name.is_empty() {
                continue;
            }

            for event in &usage.window_events {
                if event.is_afk {
                    continue;
                }

                // 时间范围过滤
                if !self.is_event_in_range(event.timestamp) {
                    continue;
                }

                let local_time = event.timestamp.with_timezone(&Local);
                let hour = local_time.hour() as usize;

                if hour < slots.len() {
                    let seconds = event.duration_secs;
                    total_seconds += seconds;

                    let slot = slots.get_slot_mut(hour).unwrap();
                    match self.group_mode {
                        GroupMode::ByApp => {
                            slot.add_duration(&usage.app_name, seconds);
                        }
                        GroupMode::ByCategory => {
                            // TODO: 添加分类支持
                            slot.add_duration("未分类", seconds);
                        }
                    }
                }
            }
        }

        // 验证：所有槽的时长之和应该等于总时长
        let calculated_total: i64 = slots
            .slots()
            .iter()
            .map(|s| s.duration().as_seconds())
            .sum();

        assert_eq!(
            calculated_total, total_seconds,
            "聚合后的总时长与计算值不匹配: {} vs {}",
            calculated_total, total_seconds
        );

        slots
    }

    /// 按周聚合（7天）
    ///
    /// 返回 7 个时间槽，每个代表一天（周一到周日）
    pub fn aggregate_by_week(&self) -> TimeSlots {
        let weekday_labels = ["周一", "周二", "周三", "周四", "周五", "周六", "周日"];
        let mut slots = TimeSlots::new(TimeGranularity::Week);
        for (i, label) in weekday_labels.iter().enumerate() {
            slots.add_slot(TimeSlot::new(label.to_string(), i));
        }

        let mut total_seconds = 0i64;

        for usage in self.app_usage {
            if usage.app_name.is_empty() {
                continue;
            }

            for event in &usage.window_events {
                if event.is_afk {
                    continue;
                }

                if !self.is_event_in_range(event.timestamp) {
                    continue;
                }

                let local_time = event.timestamp.with_timezone(&Local);
                let weekday = local_time.weekday().num_days_from_monday() as usize;

                if weekday < slots.len() {
                    let seconds = event.duration_secs;
                    total_seconds += seconds;

                    let slot = slots.get_slot_mut(weekday).unwrap();
                    match self.group_mode {
                        GroupMode::ByApp => {
                            slot.add_duration(&usage.app_name, seconds);
                        }
                        GroupMode::ByCategory => {
                            slot.add_duration("未分类", seconds);
                        }
                    }
                }
            }
        }

        // 验证总时长
        let calculated_total: i64 = slots
            .slots()
            .iter()
            .map(|s| s.duration().as_seconds())
            .sum();
        assert_eq!(calculated_total, total_seconds);

        slots
    }

    /// 按月聚合（最多6周）
    ///
    /// 返回该月的周数时间槽（最多6个）
    pub fn aggregate_by_month(&self) -> TimeSlots {
        let mut weekly_data: HashMap<u32, TimeSlot> = HashMap::new();
        let mut total_seconds = 0i64;

        for usage in self.app_usage {
            if usage.app_name.is_empty() {
                continue;
            }

            for event in &usage.window_events {
                if event.is_afk {
                    continue;
                }

                if !self.is_event_in_range(event.timestamp) {
                    continue;
                }

                let local_time = event.timestamp.with_timezone(&Local);
                let day = local_time.day();

                if day == 0 {
                    continue;
                }

                // 使用统一的周计算逻辑
                let week = crate::time::range::TimeRangeCalculator::week_of_month(
                    local_time.year(),
                    local_time.month(),
                    day,
                );

                if week == 0 || week > 6 {
                    continue;
                }

                let slot = weekly_data
                    .entry(week)
                    .or_insert_with(|| TimeSlot::new(format!("第{}周", week), (week - 1) as usize));

                let seconds = event.duration_secs;
                total_seconds += seconds;

                match self.group_mode {
                    GroupMode::ByApp => {
                        slot.add_duration(&usage.app_name, seconds);
                    }
                    GroupMode::ByCategory => {
                        slot.add_duration("未分类", seconds);
                    }
                }
            }
        }

        let mut slots = TimeSlots::new(TimeGranularity::Month);
        let mut slot_vec: Vec<_> = weekly_data.into_values().collect();
        slot_vec.sort_by_key(|s| s.index());

        // 验证总时长
        let calculated_total: i64 = slot_vec.iter().map(|s| s.duration().as_seconds()).sum();
        assert_eq!(calculated_total, total_seconds);

        for slot in slot_vec {
            slots.add_slot(slot);
        }

        slots
    }

    /// 按年聚合（12个月）
    ///
    /// 返回 12 个时间槽，每个代表一个月
    pub fn aggregate_by_year(&self) -> TimeSlots {
        let month_labels = [
            "1月", "2月", "3月", "4月", "5月", "6月", "7月", "8月", "9月", "10月", "11月", "12月",
        ];
        let mut slots = TimeSlots::new(TimeGranularity::Year);
        for (i, label) in month_labels.iter().enumerate() {
            slots.add_slot(TimeSlot::new(label.to_string(), i));
        }

        let mut total_seconds = 0i64;

        for usage in self.app_usage {
            if usage.app_name.is_empty() {
                continue;
            }

            for event in &usage.window_events {
                if event.is_afk {
                    continue;
                }

                if !self.is_event_in_range(event.timestamp) {
                    continue;
                }

                let local_time = event.timestamp.with_timezone(&Local);
                let month = local_time.month() as usize;

                if month == 0 || month > 12 {
                    continue;
                }

                let month_idx = month - 1;
                let seconds = event.duration_secs;
                total_seconds += seconds;

                let slot = slots.get_slot_mut(month_idx).unwrap();
                match self.group_mode {
                    GroupMode::ByApp => {
                        slot.add_duration(&usage.app_name, seconds);
                    }
                    GroupMode::ByCategory => {
                        slot.add_duration("未分类", seconds);
                    }
                }
            }
        }

        // 验证总时长
        let calculated_total: i64 = slots
            .slots()
            .iter()
            .map(|s| s.duration().as_seconds())
            .sum();
        assert_eq!(calculated_total, total_seconds);

        slots
    }

    /// 按小时聚合（60分钟）
    ///
    /// 返回 60 个时间槽，每个代表一分钟
    pub fn aggregate_by_hour(&self) -> TimeSlots {
        let mut slots = Self::create_slots_by_granularity(TimeGranularity::Hour);
        let mut total_seconds = 0i64;

        for usage in self.app_usage {
            if usage.app_name.is_empty() {
                continue;
            }

            for event in &usage.window_events {
                if event.is_afk {
                    continue;
                }

                if !self.is_event_in_range(event.timestamp) {
                    continue;
                }

                let local_time = event.timestamp.with_timezone(&Local);
                let minute = local_time.minute() as usize;

                if minute < slots.len() {
                    let seconds = event.duration_secs;
                    total_seconds += seconds;

                    let slot = slots.get_slot_mut(minute).unwrap();
                    match self.group_mode {
                        GroupMode::ByApp => {
                            slot.add_duration(&usage.app_name, seconds);
                        }
                        GroupMode::ByCategory => {
                            slot.add_duration("未分类", seconds);
                        }
                    }
                }
            }
        }

        // 验证总时长
        let calculated_total: i64 = slots
            .slots()
            .iter()
            .map(|s| s.duration().as_seconds())
            .sum();
        assert_eq!(calculated_total, total_seconds);

        slots
    }

    /// 按指定粒度聚合
    pub fn aggregate(&self, granularity: TimeGranularity) -> TimeSlots {
        match granularity {
            TimeGranularity::Day => self.aggregate_by_day(),
            TimeGranularity::Week => self.aggregate_by_week(),
            TimeGranularity::Month => self.aggregate_by_month(),
            TimeGranularity::Year => self.aggregate_by_year(),
            TimeGranularity::Hour => self.aggregate_by_hour(),
        }
    }

    /// 检查事件是否在时间范围内
    fn is_event_in_range(&self, timestamp: DateTime<Utc>) -> bool {
        if let Some(range) = &self.time_range {
            range.contains(timestamp)
        } else {
            true
        }
    }

    /// 创建指定粒度的空时间槽
    fn create_slots_by_granularity(granularity: TimeGranularity) -> TimeSlots {
        let mut slots = TimeSlots::new(granularity);

        for i in 0..granularity.slot_count() {
            let label = granularity.default_slot_label(i);
            slots.add_slot(TimeSlot::new(label, i));
        }

        slots
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, NaiveTime};

    fn create_test_app_usage() -> Vec<AppUsage> {
        // 创建测试数据：2024-01-15 (周一) 的一些事件
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let time1 = NaiveTime::from_hms_opt(10, 30, 0).unwrap();
        let time2 = NaiveTime::from_hms_opt(14, 0, 0).unwrap();

        let dt1 = date
            .and_time(time1)
            .and_local_timezone(Local)
            .unwrap()
            .with_timezone(&Utc);
        let dt2 = date
            .and_time(time2)
            .and_local_timezone(Local)
            .unwrap()
            .with_timezone(&Utc);

        vec![
            AppUsage {
                app_name: "App1".to_string(),
                total_seconds: 3665, // 1h 1m 5s
                window_events: vec![WindowEvent {
                    id: None,
                    timestamp: dt1,
                    duration_secs: 3665,
                    app_name: "App1".to_string(),
                    window_title: "Test".to_string(),
                    workspace: String::new(),
                    is_afk: false,
                }],
            },
            AppUsage {
                app_name: "App2".to_string(),
                total_seconds: 1800, // 30m
                window_events: vec![WindowEvent {
                    id: None,
                    timestamp: dt2,
                    duration_secs: 1800,
                    app_name: "App2".to_string(),
                    window_title: "Test".to_string(),
                    workspace: String::new(),
                    is_afk: false,
                }],
            },
        ]
    }

    #[test]
    fn test_aggregate_by_day() {
        let data = create_test_app_usage();
        let aggregator = TimeAggregator::new(&data);
        let slots = aggregator.aggregate_by_day();

        // 应该有24个小时槽
        assert_eq!(slots.len(), 24);

        // 10点的槽应该有 App1 的数据
        let slot_10 = slots.get_slot(10).unwrap();
        assert_eq!(slot_10.duration().as_seconds(), 3665);
        assert_eq!(slot_10.get_group_duration("App1"), 3665);

        // 14点的槽应该有 App2 的数据
        let slot_14 = slots.get_slot(14).unwrap();
        assert_eq!(slot_14.duration().as_seconds(), 1800);
        assert_eq!(slot_14.get_group_duration("App2"), 1800);

        // 总时长应该是 3665 + 1800 = 5465
        assert_eq!(slots.total_duration().as_seconds(), 5465);
    }

    #[test]
    fn test_aggregate_by_week() {
        let data = create_test_app_usage();
        let aggregator = TimeAggregator::new(&data);
        let slots = aggregator.aggregate_by_week();

        // 2024-01-15 是周一
        assert_eq!(slots.len(), 7);

        let slot_monday = slots.get_slot(0).unwrap();
        assert_eq!(slot_monday.label(), "周一");
        assert_eq!(slot_monday.duration().as_seconds(), 5465);
    }

    #[test]
    fn test_time_range_filter() {
        let data = create_test_app_usage();

        // 设置时间范围：只包含上午
        use crate::time::range::TimeRangeCalculator;
        let range = TimeRangeCalculator::day(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());

        // 创建只包含上午的时间范围
        let start = NaiveDate::from_ymd_opt(2024, 1, 15)
            .unwrap()
            .and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
            .and_local_timezone(Local)
            .unwrap()
            .with_timezone(&Utc);
        let end = NaiveDate::from_ymd_opt(2024, 1, 15)
            .unwrap()
            .and_time(NaiveTime::from_hms_opt(12, 0, 0).unwrap())
            .and_local_timezone(Local)
            .unwrap()
            .with_timezone(&Utc);
        let range = TimeRange::new(start, end);

        let aggregator = TimeAggregator::new(&data).with_time_range(range);
        let slots = aggregator.aggregate_by_day();

        // 应该只有10点的数据
        let slot_10 = slots.get_slot(10).unwrap();
        assert_eq!(slot_10.duration().as_seconds(), 3665);

        // 14点的数据应该被过滤掉
        let slot_14 = slots.get_slot(14).unwrap();
        assert_eq!(slot_14.duration().as_seconds(), 0);

        // 总时长应该只有被过滤的数据
        assert_eq!(slots.total_duration().as_seconds(), 3665);
    }

    #[test]
    fn test_week_of_month_consistency() {
        // 测试周计算的一致性
        use crate::time::range::TimeRangeCalculator;
        // 2024年1月1日是周一
        assert_eq!(TimeRangeCalculator::week_of_month(2024, 1, 1), 1); // 周一
        assert_eq!(TimeRangeCalculator::week_of_month(2024, 1, 7), 1); // 周日
        assert_eq!(TimeRangeCalculator::week_of_month(2024, 1, 8), 2); // 下周一
        assert_eq!(TimeRangeCalculator::week_of_month(2024, 1, 14), 2); // 该周日
        assert_eq!(TimeRangeCalculator::week_of_month(2024, 1, 15), 3); // 下周一
    }
}
