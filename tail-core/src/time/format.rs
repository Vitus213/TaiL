//! 时间格式化工具
//!
//! 提供统一的时间格式化接口，确保整个应用的时间显示一致
//!
//! # 设计原则
//!
//! 1. **单一职责**: 每个函数只负责一种格式化
//! 2. **明确命名**: 函数名清楚表明格式化用途
//! 3. **零歧义**: Y轴刻度格式化必须区分秒/分钟/小时

use crate::time::types::Duration;

/// 时间格式化风格
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeFormatterStyle {
    /// 简短格式：1h 30m 或 30m 15s 或 15s
    Short,
    /// 完整格式：1h 30m 15s
    Full,
    /// 中文格式：1小时30分钟
    Chinese,
    /// 极简格式：1h 或 30m
    Minimal,
}

/// Y轴刻度格式化风格
///
/// Y轴需要特殊的格式化逻辑：
/// - 必须清晰区分秒、分钟、小时
/// - 小时可以显示小数（如 1.5h）
/// - 避免使用"Xm"这种容易混淆的格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YAxisStyle {
    /// 自动选择最合适的单位
    Auto,
    /// 始终显示为小时（可带小数）
    Hours,
    /// 始终显示为分钟
    Minutes,
    /// 始终显示为秒
    Seconds,
}

/// 时间格式化器
///
/// 提供静态方法进行各种时间格式化
pub struct TimeFormatter;

impl TimeFormatter {
    /// 格式化时长
    ///
    /// # 参数
    /// - `duration`: 时长
    /// - `style`: 格式化风格
    ///
    /// # 示例
    /// ```
    /// use tail_core::time::{Duration, format::{TimeFormatter, TimeFormatterStyle}};
    ///
    /// let d = Duration::from_seconds(3665);
    /// assert_eq!(TimeFormatter::format_duration(d, TimeFormatterStyle::Short), "1h 1m");
    /// assert_eq!(TimeFormatter::format_duration(d, TimeFormatterStyle::Full), "1h 1m 5s");
    /// assert_eq!(TimeFormatter::format_duration(d, TimeFormatterStyle::Chinese), "1小时1分钟");
    /// ```
    pub fn format_duration(duration: Duration, style: TimeFormatterStyle) -> String {
        let secs = duration.as_seconds();
        let hours = duration.hours();
        let minutes = duration.minutes();
        let seconds = duration.seconds();

        match style {
            TimeFormatterStyle::Short => {
                if hours > 0 {
                    format!("{}h {}m", hours, minutes)
                } else if minutes > 0 {
                    format!("{}m {}s", minutes, seconds)
                } else {
                    format!("{}s", seconds)
                }
            }
            TimeFormatterStyle::Full => {
                if hours > 0 {
                    format!("{}h {}m {}s", hours, minutes, seconds)
                } else if minutes > 0 {
                    format!("{}m {}s", minutes, seconds)
                } else {
                    format!("{}s", seconds)
                }
            }
            TimeFormatterStyle::Chinese => {
                if hours > 0 {
                    format!("{}小时{}分钟", hours, minutes)
                } else {
                    format!("{}分钟", minutes)
                }
            }
            TimeFormatterStyle::Minimal => {
                if hours > 0 {
                    format!("{}h", hours)
                } else {
                    format!("{}m", minutes)
                }
            }
        }
    }

    /// 格式化秒数为时长（兼容旧API）
    #[inline]
    pub fn format_seconds(seconds: i64, style: TimeFormatterStyle) -> String {
        Self::format_duration(Duration::from_seconds(seconds), style)
    }

    /// Y轴刻度专用格式化
    ///
    /// **重要**: 此函数专门用于图表Y轴，必须明确区分单位
    ///
    /// # 规则
    /// - < 60秒: 显示为 "Xs" (秒)
    /// - >= 60秒且 < 3600秒: 显示为 "Xm" (分钟)
    /// - >= 3600秒: 显示为小时，可带小数
    ///
    /// # 示例
    /// ```
    /// use tail_core::time::format::TimeFormatter;
    ///
    /// assert_eq!(TimeFormatter::format_y_axis(30), "30s");      // 30秒
    /// assert_eq!(TimeFormatter::format_y_axis(60), "1m");       // 1分钟
    /// assert_eq!(TimeFormatter::format_y_axis(300), "5m");      // 5分钟
    /// assert_eq!(TimeFormatter::format_y_axis(3600), "1h");     // 1小时
    /// assert_eq!(TimeFormatter::format_y_axis(5400), "1h 30m"); // 1.5小时
    /// assert_eq!(TimeFormatter::format_y_axis(7200), "2h");     // 2小时
    /// ```
    pub fn format_y_axis(seconds: i64) -> String {
        if seconds < 60 {
            // 秒：明确显示 "Xs"
            format!("{}s", seconds)
        } else if seconds < 3600 {
            // 分钟：显示 "Xm"
            let mins = seconds / 60;
            format!("{}m", mins)
        } else {
            // 小时：根据小数部分决定格式
            let hours = seconds as f64 / 3600.0;
            let whole_hours = hours.floor() as i64;
            let remaining_minutes = ((hours - whole_hours as f64) * 60.0).round() as i64;

            if remaining_minutes == 0 {
                format!("{}h", whole_hours)
            } else if remaining_minutes == 30 {
                format!("{}h 30m", whole_hours)
            } else {
                // 其他情况显示小数小时
                if hours.fract() < 0.1 || hours.fract() > 0.9 {
                    format!("{}h", hours.round() as i32)
                } else if hours.fract() < 0.6 {
                    format!("{}h", hours as i32)
                } else {
                    format!("{:.1}h", hours)
                }
            }
        }
    }

    /// Y轴刻度格式化（指定风格）
    pub fn format_y_axis_with_style(seconds: i64, style: YAxisStyle) -> String {
        match style {
            YAxisStyle::Auto => Self::format_y_axis(seconds),
            YAxisStyle::Hours => {
                let hours = seconds as f64 / 3600.0;
                if hours.fract() < 0.01 {
                    format!("{}h", hours as i32)
                } else {
                    format!("{:.1}h", hours)
                }
            }
            YAxisStyle::Minutes => {
                format!("{}m", seconds / 60)
            }
            YAxisStyle::Seconds => {
                format!("{}s", seconds)
            }
        }
    }

    /// 从分钟格式化（用于目标设置等场景）
    pub fn format_minutes(minutes: i32, style: TimeFormatterStyle) -> String {
        Self::format_seconds(minutes as i64 * 60, style)
    }

    /// 格式化时间范围
    ///
    /// 将两个时间点格式化为可读范围
    pub fn format_time_range(start: i64, end: i64) -> String {
        format!("{} - {}",
            Self::format_duration(Duration::from_seconds(start), TimeFormatterStyle::Minimal),
            Self::format_duration(Duration::from_seconds(end), TimeFormatterStyle::Minimal))
    }
}

// 注意：Duration 的 format 方法已在 types.rs 中实现

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration_short() {
        assert_eq!(TimeFormatter::format_seconds(0, TimeFormatterStyle::Short), "0s");
        assert_eq!(TimeFormatter::format_seconds(30, TimeFormatterStyle::Short), "30s");
        assert_eq!(TimeFormatter::format_seconds(59, TimeFormatterStyle::Short), "59s");
        assert_eq!(TimeFormatter::format_seconds(60, TimeFormatterStyle::Short), "1m 0s");
        assert_eq!(TimeFormatter::format_seconds(125, TimeFormatterStyle::Short), "2m 5s");
        assert_eq!(TimeFormatter::format_seconds(3665, TimeFormatterStyle::Short), "1h 1m");
    }

    #[test]
    fn test_format_duration_full() {
        assert_eq!(TimeFormatter::format_seconds(3665, TimeFormatterStyle::Full), "1h 1m 5s");
        assert_eq!(TimeFormatter::format_seconds(60, TimeFormatterStyle::Full), "1m 0s");
        assert_eq!(TimeFormatter::format_seconds(59, TimeFormatterStyle::Full), "59s");
    }

    #[test]
    fn test_format_duration_chinese() {
        assert_eq!(TimeFormatter::format_seconds(3665, TimeFormatterStyle::Chinese), "1小时1分钟");
        assert_eq!(TimeFormatter::format_seconds(60, TimeFormatterStyle::Chinese), "1分钟");
        assert_eq!(TimeFormatter::format_seconds(3600, TimeFormatterStyle::Chinese), "1小时0分钟");
    }

    #[test]
    fn test_format_duration_minimal() {
        assert_eq!(TimeFormatter::format_seconds(3665, TimeFormatterStyle::Minimal), "1h");
        assert_eq!(TimeFormatter::format_seconds(300, TimeFormatterStyle::Minimal), "5m");
        assert_eq!(TimeFormatter::format_seconds(59, TimeFormatterStyle::Minimal), "0m"); // 不到1分钟
    }

    #[test]
    fn test_format_y_axis() {
        // 秒：必须显示 "Xs"，不能是 "Xm"
        assert_eq!(TimeFormatter::format_y_axis(0), "0s");
        assert_eq!(TimeFormatter::format_y_axis(30), "30s");
        assert_eq!(TimeFormatter::format_y_axis(59), "59s");

        // 分钟：显示 "Xm"
        assert_eq!(TimeFormatter::format_y_axis(60), "1m");
        assert_eq!(TimeFormatter::format_y_axis(300), "5m");
        assert_eq!(TimeFormatter::format_y_axis(600), "10m");
        assert_eq!(TimeFormatter::format_y_axis(3599), "59m");

        // 小时：显示 "Xh" 或 "Xh Xm"
        assert_eq!(TimeFormatter::format_y_axis(3600), "1h");
        assert_eq!(TimeFormatter::format_y_axis(5400), "1h 30m");
        assert_eq!(TimeFormatter::format_y_axis(7200), "2h");
        assert_eq!(TimeFormatter::format_y_axis(9000), "2h 30m");

        // 测试边界值
        assert_eq!(TimeFormatter::format_y_axis(60), "1m");       // 临界：秒->分钟
        assert_eq!(TimeFormatter::format_y_axis(3600), "1h");     // 临界：分钟->小时
    }

    #[test]
    fn test_y_axis_with_styles() {
        // 3665秒 = 1h 1m 5s
        // Auto style: 小数部分很小，会四舍五入为 "1h"
        assert_eq!(TimeFormatter::format_y_axis_with_style(3665, YAxisStyle::Auto), "1h");
        // Hours style: 小数部分 > 0.01，显示一位小数
        assert_eq!(TimeFormatter::format_y_axis_with_style(3665, YAxisStyle::Hours), "1.0h");
        // 完整的小时没有小数
        assert_eq!(TimeFormatter::format_y_axis_with_style(3600, YAxisStyle::Hours), "1h");
        assert_eq!(TimeFormatter::format_y_axis_with_style(3665, YAxisStyle::Minutes), "61m");
        assert_eq!(TimeFormatter::format_y_axis_with_style(3665, YAxisStyle::Seconds), "3665s");
    }

    #[test]
    fn test_duration_display_trait() {
        let d = Duration::from_seconds(3665);
        assert_eq!(format!("{}", d), "1h 1m");
    }
}
