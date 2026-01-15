//! TaiL Core - 数据模型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 窗口事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowEvent {
    pub id: Option<i64>,
    pub timestamp: DateTime<Utc>,
    pub app_name: String,
    pub window_title: String,
    pub workspace: String,
    pub duration_secs: i64,
    pub is_afk: bool,
}

/// AFK 事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AfkEvent {
    pub id: Option<i64>,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_secs: i64,
}

/// 每日目标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyGoal {
    pub id: Option<i64>,
    pub app_name: String,
    pub max_minutes: i32,
    pub notify_enabled: bool,
}

/// 应用使用统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUsage {
    pub app_name: String,
    pub total_seconds: i64,
    pub window_events: Vec<WindowEvent>,
}

/// 时间范围
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TimeRange {
    Today,
    Yesterday,
    Last7Days,
    Last30Days,
    Custom(DateTime<Utc>, DateTime<Utc>),
}

/// 时间导航层级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeNavigationLevel {
    /// 年份视图 - 显示多年的柱形图
    Year,
    /// 月份视图 - 显示12个月的柱形图
    Month,
    /// 周视图 - 显示4-5周的柱形图
    Week,
    /// 天视图 - 显示7天的柱形图
    Day,
    /// 小时视图 - 显示24小时的柱形图
    Hour,
}

impl Default for TimeNavigationLevel {
    fn default() -> Self {
        Self::Year
    }
}

/// 时间导航状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeNavigationState {
    /// 当前导航层级
    pub level: TimeNavigationLevel,
    /// 选中的年份
    pub selected_year: i32,
    /// 选中的月份 (1-12)
    pub selected_month: Option<u32>,
    /// 选中的周 (1-5)
    pub selected_week: Option<u32>,
    /// 选中的日期
    pub selected_day: Option<u32>,
}

impl TimeNavigationState {
    /// 创建新的导航状态，默认为年份视图
    pub fn new(current_year: i32) -> Self {
        Self {
            level: TimeNavigationLevel::Year,
            selected_year: current_year,
            selected_month: None,
            selected_week: None,
            selected_day: None,
        }
    }

    /// 返回上一级
    pub fn go_back(&mut self) {
        match self.level {
            TimeNavigationLevel::Year => {}
            TimeNavigationLevel::Month => {
                self.level = TimeNavigationLevel::Year;
                self.selected_month = None;
            }
            TimeNavigationLevel::Week => {
                self.level = TimeNavigationLevel::Month;
                self.selected_week = None;
            }
            TimeNavigationLevel::Day => {
                self.level = TimeNavigationLevel::Week;
                self.selected_day = None;
            }
            TimeNavigationLevel::Hour => {
                self.level = TimeNavigationLevel::Day;
            }
        }
    }

    /// 进入年份的月份视图
    pub fn drill_into_year(&mut self, year: i32) {
        self.selected_year = year;
        self.level = TimeNavigationLevel::Month;
    }

    /// 进入月份的周视图
    pub fn drill_into_month(&mut self, month: u32) {
        self.selected_month = Some(month);
        self.level = TimeNavigationLevel::Week;
    }

    /// 进入周的天视图
    pub fn drill_into_week(&mut self, week: u32) {
        self.selected_week = Some(week);
        self.level = TimeNavigationLevel::Day;
    }

    /// 进入天的小时视图
    pub fn drill_into_day(&mut self, day: u32) {
        self.selected_day = Some(day);
        self.level = TimeNavigationLevel::Hour;
    }

    /// 跳转到今天
    pub fn go_to_today(&mut self, year: i32, month: u32, day: u32) {
        self.selected_year = year;
        self.selected_month = Some(month);
        self.selected_day = Some(day);
        self.selected_week = None;
        self.level = TimeNavigationLevel::Hour;
    }

    /// 获取当前路径的显示文本
    pub fn get_breadcrumb(&self) -> String {
        let mut parts = vec![format!("{}年", self.selected_year)];
        
        if let Some(month) = self.selected_month {
            parts.push(format!("{}月", month));
        }
        if let Some(week) = self.selected_week {
            parts.push(format!("第{}周", week));
        }
        if let Some(day) = self.selected_day {
            parts.push(format!("{}日", day));
        }
        
        parts.join(" > ")
    }
}

/// 统计视图模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum StatisticsViewMode {
    /// 按应用显示
    #[default]
    ByApp,
    /// 按分类显示
    ByCategory,
}

/// 时间段使用统计（用于柱形图）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodUsage {
    /// 时间段标签（如 "2026年"、"1月"、"第1周"、"周一"、"9时"）
    pub label: String,
    /// 时间段索引（用于点击时识别）
    pub index: i32,
    /// 总使用时间（秒）
    pub total_seconds: i64,
}

/// 应用分类
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: Option<i64>,
    pub name: String,
    pub icon: String,  // emoji 图标
    pub color: Option<String>,
}

/// 应用-分类关联
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppCategory {
    pub id: Option<i64>,
    pub app_name: String,
    pub category_id: i64,
}

/// 分类使用统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryUsage {
    pub category: Category,
    pub total_seconds: i64,
    pub app_count: usize,
    pub apps: Vec<AppUsageInCategory>,
}

/// 分类中的应用使用统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUsageInCategory {
    pub app_name: String,
    pub total_seconds: i64,
}

/// 预设的分类图标列表（使用 egui 默认字体支持的 emoji）
/// 参考: https://docs.rs/egui/latest/egui/special_emojis/index.html
pub const CATEGORY_ICONS: &[&str] = &[
    // 文件夹和文档
    "🗀", "🗁", "🗋", "🗐", "📋", "📌", "📎",
    // 图表和统计
    "📈", "📉", "📊",
    // 日历和时间
    "📅", "📆", "🕓",
    // 媒体控制
    "⏵", "⏸", "⏹", "⏺", "⏏", "▶", "■",
    // 导航箭头
    "⬅", "➡", "⬆", "⬇", "↺", "↻", "⟲", "⟳",
    // 搜索和链接
    "🔍", "🔎", "🔗", "🔘",
    // 音量
    "🔈", "🔉", "🔊", "🔆",
    // 设备
    "🖧", "🖩", "🖮", "🖱", "🖴", "🖵", "🖼",
    // 状态和选择
    "☐", "☑", "✔", "★", "☆", "♡",
    // 天气和符号
    "☀", "☁", "⛃", "⛶",
    // 其他
    "🗑", "🗙", "🚫", "❓", "∞", "⊗",
    // 传输
    "📤", "📥", "🔀", "🔁", "🔃",
];
