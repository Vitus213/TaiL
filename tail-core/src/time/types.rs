//! 时间类型定义
//!
//! 提供强类型的时间表示，避免原始值混淆

use std::collections::HashMap;
use std::fmt;

/// 时间粒度
///
/// 定义数据聚合的时间维度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeGranularity {
    /// 年级（显示12个月）
    Year,
    /// 月级（显示该月的周）
    Month,
    /// 周级（显示7天）
    Week,
    /// 日级（显示24小时）
    Day,
    /// 小时级（显示60分钟）
    Hour,
}

impl TimeGranularity {
    /// 获取该粒度下的槽数量
    pub fn slot_count(&self) -> usize {
        match self {
            Self::Year => 12,
            Self::Month => 6, // 最多6周
            Self::Week => 7,
            Self::Day => 24,
            Self::Hour => 60,
        }
    }

    /// 获取该粒度的默认标签
    pub fn default_slot_label(&self, index: usize) -> String {
        match self {
            Self::Year => {
                let months = [
                    "1月", "2月", "3月", "4月", "5月", "6月", "7月", "8月", "9月", "10月", "11月",
                    "12月",
                ];
                months.get(index).map(|s| s.to_string()).unwrap_or_default()
            }
            Self::Month => format!("第{}周", index + 1),
            Self::Week => {
                let weekdays = ["周一", "周二", "周三", "周四", "周五", "周六", "周日"];
                weekdays
                    .get(index)
                    .map(|s| s.to_string())
                    .unwrap_or_default()
            }
            Self::Day => format!("{}h", index),
            Self::Hour => format!("{}m", index),
        }
    }
}

/// 时长
///
/// 内部存储为秒，提供便捷的转换和格式化方法
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Duration(i64);

impl Duration {
    /// 创建零时长
    pub fn zero() -> Self {
        Self(0)
    }

    /// 从秒数创建
    pub fn from_seconds(seconds: i64) -> Self {
        Self(seconds)
    }

    /// 从分钟创建
    pub fn from_minutes(minutes: i64) -> Self {
        Self(minutes * 60)
    }

    /// 从小时创建
    pub fn from_hours(hours: i64) -> Self {
        Self(hours * 3600)
    }

    /// 获取秒数
    pub fn as_seconds(&self) -> i64 {
        self.0
    }

    /// 获取分钟数（向下取整）
    pub fn as_minutes(&self) -> i64 {
        self.0 / 60
    }

    /// 获取小时数（向下取整）
    pub fn as_hours(&self) -> i64 {
        self.0 / 3600
    }

    /// 获取小时部分
    pub fn hours(&self) -> i64 {
        self.0 / 3600
    }

    /// 获取分钟部分（0-59）
    pub fn minutes(&self) -> i64 {
        (self.0 % 3600) / 60
    }

    /// 获取秒部分（0-59）
    pub fn seconds(&self) -> i64 {
        self.0 % 60
    }

    /// 检查是否为零
    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }

    /// 检查是否为正
    pub fn is_positive(&self) -> bool {
        self.0 > 0
    }

    /// 添加时长
    pub fn saturating_add(self, other: Duration) -> Duration {
        Self(self.0.saturating_add(other.0))
    }

    /// 格式化时长
    pub fn format(self, style: super::format::TimeFormatterStyle) -> String {
        super::format::TimeFormatter::format_duration(self, style)
    }
}

impl Default for Duration {
    fn default() -> Self {
        Self::zero()
    }
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.format(super::format::TimeFormatterStyle::Short)
        )
    }
}

impl std::ops::Add for Duration {
    type Output = Duration;

    fn add(self, other: Self) -> Self::Output {
        Self(self.0 + other.0)
    }
}

impl std::ops::AddAssign for Duration {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

impl std::ops::Mul<i64> for Duration {
    type Output = Duration;

    fn mul(self, rhs: i64) -> Self::Output {
        Self(self.0 * rhs)
    }
}

/// 单个时间槽
///
/// 表示特定时间粒度下的一个时间单位（如某小时、某天等）
/// 及其包含的各分组时长
#[derive(Debug, Clone)]
pub struct TimeSlot {
    /// 时间槽标签
    label: String,
    /// 时间槽索引
    index: usize,
    /// 各分组的时长（分组名 -> 秒数）
    group_durations: HashMap<String, i64>,
    /// 总时长
    total_duration: Duration,
}

impl TimeSlot {
    /// 创建新的时间槽
    pub fn new(label: String, index: usize) -> Self {
        Self {
            label,
            index,
            group_durations: HashMap::new(),
            total_duration: Duration::zero(),
        }
    }

    /// 获取标签
    pub fn label(&self) -> &str {
        &self.label
    }

    /// 获取索引
    pub fn index(&self) -> usize {
        self.index
    }

    /// 获取总时长
    pub fn duration(&self) -> Duration {
        self.total_duration
    }

    /// 添加某分组的时长
    ///
    /// # 参数
    /// - `group_name`: 分组名称（如应用名、分类名）
    /// - `seconds`: 要添加的秒数
    pub fn add_duration(&mut self, group_name: &str, seconds: i64) {
        if seconds <= 0 {
            return;
        }
        *self
            .group_durations
            .entry(group_name.to_string())
            .or_insert(0) += seconds;
        self.total_duration = Duration::from_seconds(self.total_duration.as_seconds() + seconds);
    }

    /// 获取所有分组及其时长
    pub fn group_durations(&self) -> &HashMap<String, i64> {
        &self.group_durations
    }

    /// 获取时长最高的N个分组
    ///
    /// 返回 Vec<(分组名, 秒数)>，按时长降序排列
    pub fn top_groups(&self, limit: usize) -> Vec<(String, i64)> {
        let mut groups: Vec<_> = self
            .group_durations
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        groups.sort_by(|a, b| b.1.cmp(&a.1));
        groups.truncate(limit);
        groups
    }

    /// 获取某分组的时长
    pub fn get_group_duration(&self, group_name: &str) -> i64 {
        self.group_durations.get(group_name).copied().unwrap_or(0)
    }
}

/// 时间槽集合
///
/// 表示特定时间粒度下的所有时间槽
#[derive(Debug, Clone)]
pub struct TimeSlots {
    /// 时间槽数据
    slots: Vec<TimeSlot>,
    /// 时间粒度
    granularity: TimeGranularity,
}

impl TimeSlots {
    /// 创建新的时间槽集合
    pub fn new(granularity: TimeGranularity) -> Self {
        Self {
            slots: Vec::new(),
            granularity,
        }
    }

    /// 获取粒度
    pub fn granularity(&self) -> TimeGranularity {
        self.granularity
    }

    /// 添加时间槽
    pub fn add_slot(&mut self, slot: TimeSlot) {
        self.slots.push(slot);
    }

    /// 获取所有时间槽
    pub fn slots(&self) -> &[TimeSlot] {
        &self.slots
    }

    /// 获取时间槽数量
    pub fn len(&self) -> usize {
        self.slots.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    /// 获取指定索引的时间槽
    pub fn get_slot(&self, index: usize) -> Option<&TimeSlot> {
        self.slots.get(index)
    }

    /// 获取指定索引的时间槽（可变）
    pub fn get_slot_mut(&mut self, index: usize) -> Option<&mut TimeSlot> {
        self.slots.get_mut(index)
    }

    /// 计算总时长
    ///
    /// 注意：这是所有时间槽的时长之和，不是去重的应用时长
    pub fn total_duration(&self) -> Duration {
        self.slots
            .iter()
            .map(|s| s.duration().as_seconds())
            .sum::<i64>()
            .pipe(Duration::from_seconds)
    }

    /// 获取最大时长（用于图表归一化）
    pub fn max_duration(&self) -> Duration {
        self.slots
            .iter()
            .map(|s| s.duration())
            .max()
            .unwrap_or(Duration::zero())
    }

    /// 获取所有出现过的分组名称
    pub fn all_groups(&self) -> Vec<String> {
        let groups: std::collections::HashSet<_> = self
            .slots
            .iter()
            .flat_map(|slot| slot.group_durations().keys())
            .cloned()
            .collect();
        let mut result: Vec<_> = groups.into_iter().collect();
        result.sort();
        result
    }
}

// 管道trait用于链式调用
trait Pipe<T> {
    fn pipe(self, f: impl FnOnce(Self) -> T) -> T
    where
        Self: Sized;
}

impl<S, T> Pipe<T> for S
where
    S: Sized,
{
    fn pipe(self, f: impl FnOnce(Self) -> T) -> T {
        f(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration_arithmetic() {
        let d1 = Duration::from_hours(1);
        let d2 = Duration::from_minutes(30);

        assert_eq!(d1.as_seconds(), 3600);
        assert_eq!(d2.as_seconds(), 1800);

        let sum = d1 + d2;
        assert_eq!(sum.as_seconds(), 5400);
        assert_eq!(sum.hours(), 1);
        assert_eq!(sum.minutes(), 30);
    }

    #[test]
    fn test_duration_zero() {
        let d = Duration::zero();
        assert!(d.is_zero());
        assert!(!d.is_positive());
    }

    #[test]
    fn test_time_slot_top_groups() {
        let mut slot = TimeSlot::new("测试".to_string(), 0);
        slot.add_duration("app1", 100);
        slot.add_duration("app2", 300);
        slot.add_duration("app3", 200);

        let top = slot.top_groups(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0], ("app2".to_string(), 300));
        assert_eq!(top[1], ("app3".to_string(), 200));
    }

    #[test]
    fn test_granularity_labels() {
        assert_eq!(TimeGranularity::Day.default_slot_label(0), "0h");
        assert_eq!(TimeGranularity::Day.default_slot_label(12), "12h");

        assert_eq!(TimeGranularity::Week.default_slot_label(0), "周一");
        assert_eq!(TimeGranularity::Week.default_slot_label(6), "周日");

        assert_eq!(TimeGranularity::Month.default_slot_label(0), "第1周");
        assert_eq!(TimeGranularity::Month.default_slot_label(3), "第4周");

        assert_eq!(TimeGranularity::Year.default_slot_label(0), "1月");
        assert_eq!(TimeGranularity::Year.default_slot_label(11), "12月");
    }
}
