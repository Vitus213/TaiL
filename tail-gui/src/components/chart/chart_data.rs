//! 通用图表数据结构
//!
//! 提供统一的图表数据接口，支持不同时间粒度和分组模式

use chrono::{DateTime, Datelike, Local, Timelike, Utc};
use std::collections::HashMap;
use tail_core::AppUsage;

/// 时间粒度
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartTimeGranularity {
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

impl ChartTimeGranularity {
    /// 获取时间槽数量
    pub fn slot_count(&self) -> usize {
        match self {
            Self::Year => 12,
            Self::Month => 6,  // 最多6周
            Self::Week => 7,
            Self::Day => 24,
            Self::Hour => 60,
        }
    }
}

/// 分组模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartGroupMode {
    /// 按应用分组
    ByApp,
    /// 按分类分组
    ByCategory,
}

/// 单个时间槽的数据
#[derive(Debug, Clone)]
pub struct ChartTimeSlot {
    /// 时间槽标签
    pub label: String,
    /// 时间槽索引
    pub index: usize,
    /// 该时间槽内各分组的时长（分组名 -> 秒数）
    pub group_durations: HashMap<String, i64>,
    /// 该时间槽的总时长
    pub total_seconds: i64,
}

impl ChartTimeSlot {
    pub fn new(label: String, index: usize) -> Self {
        Self {
            label,
            index,
            group_durations: HashMap::new(),
            total_seconds: 0,
        }
    }

    pub fn add_group(&mut self, group_name: String, seconds: i64) {
        self.total_seconds += seconds;
        *self.group_durations.entry(group_name).or_insert(0) += seconds;
    }

    /// 获取该时间槽内时长最高的分组
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
}

/// 图表数据
#[derive(Debug, Clone)]
pub struct ChartData {
    /// 时间槽数据
    pub time_slots: Vec<ChartTimeSlot>,
    /// 总时长
    pub total_seconds: i64,
    /// 时间粒度
    pub granularity: ChartTimeGranularity,
    /// 分组模式
    pub group_mode: ChartGroupMode,
}

impl ChartData {
    pub fn new(granularity: ChartTimeGranularity, group_mode: ChartGroupMode) -> Self {
        Self {
            time_slots: Vec::new(),
            total_seconds: 0,
            granularity,
            group_mode,
        }
    }

    /// 添加时间槽
    pub fn add_slot(&mut self, slot: ChartTimeSlot) {
        self.total_seconds += slot.total_seconds;
        self.time_slots.push(slot);
    }

    /// 获取最大时长（用于归一化）
    pub fn max_seconds(&self) -> i64 {
        self.time_slots
            .iter()
            .map(|s| s.total_seconds)
            .max()
            .unwrap_or(0)
    }

    /// 获取所有出现过的分组名称
    pub fn all_groups(&self) -> Vec<String> {
        let mut groups: std::collections::HashSet<_> = self
            .time_slots
            .iter()
            .flat_map(|slot| slot.group_durations.keys())
            .cloned()
            .collect();
        let mut result: Vec<_> = groups.into_iter().collect();
        result.sort();
        result
    }
}

/// 图表数据构建器
pub struct ChartDataBuilder<'a> {
    app_usage: &'a [AppUsage],
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    granularity: ChartTimeGranularity,
    group_mode: ChartGroupMode,
    category_cache: HashMap<String, Vec<String>>,
}

impl<'a> ChartDataBuilder<'a> {
    pub fn new(app_usage: &'a [AppUsage]) -> Self {
        Self {
            app_usage,
            start: DateTime::default(),
            end: DateTime::default(),
            granularity: ChartTimeGranularity::Day,
            group_mode: ChartGroupMode::ByApp,
            category_cache: HashMap::new(),
        }
    }

    pub fn with_time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.start = start;
        self.end = end;
        self
    }

    pub fn with_granularity(mut self, granularity: ChartTimeGranularity) -> Self {
        self.granularity = granularity;
        self
    }

    pub fn with_group_mode(mut self, mode: ChartGroupMode) -> Self {
        self.group_mode = mode;
        self
    }

    /// 检查是否需要根据时间范围过滤
    fn should_filter_by_time_range(&self) -> bool {
        // DateTime::default() 返回 1970-01-01 00:00:00 UTC
        // 只有当 start 和 end 都大于默认值时，才认为设置了有效的时间范围
        let default_dt = DateTime::<Utc>::default();
        self.start > default_dt && self.end > default_dt && self.start < self.end
    }

    /// 检查事件是否在时间范围内
    fn is_event_in_range(&self, event_timestamp: DateTime<Utc>) -> bool {
        if !self.should_filter_by_time_range() {
            return true; // 没有设置时间范围，接受所有事件
        }
        event_timestamp >= self.start && event_timestamp < self.end
    }

    /// 构建图表数据
    pub fn build(mut self) -> ChartData {
        // 如果是按分类分组，先加载分类信息
        if self.group_mode == ChartGroupMode::ByCategory {
            self.load_categories();
        }

        match self.granularity {
            ChartTimeGranularity::Day => self.build_day_slots(),
            ChartTimeGranularity::Week => self.build_week_slots(),
            ChartTimeGranularity::Month => self.build_month_slots(),
            ChartTimeGranularity::Year => self.build_year_slots(),
            ChartTimeGranularity::Hour => self.build_hour_slots(),
        }
    }

    fn load_categories(&mut self) {
        // 不再从数据库加载分类信息
        // 分类缓存将保持为空，应用将显示为"未分类"
        self.category_cache.clear();
    }

    /// 获取应用所属的分类名称
    fn get_app_categories(&self, app_name: &str) -> Vec<String> {
        self.category_cache
            .get(app_name)
            .cloned()
            .unwrap_or_else(|| vec!["未分类".to_string()])
    }

    /// 构建24小时时间槽（单日）
    fn build_day_slots(mut self) -> ChartData {
        let mut slots: Vec<ChartTimeSlot> = (0..24).map(|i| {
            ChartTimeSlot::new(format!("{}h", i), i)
        }).collect();

        for usage in self.app_usage {
            if usage.app_name.is_empty() {
                continue;
            }

            for event in &usage.window_events {
                if event.is_afk {
                    continue;
                }

                // 检查事件是否在时间范围内
                if !self.is_event_in_range(event.timestamp) {
                    continue;
                }

                let local_time = event.timestamp.with_timezone(&Local);
                let hour = local_time.hour() as usize;

                if hour < slots.len() {
                    let seconds = event.duration_secs;
                    match self.group_mode {
                        ChartGroupMode::ByApp => {
                            slots[hour].add_group(usage.app_name.clone(), seconds);
                        }
                        ChartGroupMode::ByCategory => {
                            let categories = self.get_app_categories(&usage.app_name);
                            for cat in &categories {
                                slots[hour].add_group(cat.clone(), seconds);
                            }
                        }
                    }
                }
            }
        }

        ChartData {
            time_slots: slots,
            total_seconds: self.app_usage.iter().map(|u| u.total_seconds).sum(),
            granularity: ChartTimeGranularity::Day,
            group_mode: self.group_mode,
        }
    }

    /// 构建7天时间槽（周）
    fn build_week_slots(mut self) -> ChartData {
        let weekday_labels = ["周一", "周二", "周三", "周四", "周五", "周六", "周日"];
        let mut slots: Vec<ChartTimeSlot> = (0..7).map(|i| {
            ChartTimeSlot::new(weekday_labels[i].to_string(), i)
        }).collect();

        for usage in self.app_usage {
            if usage.app_name.is_empty() {
                continue;
            }

            for event in &usage.window_events {
                if event.is_afk {
                    continue;
                }

                // 检查事件是否在时间范围内
                if !self.is_event_in_range(event.timestamp) {
                    continue;
                }

                let local_time = event.timestamp.with_timezone(&Local);
                let weekday = local_time.weekday().num_days_from_monday() as usize;
                let seconds = event.duration_secs;

                match self.group_mode {
                    ChartGroupMode::ByApp => {
                        slots[weekday].add_group(usage.app_name.clone(), seconds);
                    }
                    ChartGroupMode::ByCategory => {
                        let categories = self.get_app_categories(&usage.app_name);
                        for cat in &categories {
                            slots[weekday].add_group(cat.clone(), seconds);
                        }
                    }
                }
            }
        }

        ChartData {
            time_slots: slots,
            total_seconds: self.app_usage.iter().map(|u| u.total_seconds).sum(),
            granularity: ChartTimeGranularity::Week,
            group_mode: self.group_mode,
        }
    }

    /// 构建月份周数槽（月）
    fn build_month_slots(mut self) -> ChartData {
        let mut slots: Vec<ChartTimeSlot> = Vec::new();
        let mut weekly_data: HashMap<u32, ChartTimeSlot> = HashMap::new();

        for usage in self.app_usage {
            if usage.app_name.is_empty() {
                continue;
            }

            for event in &usage.window_events {
                if event.is_afk {
                    continue;
                }

                // 检查事件是否在时间范围内
                if !self.is_event_in_range(event.timestamp) {
                    continue;
                }

                let local_time = event.timestamp.with_timezone(&Local);
                let day = local_time.day();

                // 边界检查：确保日期有效 (1-31)
                if day == 0 {
                    continue;
                }

                let week = ((day - 1) / 7) + 1;

                // 边界检查：周数应该在 1-6 范围内
                if week == 0 || week > 6 {
                    continue;
                }

                let slot = weekly_data
                    .entry(week)
                    .or_insert_with(|| ChartTimeSlot::new(format!("第{}周", week), (week - 1) as usize));

                let seconds = event.duration_secs;
                match self.group_mode {
                    ChartGroupMode::ByApp => {
                        slot.add_group(usage.app_name.clone(), seconds);
                    }
                    ChartGroupMode::ByCategory => {
                        let categories = self.get_app_categories(&usage.app_name);
                        for cat in &categories {
                            slot.add_group(cat.clone(), seconds);
                        }
                    }
                }
            }
        }

        let mut sorted_slots: Vec<_> = weekly_data.into_values().collect();
        sorted_slots.sort_by_key(|s| s.index);
        slots = sorted_slots;

        ChartData {
            time_slots: slots,
            total_seconds: self.app_usage.iter().map(|u| u.total_seconds).sum(),
            granularity: ChartTimeGranularity::Month,
            group_mode: self.group_mode,
        }
    }

    /// 构建12个月槽（年）
    fn build_year_slots(mut self) -> ChartData {
        let month_labels = [
            "1月", "2月", "3月", "4月", "5月", "6月",
            "7月", "8月", "9月", "10月", "11月", "12月",
        ];
        let mut slots: Vec<ChartTimeSlot> = (0..12).map(|i| {
            ChartTimeSlot::new(month_labels[i].to_string(), i)
        }).collect();

        for usage in self.app_usage {
            if usage.app_name.is_empty() {
                continue;
            }

            for event in &usage.window_events {
                if event.is_afk {
                    continue;
                }

                // 检查事件是否在时间范围内
                if !self.is_event_in_range(event.timestamp) {
                    continue;
                }

                let local_time = event.timestamp.with_timezone(&Local);
                let month = local_time.month() as usize;

                // 边界检查：确保月份索引在有效范围内 (1-12 -> 0-11)
                if month == 0 || month > 12 {
                    continue;
                }
                let month_idx = month - 1;

                let seconds = event.duration_secs;
                match self.group_mode {
                    ChartGroupMode::ByApp => {
                        slots[month_idx].add_group(usage.app_name.clone(), seconds);
                    }
                    ChartGroupMode::ByCategory => {
                        let categories = self.get_app_categories(&usage.app_name);
                        for cat in &categories {
                            slots[month_idx].add_group(cat.clone(), seconds);
                        }
                    }
                }
            }
        }

        ChartData {
            time_slots: slots,
            total_seconds: self.app_usage.iter().map(|u| u.total_seconds).sum(),
            granularity: ChartTimeGranularity::Year,
            group_mode: self.group_mode,
        }
    }

    /// 构建60分钟槽（小时）
    fn build_hour_slots(mut self) -> ChartData {
        let mut slots: Vec<ChartTimeSlot> = (0..60).map(|i| {
            ChartTimeSlot::new(format!("{}m", i), i)
        }).collect();

        for usage in self.app_usage {
            if usage.app_name.is_empty() {
                continue;
            }

            for event in &usage.window_events {
                if event.is_afk {
                    continue;
                }

                // 检查事件是否在时间范围内
                if !self.is_event_in_range(event.timestamp) {
                    continue;
                }

                let local_time = event.timestamp.with_timezone(&Local);
                let minute = local_time.minute() as usize;

                let seconds = event.duration_secs;
                match self.group_mode {
                    ChartGroupMode::ByApp => {
                        slots[minute].add_group(usage.app_name.clone(), seconds);
                    }
                    ChartGroupMode::ByCategory => {
                        let categories = self.get_app_categories(&usage.app_name);
                        for cat in &categories {
                            slots[minute].add_group(cat.clone(), seconds);
                        }
                    }
                }
            }
        }

        ChartData {
            time_slots: slots,
            total_seconds: self.app_usage.iter().map(|u| u.total_seconds).sum(),
            granularity: ChartTimeGranularity::Hour,
            group_mode: self.group_mode,
        }
    }
}

/// 分类颜色配置
#[derive(Debug, Clone)]
pub struct CategoryColorMap {
    colors: HashMap<String, egui::Color32>,
    default_colors: Vec<egui::Color32>,
    other_color: egui::Color32,
}

impl Default for CategoryColorMap {
    fn default() -> Self {
        let mut colors = HashMap::new();
        colors.insert("工作".to_string(), egui::Color32::from_rgb(74, 144, 226));
        colors.insert("开发".to_string(), egui::Color32::from_rgb(52, 168, 83));
        colors.insert("娱乐".to_string(), egui::Color32::from_rgb(255, 99, 71));
        colors.insert("社交".to_string(), egui::Color32::from_rgb(155, 89, 182));
        colors.insert("学习".to_string(), egui::Color32::from_rgb(255, 205, 86));
        colors.insert("未分类".to_string(), egui::Color32::from_gray(150));

        let default_colors = vec![
            egui::Color32::from_rgb(74, 144, 226),
            egui::Color32::from_rgb(52, 168, 83),
            egui::Color32::from_rgb(255, 205, 86),
            egui::Color32::from_rgb(255, 99, 71),
            egui::Color32::from_rgb(155, 89, 182),
            egui::Color32::from_rgb(255, 152, 0),
            egui::Color32::from_rgb(220, 57, 218),
            egui::Color32::from_rgb(0, 200, 150),
        ];

        Self {
            colors,
            default_colors,
            other_color: egui::Color32::from_gray(150),
        }
    }
}

impl CategoryColorMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, category: String, color: egui::Color32) {
        self.colors.insert(category, color);
    }

    pub fn get(&self, category: &str) -> Option<egui::Color32> {
        self.colors.get(category).copied()
    }

    /// 获取"其他"分类的颜色
    pub fn other_color(&self) -> egui::Color32 {
        self.other_color
    }

    /// 为分组分配颜色
    pub fn assign_colors(&self, groups: &[String]) -> HashMap<String, egui::Color32> {
        let mut result = HashMap::new();
        for (idx, group) in groups.iter().enumerate() {
            let color = self
                .colors
                .get(group)
                .copied()
                .unwrap_or_else(|| {
                    self.default_colors
                        .get(idx % self.default_colors.len())
                        .copied()
                        .unwrap_or(self.other_color)
                });
            result.insert(group.clone(), color);
        }
        result
    }
}
