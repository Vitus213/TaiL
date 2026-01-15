//! TaiL GUI - UI 图标常量
//! 
//! 这些图标使用 egui 默认字体支持的 emoji
//! 参考: https://docs.rs/egui/latest/egui/special_emojis/index.html
//! 
//! 你可以修改这些常量来自定义 UI 图标

/// 分类视图图标
pub mod categories {
    /// 页面标题图标
    pub const PAGE_ICON: &str = "🗀";
    /// 分类总数图标
    pub const CATEGORY_COUNT: &str = "🗁";
    /// 已分类应用数图标
    pub const APP_COUNT: &str = "🖵";
    /// 总使用时间图标
    pub const TOTAL_TIME: &str = "🕓";
    /// 空状态图标
    pub const EMPTY_STATE: &str = "📋";
}

/// 仪表盘视图图标
pub mod dashboard {
    /// 页面标题图标
    pub const PAGE_ICON: &str = "📊";
    /// 今日使用时间图标
    pub const TODAY_TIME: &str = "🕓";
    /// 活跃应用数图标
    pub const ACTIVE_APPS: &str = "🖵";
}

/// 统计视图图标
pub mod statistics {
    /// 页面标题图标
    pub const PAGE_ICON: &str = "📈";
}

/// 设置视图图标
pub mod settings {
    /// 页面标题图标
    pub const PAGE_ICON: &str = "⛶";
}

/// 时间选择器图标
pub mod time_selector {
    /// 今天图标
    pub const TODAY: &str = "📅";
    /// 昨天图标
    pub const YESTERDAY: &str = "📆";
    /// 本周图标
    pub const THIS_WEEK: &str = "📊";
    /// 本月图标
    pub const THIS_MONTH: &str = "📈";
}

/// 通用图标
pub mod common {
    /// 删除图标
    pub const DELETE: &str = "🗑";
    /// 编辑图标
    pub const EDIT: &str = "✔";
    /// 添加图标
    pub const ADD: &str = "+";
    /// 关闭图标
    pub const CLOSE: &str = "✕";
    /// 搜索图标
    pub const SEARCH: &str = "🔍";
    /// 刷新图标
    pub const REFRESH: &str = "↻";
    /// 设置图标
    pub const SETTINGS: &str = "⛶";
    /// 警告图标
    pub const WARNING: &str = "❓";
    /// 成功图标
    pub const SUCCESS: &str = "✔";
    /// 错误图标
    pub const ERROR: &str = "🚫";
}
