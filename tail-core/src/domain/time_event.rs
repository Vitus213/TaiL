//! 时间事件 - 领域模型的值对象
//!
//! TimeEvent 是一个值对象，表示一个时间窗口内的应用使用事件。

use chrono::{DateTime, Datelike, Duration as ChronoDuration, Local, Timelike, Utc};
use std::fmt;

use super::errors::{DomainError, ValidationError};

/// 时间事件 - 值对象
///
/// # 设计原则
/// - 不可变（Immutable）
/// - 值语义（Value Semantics）
/// - 自验证（Self-Validating）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeEvent {
    pub timestamp: DateTime<Utc>,
    pub app_name: AppName,
    pub window_title: Option<String>,
    pub workspace: Option<String>,
    pub duration: ChronoDuration,
    pub is_afk: bool,
}

impl TimeEvent {
    /// 创建新的时间事件
    ///
    /// # 参数
    /// - `timestamp`: 事件时间戳
    /// - `app_name`: 应用名称（非空字符串）
    /// - `duration_secs`: 持续时间（秒）
    pub fn new(
        timestamp: DateTime<Utc>,
        app_name: impl Into<String>,
        duration_secs: i64,
    ) -> Result<Self, DomainError> {
        let app_name = AppName::new(app_name)?;
        let duration = ChronoDuration::seconds(duration_secs);

        if duration.num_seconds() < 0 {
            return Err(DomainError::InvalidDuration(duration_secs));
        }

        Ok(Self {
            timestamp,
            app_name,
            window_title: None,
            workspace: None,
            duration,
            is_afk: false,
        })
    }

    /// 创建 AFK 事件
    pub fn afk(
        start_time: DateTime<Utc>,
        duration_secs: i64,
    ) -> Result<Self, DomainError> {
        let duration = ChronoDuration::seconds(duration_secs);

        if duration.num_seconds() < 0 {
            return Err(DomainError::InvalidDuration(duration_secs));
        }

        Ok(Self {
            timestamp: start_time,
            app_name: AppName::afk(),
            window_title: None,
            workspace: None,
            duration,
            is_afk: true,
        })
    }

    /// 设置窗口标题
    pub fn with_window_title(mut self, title: impl Into<String>) -> Self {
        self.window_title = Some(title.into());
        self
    }

    /// 设置工作区
    pub fn with_workspace(mut self, workspace: impl Into<String>) -> Self {
        self.workspace = Some(workspace.into());
        self
    }

    /// 检查事件是否在指定时间范围内
    pub fn is_within_range(&self, range: &TimeRange) -> bool {
        range.contains(&self.timestamp)
    }

    /// 获取事件的小时索引 (0-23)
    pub fn hour_index(&self) -> usize {
        self.timestamp.with_timezone(&Local).hour() as usize
    }

    /// 获取事件的分钟索引 (0-59)
    pub fn minute_index(&self) -> usize {
        self.timestamp.with_timezone(&Local).minute() as usize
    }

    /// 获取事件的星期索引 (0-6, 周一为0)
    pub fn weekday_index(&self) -> usize {
        self.timestamp
            .with_timezone(&Local)
            .weekday()
            .num_days_from_monday() as usize
    }

    /// 获取事件的月份索引 (1-12)
    pub fn month_index(&self) -> usize {
        self.timestamp.with_timezone(&Local).month() as usize
    }

    /// 获取事件的日期 (1-31)
    pub fn day_index(&self) -> usize {
        self.timestamp.with_timezone(&Local).day() as usize
    }

    /// 持续时间（秒）
    pub fn duration_secs(&self) -> i64 {
        self.duration.num_seconds()
    }

    /// 持续时间是否为零
    pub fn is_zero_duration(&self) -> bool {
        self.duration.num_seconds() == 0
    }
}

impl fmt::Display for TimeEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} - {} ({}秒)",
            self.app_name,
            self.timestamp.format("%Y-%m-%d %H:%M:%S"),
            self.duration_secs()
        )
    }
}

/// 应用名称 - 值对象
///
/// 确保应用名称非空且有效
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AppName(String);

impl AppName {
    /// 创建新的应用名称
    pub fn new(name: impl Into<String>) -> Result<Self, ValidationError> {
        let name = name.into();
        let trimmed = name.trim();

        if trimmed.is_empty() {
            return Err(ValidationError::EmptyAppName);
        }

        Ok(Self(trimmed.to_string()))
    }

    /// 创建 AFK 应用名称
    pub fn afk() -> Self {
        Self("(AFK)".to_string())
    }

    /// 获取应用名称字符串
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// 获取显示名称
    ///
    /// 去除路径、括号内容等
    pub fn display_name(&self) -> &str {
        // 尝试去除路径
        if let Some(last) = self.0.split('/').last() {
            if !last.is_empty() {
                // 去除括号内容
                if let Some(clean) = last.split('(').next() {
                    let clean = clean.trim();
                    if !clean.is_empty() {
                        return clean;
                    }
                }
                return last;
            }
        }

        // 去除括号内容
        if let Some(clean) = self.0.split('(').next() {
            let clean = clean.trim();
            if !clean.is_empty() {
                return clean;
            }
        }

        &self.0
    }

    /// 是否是 AFK
    pub fn is_afk(&self) -> bool {
        self.0 == "(AFK)" || self.0.to_lowercase().contains("afk")
    }
}

impl fmt::Display for AppName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<AppName> for String {
    fn from(name: AppName) -> String {
        name.0
    }
}

impl AsRef<str> for AppName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// 时间范围 - 值对象
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl TimeRange {
    /// 创建新的时间范围
    pub fn new(start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Self, DomainError> {
        if end < start {
            return Err(DomainError::InvalidTimeRange { start, end });
        }
        Ok(Self { start, end })
    }

    /// 今日时间范围
    pub fn today() -> Self {
        let now = Local::now();
        let start = now
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .with_timezone(&Utc);
        let end = start + ChronoDuration::days(1) - ChronoDuration::seconds(1);
        Self { start, end }
    }

    /// 昨日时间范围
    pub fn yesterday() -> Self {
        let now = Local::now();
        let yesterday = now.date_naive() - ChronoDuration::days(1);
        let start = yesterday
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .with_timezone(&Utc);
        let end = start + ChronoDuration::days(1) - ChronoDuration::seconds(1);
        Self { start, end }
    }

    /// 本周时间范围
    pub fn this_week() -> Self {
        let now = Local::now();
        let weekday = now.weekday().num_days_from_monday();
        let start = now
            .date_naive()
            - ChronoDuration::days(weekday as i64)
            - ChronoDuration::days(7); // 上周一
        let start = start
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .with_timezone(&Utc);
        let end = start + ChronoDuration::days(7) - ChronoDuration::seconds(1);
        Self { start, end }
    }

    /// 本月时间范围
    pub fn this_month() -> Self {
        let now = Local::now();
        let start = now
            .date_naive()
            .with_day(1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .with_timezone(&Utc);

        let next_month = if now.month() == 12 {
            now.with_month(1).unwrap().with_year(now.year() + 1).unwrap()
        } else {
            now.with_month(now.month() + 1).unwrap()
        };

        let end = next_month
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .with_timezone(&Utc)
            - ChronoDuration::seconds(1);

        Self { start, end }
    }

    /// 自定义时间范围
    pub fn custom(start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Self, DomainError> {
        Self::new(start, end)
    }

    /// 检查时间戳是否在范围内
    pub fn contains(&self, timestamp: &DateTime<Utc>) -> bool {
        *timestamp >= self.start && *timestamp <= self.end
    }

    /// 时间范围持续时间
    pub fn duration(&self) -> ChronoDuration {
        self.end - self.start
    }

    /// 范围内的天数
    pub fn days(&self) -> i64 {
        self.duration().num_days() + 1
    }
}

impl fmt::Display for TimeRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ~ {}",
            self.start.format("%Y-%m-%d %H:%M"),
            self.end.format("%Y-%m-%d %H:%M")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_event_creation() {
        let ts = Utc::now();
        let event = TimeEvent::new(ts, "firefox", 3600).unwrap();

        assert_eq!(event.app_name.as_str(), "firefox");
        assert_eq!(event.duration_secs(), 3600);
        assert!(!event.is_afk);
    }

    #[test]
    fn test_time_event_negative_duration() {
        let ts = Utc::now();
        let result = TimeEvent::new(ts, "firefox", -100);

        assert!(matches!(result, Err(DomainError::InvalidDuration(-100))));
    }

    #[test]
    fn test_app_name_validation() {
        assert!(AppName::new("").is_err());
        assert!(AppName::new("  ").is_err());
        assert!(AppName::new("firefox").is_ok());
        assert!(AppName::new("  vscode  ").is_ok()); // trim
    }

    #[test]
    fn test_app_name_display() {
        let name = AppName::new("/usr/bin/firefox").unwrap();
        assert_eq!(name.display_name(), "firefox");

        let name = AppName::new("code (main.rs)").unwrap();
        assert_eq!(name.display_name(), "code");
    }

    #[test]
    fn test_time_range_today() {
        let range = TimeRange::today();
        let now = Local::now();

        assert!(range.contains(&now.with_timezone(&Utc)));
    }

    #[test]
    fn test_time_range_invalid() {
        let start = Utc::now();
        let end = start - ChronoDuration::hours(1);

        let result = TimeRange::new(start, end);
        assert!(matches!(result, Err(DomainError::InvalidTimeRange { .. })));
    }

    #[test]
    fn test_event_time_indices() {
        let ts = "2024-01-15T14:30:00Z".parse::<DateTime<Utc>>().unwrap();
        let event = TimeEvent::new(ts, "test", 60).unwrap();

        // hour_index() 返回本地时区的小时数
        // 在 UTC+8 时区，14:00 UTC = 22:00 本地时间
        let local_hour = ts.with_timezone(&Local).hour() as usize;
        assert_eq!(event.hour_index(), local_hour);
        assert_eq!(event.minute_index(), 30);
    }

    #[test]
    fn test_event_weekday() {
        // 2024-01-15 是周一
        let ts = "2024-01-15T10:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let event = TimeEvent::new(ts, "test", 60).unwrap();

        assert_eq!(event.weekday_index(), 0); // 周一
    }
}
