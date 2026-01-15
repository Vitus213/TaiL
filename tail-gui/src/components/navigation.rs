//! TaiL GUI - 导航模式

/// 导航模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NavigationMode {
    /// 侧边栏导航（Tai 风格，默认）
    #[default]
    Sidebar,
    /// 顶部 Tab 导航（传统风格）
    TopTab,
}

/// 视图类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Dashboard,
    Statistics,
    Details,
    Categories,
    Settings,
}

impl View {
    pub const ALL: &[View] = &[
        View::Dashboard,
        View::Statistics,
        View::Details,
        View::Categories,
        View::Settings,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            View::Dashboard => "仪表板",
            View::Statistics => "统计",
            View::Details => "详细",
            View::Categories => "分类",
            View::Settings => "设置",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            View::Dashboard => "📊",
            View::Statistics => "📈",
            View::Details => "📋",
            View::Categories => "📂",
            View::Settings => "⚙",
        }
    }

    /// 侧边栏显示的图标（更简洁）
    pub fn sidebar_icon(&self) -> &'static str {
        match self {
            View::Dashboard => "⊞",
            View::Statistics => "≣",
            View::Details => "≡",
            View::Categories => "⌘",
            View::Settings => "⚙",
        }
    }
}
