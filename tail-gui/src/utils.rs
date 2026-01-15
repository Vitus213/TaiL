//! TaiL GUI - 工具函数
//!
//! GUI 层的通用工具函数，封装 tail_core 的工具模块

/// 时长格式化工具
pub mod duration {
    use tail_core::utils::duration::DurationStyle;

    /// 格式化秒数为简短时长 (1h 30m)
    pub fn format_duration(seconds: i64) -> String {
        tail_core::utils::duration::format_duration(seconds, DurationStyle::Short)
    }

    /// 格式化秒数为完整时长 (1h 30m 15s)
    pub fn format_duration_full(seconds: i64) -> String {
        tail_core::utils::duration::format_duration(seconds, DurationStyle::Full)
    }

    /// 格式化秒数为极简时长 (1h)
    pub fn format_duration_short(seconds: i64) -> String {
        tail_core::utils::duration::format_duration(seconds, DurationStyle::Minimal)
    }

    /// 格式化秒数为中文时长 (1小时30分钟)
    pub fn format_duration_chinese(seconds: i64) -> String {
        tail_core::utils::duration::format_duration(seconds, DurationStyle::Chinese)
    }

    /// 从分钟格式化（用于目标进度等场景）
    pub fn format_minutes(minutes: i32) -> String {
        tail_core::utils::duration::format_minutes(minutes, DurationStyle::Short)
    }

    /// 从分钟格式化为中文（用于目标进度等场景）
    pub fn format_minutes_chinese(minutes: i32) -> String {
        tail_core::utils::duration::format_minutes(minutes, DurationStyle::Chinese)
    }
}

/// 时间范围计算工具（重新导出）
pub mod time_range {
    pub use tail_core::utils::time_range::*;
}

/// 数据过滤工具（重新导出）
pub mod filter {
    pub use tail_core::utils::filter::*;
}

#[cfg(test)]
mod tests {
    use super::duration::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(0), "0s");
        assert_eq!(format_duration(59), "59s");
        assert_eq!(format_duration(60), "1m 0s");
        assert_eq!(format_duration(3665), "1h 1m");
    }

    #[test]
    fn test_format_duration_full() {
        assert_eq!(format_duration_full(3665), "1h 1m 5s");
    }

    #[test]
    fn test_format_duration_short() {
        assert_eq!(format_duration_short(3665), "1h");
        assert_eq!(format_duration_short(300), "5m");
    }

    #[test]
    fn test_format_duration_chinese() {
        assert_eq!(format_duration_chinese(3665), "1小时1分钟");
        assert_eq!(format_duration_chinese(60), "1分钟");
    }
}
