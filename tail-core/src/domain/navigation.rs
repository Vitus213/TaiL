//! 导航路径 - 状态机实现
//!
//! 管理时间导航的状态和转换。

use chrono::{Datelike, Local, NaiveDate, TimeZone as ChronoTZ, Utc};
use std::fmt;

use super::errors::{DomainError, NavigationError};
use super::time_event::TimeRange;

/// 导航路径 - 状态机
///
/// 管理用户在时间层级中的导航状态
#[derive(Debug, Clone)]
pub struct NavigationPath {
    state: NavigationState,
    current_year: i32,
}

#[derive(Debug, Clone)]
enum NavigationState {
    Year,
    Month { month: u32 },
    Week { month: u32, week: u32 },
    Day { month: u32, week: u32, day: u32 },
    Hour { month: u32, day: u32 },
}

impl NavigationPath {
    /// 创建新的导航路径（默认在年视图）
    pub fn new() -> Self {
        let now = Local::now();
        Self {
            state: NavigationState::Year,
            current_year: now.year(),
        }
    }

    /// 从指定年份创建
    pub fn from_year(year: i32) -> Self {
        Self {
            state: NavigationState::Year,
            current_year: year,
        }
    }

    /// 跳转到今天（小时视图）
    pub fn go_to_today(&mut self) -> Result<(), DomainError> {
        let now = Local::now();
        self.current_year = now.year();

        self.state = NavigationState::Hour {
            month: now.month(),
            day: now.day(),
        };

        Ok(())
    }

    /// 跳转到昨天（小时视图）
    pub fn go_to_yesterday(&mut self) -> Result<(), DomainError> {
        let yesterday = Local::now() - chrono::Duration::days(1);
        self.current_year = yesterday.year();

        self.state = NavigationState::Hour {
            month: yesterday.month(),
            day: yesterday.day(),
        };

        Ok(())
    }

    /// 跳转到本周（日视图）
    pub fn go_to_this_week(&mut self) -> Result<(), DomainError> {
        let now = Local::now();
        self.current_year = now.year();

        let week = Self::week_of_month(now.year(), now.month(), now.day());

        self.state = NavigationState::Day {
            month: now.month(),
            week,
            day: now.day(),
        };

        Ok(())
    }

    /// 跳转到本月（周视图）
    pub fn go_to_this_month(&mut self) -> Result<(), DomainError> {
        let now = Local::now();
        self.current_year = now.year();

        self.state = NavigationState::Week {
            month: now.month(),
            week: 1,
        };

        Ok(())
    }

    /// 向下钻取
    pub fn drill_down(&mut self, selector: TimeSelector) -> Result<(), DomainError> {
        match (&self.state, selector) {
            (NavigationState::Year, TimeSelector::Month(month)) => {
                if month < 1 || month > 12 {
                    return Err(NavigationError::InvalidSelector.into());
                }
                self.state = NavigationState::Month { month };
                Ok(())
            }
            (NavigationState::Month { month }, TimeSelector::Week(week)) => {
                if week < 1 || week > 6 {
                    return Err(NavigationError::InvalidSelector.into());
                }
                self.state = NavigationState::Week { month: *month, week };
                Ok(())
            }
            (NavigationState::Week { month, week }, TimeSelector::Day(day)) => {
                if day < 1 || day > 31 {
                    return Err(NavigationError::InvalidSelector.into());
                }
                self.state = NavigationState::Day {
                    month: *month,
                    week: *week,
                    day,
                };
                Ok(())
            }
            (NavigationState::Day { month, week: _, day }, TimeSelector::Hour(_)) => {
                self.state = NavigationState::Hour {
                    month: *month,
                    day: *day,
                };
                Ok(())
            }
            (NavigationState::Hour { .. }, _) => {
                Err(NavigationError::AlreadyAtBottom.into())
            }
            _ => Err(NavigationError::InvalidTransition.into()),
        }
    }

    /// 向上返回
    ///
    /// 返回我们离开的层级，而不是进入的层级
    pub fn drill_up(&mut self) -> Result<Option<NavigationLevel>, DomainError> {
        match &self.state {
            NavigationState::Year => Err(NavigationError::AlreadyAtTop.into()),
            NavigationState::Month { .. } => {
                self.state = NavigationState::Year;
                Ok(Some(NavigationLevel::Month))  // 返回离开的层级
            }
            NavigationState::Week { month, .. } => {
                self.state = NavigationState::Month { month: *month };
                Ok(Some(NavigationLevel::Week))  // 返回离开的层级
            }
            NavigationState::Day { month, .. } => {
                self.state = NavigationState::Week {
                    month: *month,
                    week: 1,
                };
                Ok(Some(NavigationLevel::Day))  // 返回离开的层级
            }
            NavigationState::Hour { month, day } => {
                let week = Self::week_of_month(self.current_year, *month, *day);
                self.state = NavigationState::Day {
                    month: *month,
                    week,
                    day: *day,
                };
                Ok(Some(NavigationLevel::Hour))  // 返回离开的层级
            }
        }
    }

    /// 切换视图级别（保留当前上下文）
    pub fn switch_level(&mut self, level: NavigationLevel) -> Result<(), DomainError> {
        let now = Local::now();

        match level {
            NavigationLevel::Year => {
                self.state = NavigationState::Year;
            }
            NavigationLevel::Month => {
                let month = match self.state {
                    NavigationState::Month { month } => month,
                    NavigationState::Week { month, .. } => month,
                    NavigationState::Day { month, .. } => month,
                    NavigationState::Hour { month, .. } => month,
                    NavigationState::Year => now.month(),
                };
                self.state = NavigationState::Month { month };
            }
            NavigationLevel::Week => {
                let (month, week) = match self.state {
                    NavigationState::Week { month, week } => (month, week),
                    NavigationState::Day { month, week, .. } => (month, week),
                    NavigationState::Hour { month, day } => {
                        (month, Self::week_of_month(self.current_year, month, day))
                    }
                    NavigationState::Month { month } => (month, 1),
                    NavigationState::Year => (now.month(), 1),
                };
                self.state = NavigationState::Week { month, week };
            }
            NavigationLevel::Day => {
                let (month, week, day) = match self.state {
                    NavigationState::Day { month, week, day } => (month, week, day),
                    NavigationState::Hour { month, day } => {
                        let week = Self::week_of_month(self.current_year, month, day);
                        (month, week, day)
                    }
                    NavigationState::Week { month, week } => (month, week, 1),
                    NavigationState::Month { month } => (month, 1, 1),
                    NavigationState::Year => {
                        let week = Self::week_of_month(now.year(), now.month(), now.day());
                        (now.month(), week, now.day())
                    }
                };
                self.state = NavigationState::Day { month, week, day };
            }
            NavigationLevel::Hour => {
                let (month, day) = match self.state {
                    NavigationState::Hour { month, day } => (month, day),
                    NavigationState::Day { month, day, .. } => (month, day),
                    NavigationState::Week { month, .. } => (month, 1),
                    NavigationState::Month { month } => (month, 1),
                    NavigationState::Year => (now.month(), now.day()),
                };
                self.state = NavigationState::Hour { month, day };
            }
        }

        Ok(())
    }

    /// 获取当前时间范围
    pub fn current_range(&self) -> TimeRange {
        match &self.state {
            NavigationState::Year => TimeRange::year(self.current_year),
            NavigationState::Month { month } => TimeRange::month(self.current_year, *month),
            NavigationState::Week { month, week } => {
                TimeRange::week(self.current_year, *month, *week)
            }
            NavigationState::Day { month, week, .. } => {
                TimeRange::week(self.current_year, *month, *week)
            }
            NavigationState::Hour { month, day } => {
                TimeRange::day(self.current_year, *month, *day)
            }
        }
    }

    /// 获取当前粒度
    pub fn current_granularity(&self) -> crate::domain::aggregation::TimeGranularity {
        match &self.state {
            NavigationState::Year => crate::domain::aggregation::TimeGranularity::Month,
            NavigationState::Month { .. } => {
                crate::domain::aggregation::TimeGranularity::Month
            }
            NavigationState::Week { .. } => crate::domain::aggregation::TimeGranularity::Day,
            NavigationState::Day { .. } => crate::domain::aggregation::TimeGranularity::Day,
            NavigationState::Hour { .. } => crate::domain::aggregation::TimeGranularity::Hour,
        }
    }

    /// 获取当前层级
    pub fn current_level(&self) -> NavigationLevel {
        match &self.state {
            NavigationState::Year => NavigationLevel::Year,
            NavigationState::Month { .. } => NavigationLevel::Month,
            NavigationState::Week { .. } => NavigationLevel::Week,
            NavigationState::Day { .. } => NavigationLevel::Day,
            NavigationState::Hour { .. } => NavigationLevel::Hour,
        }
    }

    /// 获取面包屑导航文本
    pub fn breadcrumb(&self) -> String {
        match &self.state {
            NavigationState::Year => format!("{}年", self.current_year),
            NavigationState::Month { month } => format!("{}年 > {}月", self.current_year, month),
            NavigationState::Week { month, week } => {
                format!("{}年 > {}月 > 第{}周", self.current_year, month, week)
            }
            NavigationState::Day { month, week, day } => {
                format!("{}年 > {}月 > 第{}周 > {}日", self.current_year, month, week, day)
            }
            NavigationState::Hour { month, day } => {
                format!("{}年 > {}月 > {}日", self.current_year, month, day)
            }
        }
    }

    /// 是否可以向下钻取
    pub fn can_drill_down(&self) -> bool {
        !matches!(self.state, NavigationState::Hour { .. })
    }

    /// 是否可以向上返回
    pub fn can_drill_up(&self) -> bool {
        !matches!(self.state, NavigationState::Year)
    }

    /// 获取当前年份
    pub fn year(&self) -> i32 {
        self.current_year
    }

    /// 获取当前月份（如果适用）
    pub fn month(&self) -> Option<u32> {
        match &self.state {
            NavigationState::Month { month } => Some(*month),
            NavigationState::Week { month, .. } => Some(*month),
            NavigationState::Day { month, .. } => Some(*month),
            NavigationState::Hour { month, .. } => Some(*month),
            NavigationState::Year => None,
        }
    }

    /// 计算某日是该月的第几周
    fn week_of_month(year: i32, month: u32, day: u32) -> u32 {
        if let Some(first_day) = NaiveDate::from_ymd_opt(year, month, 1) {
            let first_weekday = first_day.weekday().num_days_from_monday();
            ((day as i32 - 1 + first_weekday as i32) / 7 + 1) as u32
        } else {
            1
        }
    }
}

impl Default for NavigationPath {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for NavigationPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.breadcrumb())
    }
}

/// 时间选择器 - 用于向下钻取
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeSelector {
    Month(u32),
    Week(u32),
    Day(u32),
    Hour(u32),
}

impl From<u32> for TimeSelector {
    fn from(value: u32) -> Self {
        TimeSelector::Month(value)
    }
}

/// 导航层级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NavigationLevel {
    Year,
    Month,
    Week,
    Day,
    Hour,
}

impl fmt::Display for NavigationLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NavigationLevel::Year => write!(f, "年"),
            NavigationLevel::Month => write!(f, "月"),
            NavigationLevel::Week => write!(f, "周"),
            NavigationLevel::Day => write!(f, "日"),
            NavigationLevel::Hour => write!(f, "时"),
        }
    }
}

// 为 TimeRange 添加构造方法
impl TimeRange {
    pub fn year(year: i32) -> Self {
        let start = Utc
            .with_ymd_and_hms(year, 1, 1, 0, 0, 0)
            .unwrap();
        let end = Utc
            .with_ymd_and_hms(year, 12, 31, 23, 59, 59)
            .unwrap();
        Self { start, end }
    }

    pub fn month(year: i32, month: u32) -> Self {
        let start = Utc.with_ymd_and_hms(year, month, 1, 0, 0, 0).unwrap();

        let next_month = if month == 12 {
            Utc.with_ymd_and_hms(year + 1, 1, 1, 0, 0, 0).unwrap()
        } else {
            Utc.with_ymd_and_hms(year, month + 1, 1, 0, 0, 0).unwrap()
        };

        let end = next_month - chrono::Duration::seconds(1);
        Self { start, end }
    }

    pub fn week(year: i32, month: u32, week: u32) -> Self {
        let first_day = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        let first_weekday = first_day.weekday().num_days_from_monday();

        let week_start_day = (week as i32 - 1) * 7 - first_weekday as i32 + 1;
        let week_start = if week_start_day < 1 {
            first_day
        } else {
            first_day + chrono::Duration::days(week_start_day as i64 - 1)
        };

        let week_end = week_start + chrono::Duration::days(6);

        let start = Utc
            .from_utc_datetime(&week_start.and_hms_opt(0, 0, 0).unwrap());
        let end = Utc.from_utc_datetime(&week_end.and_hms_opt(23, 59, 59).unwrap());

        Self { start, end }
    }

    pub fn day(year: i32, month: u32, day: u32) -> Self {
        let start = Utc.with_ymd_and_hms(year, month, day, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(year, month, day, 23, 59, 59).unwrap();
        Self { start, end }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_navigation_new() {
        let nav = NavigationPath::new();
        assert_eq!(nav.current_level(), NavigationLevel::Year);
        assert!(nav.can_drill_down());
        assert!(!nav.can_drill_up());
    }

    #[test]
    fn test_navigation_drill_down() {
        let mut nav = NavigationPath::from_year(2024);

        // Year -> Month
        nav.drill_down(TimeSelector::Month(1)).unwrap();
        assert_eq!(nav.current_level(), NavigationLevel::Month);
        assert_eq!(nav.month(), Some(1));

        // Month -> Week
        nav.drill_down(TimeSelector::Week(1)).unwrap();
        assert_eq!(nav.current_level(), NavigationLevel::Week);

        // Week -> Day
        nav.drill_down(TimeSelector::Day(15)).unwrap();
        assert_eq!(nav.current_level(), NavigationLevel::Day);

        // Day -> Hour
        nav.drill_down(TimeSelector::Hour(10)).unwrap();
        assert_eq!(nav.current_level(), NavigationLevel::Hour);
        assert!(!nav.can_drill_down());
    }

    #[test]
    fn test_navigation_drill_up() {
        let mut nav = NavigationPath::from_year(2024);

        nav.drill_down(TimeSelector::Month(1)).unwrap();
        nav.drill_down(TimeSelector::Week(1)).unwrap();

        let level = nav.drill_up().unwrap();
        assert_eq!(level, Some(NavigationLevel::Week));  // 返回离开的 Week 层级

        let level = nav.drill_up().unwrap();
        assert_eq!(level, Some(NavigationLevel::Month));  // 返回离开的 Month 层级

        // 现在已经到了 Year (顶层)，再次 drill_up 应该报错
        assert!(nav.drill_up().is_err());
    }

    #[test]
    fn test_navigation_invalid_selector() {
        let mut nav = NavigationPath::from_year(2024);

        // 无效月份
        assert!(matches!(
            nav.drill_down(TimeSelector::Month(13)),
            Err(DomainError::Navigation(NavigationError::InvalidSelector))
        ));

        // 无效转换
        assert!(matches!(
            nav.drill_down(TimeSelector::Week(1)),
            Err(DomainError::Navigation(NavigationError::InvalidTransition))
        ));
    }

    #[test]
    fn test_navigation_breadcrumb() {
        let mut nav = NavigationPath::from_year(2024);

        assert_eq!(nav.breadcrumb(), "2024年");

        nav.drill_down(TimeSelector::Month(1)).unwrap();
        assert_eq!(nav.breadcrumb(), "2024年 > 1月");

        nav.drill_down(TimeSelector::Week(1)).unwrap();
        assert_eq!(nav.breadcrumb(), "2024年 > 1月 > 第1周");

        nav.drill_down(TimeSelector::Day(15)).unwrap();
        assert_eq!(nav.breadcrumb(), "2024年 > 1月 > 第1周 > 15日");
    }

    #[test]
    fn test_time_range_constructors() {
        let year_range = TimeRange::year(2024);
        assert_eq!(year_range.start.year(), 2024);
        assert_eq!(year_range.end.year(), 2024);

        let month_range = TimeRange::month(2024, 1);
        assert_eq!(month_range.start.month(), 1);
        assert_eq!(month_range.start.year(), 2024);

        let day_range = TimeRange::day(2024, 1, 15);
        assert_eq!(day_range.start.day(), 15);
        assert_eq!(day_range.start.month(), 1);
    }

    #[test]
    fn test_go_to_today() {
        let mut nav = NavigationPath::new();
        nav.go_to_today().unwrap();

        assert_eq!(nav.current_level(), NavigationLevel::Hour);
        assert!(nav.month().is_some());
    }

    #[test]
    fn test_switch_level() {
        let mut nav = NavigationPath::from_year(2024);
        nav.drill_down(TimeSelector::Month(1)).unwrap();

        nav.switch_level(NavigationLevel::Week).unwrap();
        assert_eq!(nav.current_level(), NavigationLevel::Week);
        assert_eq!(nav.month(), Some(1));

        nav.switch_level(NavigationLevel::Year).unwrap();
        assert_eq!(nav.current_level(), NavigationLevel::Year);
    }
}
