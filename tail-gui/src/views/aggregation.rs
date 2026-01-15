//! 数据聚合逻辑 - 用于层级时间导航

use chrono::{Datelike, Duration, Local, NaiveDate, Timelike};
use std::collections::HashMap;
use tail_core::AppUsage;
use tail_core::models::{PeriodUsage, TimeNavigationLevel, TimeNavigationState};

/// 数据聚合器
pub struct DataAggregator<'a> {
    app_usage: &'a [AppUsage],
}

impl<'a> DataAggregator<'a> {
    pub fn new(app_usage: &'a [AppUsage]) -> Self {
        Self { app_usage }
    }

    /// 根据导航状态聚合数据
    pub fn aggregate(&self, state: &TimeNavigationState) -> Vec<PeriodUsage> {
        match state.level {
            TimeNavigationLevel::Year => {
                // 年视图不显示（直接进入月视图）
                vec![]
            }
            TimeNavigationLevel::Month => self.aggregate_by_year(state.selected_year),
            TimeNavigationLevel::Week => {
                let month = state.selected_month.unwrap_or(1);
                self.aggregate_by_month(state.selected_year, month)
            }
            TimeNavigationLevel::Day => {
                let month = state.selected_month.unwrap_or(1);
                let week = state.selected_week.unwrap_or(1);
                self.aggregate_by_week(state.selected_year, month, week)
            }
            TimeNavigationLevel::Hour => {
                let month = state.selected_month.unwrap_or(1);
                let day = state.selected_day.unwrap_or(1);
                self.aggregate_by_day(state.selected_year, month, day)
            }
        }
    }

    /// 按年聚合：返回12个月的使用数据
    fn aggregate_by_year(&self, year: i32) -> Vec<PeriodUsage> {
        let mut monthly_usage: HashMap<u32, i64> = HashMap::new();

        for usage in self.app_usage {
            for event in &usage.window_events {
                // 使用本地时间进行比较
                let local_time = event.timestamp.with_timezone(&Local);
                if local_time.year() == year {
                    let month = local_time.month();
                    *monthly_usage.entry(month).or_insert(0) += event.duration_secs;
                }
            }
        }

        (1..=12)
            .map(|month| PeriodUsage {
                label: format!("{}月", month),
                index: month as i32,
                total_seconds: monthly_usage.get(&month).copied().unwrap_or(0),
            })
            .collect()
    }

    /// 按月聚合：返回当月各周的使用数据
    fn aggregate_by_month(&self, year: i32, month: u32) -> Vec<PeriodUsage> {
        let mut weekly_usage: HashMap<u32, i64> = HashMap::new();

        for usage in self.app_usage {
            for event in &usage.window_events {
                // 使用本地时间进行比较
                let local_time = event.timestamp.with_timezone(&Local);
                if local_time.year() == year && local_time.month() == month {
                    let day = local_time.day();
                    let week = Self::get_week_of_month(year, month, day);
                    *weekly_usage.entry(week).or_insert(0) += event.duration_secs;
                }
            }
        }

        // 计算该月有几周
        let days_in_month = Self::days_in_month(year, month);
        let max_week = Self::get_week_of_month(year, month, days_in_month);

        (1..=max_week)
            .map(|week| PeriodUsage {
                label: format!("第{}周", week),
                index: week as i32,
                total_seconds: weekly_usage.get(&week).copied().unwrap_or(0),
            })
            .collect()
    }

    /// 按周聚合：返回当周7天的使用数据
    fn aggregate_by_week(&self, year: i32, month: u32, week: u32) -> Vec<PeriodUsage> {
        // 计算该周的起止日期
        let first_day = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        let first_weekday = first_day.weekday().num_days_from_monday();

        let week_start_day = ((week - 1) * 7) as i64 - first_weekday as i64 + 1;
        let week_start = if week_start_day < 1 {
            first_day
        } else {
            first_day + Duration::days(week_start_day - 1)
        };

        let mut daily_usage: HashMap<NaiveDate, i64> = HashMap::new();

        for usage in self.app_usage {
            for event in &usage.window_events {
                let event_date = event.timestamp.date_naive();
                if event_date >= week_start && event_date < week_start + Duration::days(7) {
                    *daily_usage.entry(event_date).or_insert(0) += event.duration_secs;
                }
            }
        }

        (0..7)
            .map(|i| {
                let date = week_start + Duration::days(i);
                let weekday = date.weekday();
                let label = match weekday {
                    chrono::Weekday::Mon => "周一",
                    chrono::Weekday::Tue => "周二",
                    chrono::Weekday::Wed => "周三",
                    chrono::Weekday::Thu => "周四",
                    chrono::Weekday::Fri => "周五",
                    chrono::Weekday::Sat => "周六",
                    chrono::Weekday::Sun => "周日",
                };

                PeriodUsage {
                    label: label.to_string(),
                    index: date.day() as i32,
                    total_seconds: daily_usage.get(&date).copied().unwrap_or(0),
                }
            })
            .collect()
    }

    /// 按天聚合：返回当天24小时的使用数据
    fn aggregate_by_day(&self, year: i32, month: u32, day: u32) -> Vec<PeriodUsage> {
        let mut hourly_usage: HashMap<u32, i64> = HashMap::new();

        eprintln!("[DEBUG] aggregate_by_day - year={}, month={}, day={}, app_usage.len()={}",
            year, month, day, self.app_usage.len());

        for usage in self.app_usage {
            for event in &usage.window_events {
                // 使用本地时间进行比较
                let local_time = event.timestamp.with_timezone(&Local);
                if local_time.year() == year
                    && local_time.month() == month
                    && local_time.day() == day
                {
                    let hour = local_time.hour();
                    *hourly_usage.entry(hour).or_insert(0) += event.duration_secs;
                }
            }
        }

        let total_hours: i64 = hourly_usage.values().sum();
        eprintln!("[DEBUG] aggregate_by_day - hourly_usage.len()={}, total_hours={}",
            hourly_usage.len(), total_hours);

        (0..24)
            .map(|hour| PeriodUsage {
                label: format!("{}时", hour),
                index: hour as i32,
                total_seconds: hourly_usage.get(&hour).copied().unwrap_or(0),
            })
            .collect()
    }

    /// 计算某天是该月的第几周
    fn get_week_of_month(year: i32, month: u32, day: u32) -> u32 {
        let first_day = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        let first_weekday = first_day.weekday().num_days_from_monday();
        ((day + first_weekday - 1) / 7) + 1
    }

    /// 获取某月的天数
    fn days_in_month(year: i32, month: u32) -> u32 {
        if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap()
        }
        .signed_duration_since(NaiveDate::from_ymd_opt(year, month, 1).unwrap())
        .num_days() as u32
    }
}
