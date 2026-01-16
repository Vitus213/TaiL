//! TaiL Core - 统一时间记录模块
//!
//! # 设计目标
//!
//! 1. **单一数据源**: 所有时间计算都通过此模块，确保一致性
//! 2. **可测试性**: 纯函数设计，便于单元测试
//! 3. **类型安全**: 使用强类型避免混淆
//! 4. **清晰的API**: 函数名明确表达意图
//!
//! # 模块结构
//!
//! - `types`: 时间相关的核心类型定义
//! - `format`: 时间格式化工具
//! - `aggregate`: 时间聚合计算
//! - `range`: 时间范围计算

pub mod aggregate;
pub mod format;
pub mod range;
pub mod types;

// 重新导出常用类型
pub use types::{Duration, TimeGranularity, TimeSlot, TimeSlots};

/// 时间记录模块的预导出
pub mod prelude {
    pub use super::format::{TimeFormatter, TimeFormatterStyle};
    pub use super::range::TimeRange;
    pub use super::types::{Duration, TimeGranularity, TimeSlot, TimeSlots};
}

#[cfg(test)]
mod tests {
    use super::*;
    use format::TimeFormatterStyle;

    #[test]
    fn test_duration_creation() {
        let d = Duration::from_seconds(3665);
        assert_eq!(d.as_seconds(), 3665);
        assert_eq!(d.hours(), 1);
        assert_eq!(d.minutes(), 1);
        assert_eq!(d.seconds(), 5);
    }

    #[test]
    fn test_duration_formatting() {
        let d = Duration::from_seconds(3665);
        assert_eq!(d.format(TimeFormatterStyle::Short), "1h 1m");
        assert_eq!(d.format(TimeFormatterStyle::Full), "1h 1m 5s");
        assert_eq!(d.format(TimeFormatterStyle::Chinese), "1小时1分钟");
    }

    #[test]
    fn test_y_axis_tick_formatting() {
        use format::TimeFormatter;

        // Y轴刻度专用格式化
        assert_eq!(TimeFormatter::format_y_axis(30), "30s"); // 秒显示为秒
        assert_eq!(TimeFormatter::format_y_axis(60), "1m"); // 60秒 = 1分钟
        assert_eq!(TimeFormatter::format_y_axis(300), "5m"); // 5分钟
        assert_eq!(TimeFormatter::format_y_axis(3600), "1h"); // 1小时
        assert_eq!(TimeFormatter::format_y_axis(5400), "1h 30m"); // 1.5小时
        assert_eq!(TimeFormatter::format_y_axis(7200), "2h"); // 2小时
    }

    #[test]
    fn test_time_slot_creation() {
        let mut slot = TimeSlot::new("周一".to_string(), 0);
        assert_eq!(slot.label(), "周一");
        assert_eq!(slot.index(), 0);
        assert_eq!(slot.duration(), Duration::zero());

        slot.add_duration("app1", 100);
        assert_eq!(slot.duration().as_seconds(), 100);
    }

    #[test]
    fn test_time_slots_total() {
        let mut slots = TimeSlots::new(TimeGranularity::Day);
        slots.add_slot(TimeSlot::new("0h".to_string(), 0));
        slots.add_slot(TimeSlot::new("1h".to_string(), 1));

        slots.get_slot_mut(0).unwrap().add_duration("app1", 3600);
        slots.get_slot_mut(1).unwrap().add_duration("app2", 7200);

        assert_eq!(slots.total_duration().as_seconds(), 10800);
        assert_eq!(slots.max_duration().as_seconds(), 7200);
    }
}
