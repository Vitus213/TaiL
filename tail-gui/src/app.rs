//! TaiL GUI - egui 应用
//!
//! 重构版本：使用 AsyncBridge 替代阻塞的 tokio runtime

use chrono::{DateTime, Datelike, Duration as ChronoDuration, Local, Utc};
use std::sync::Arc;
use tail_core::db::Config as DbConfig;
use tail_core::models::{TimeNavigationState, TimeRange};
use tail_core::{AppUsage, DailyGoal, Repository};
use tracing::{debug, info};

use crate::async_bridge::{AsyncBridge, CategoriesDataPayload, GuiCommand, GuiResponse};
use crate::components::{
    AliasDialog, DefaultStatsView, NavigationMode, SidebarNav, TopTabNav, View,
};
use crate::icons::IconCache;
use crate::theme::{TaiLTheme, ThemeType};
use crate::views::{
    AddGoalDialog, CategoriesView, CategoryAction, DashboardView, DetailsView, SettingsAction,
    SettingsView, StatisticsView,
};

/// TaiL GUI 应用
///
/// 重构版本：使用 AsyncBridge 进行非阻塞异步通信
pub struct TaiLApp {
    /// 当前视图
    current_view: View,

    /// 统计页面选中的时间范围
    stats_time_range: TimeRange,

    /// 时间导航状态
    navigation_state: TimeNavigationState,

    /// 统计页面是否使用堆叠视图
    stats_use_stacked_view: bool,

    /// 异步桥接器（替代阻塞的 tokio runtime）
    async_bridge: AsyncBridge,

    /// 仪表板数据缓存（固定为今天）
    dashboard_usage_cache: Vec<AppUsage>,

    /// 统计页面数据缓存
    stats_usage_cache: Vec<AppUsage>,

    /// 详细记录数据缓存（所有历史数据）
    details_usage_cache: Vec<AppUsage>,

    /// 每日目标缓存
    daily_goals_cache: Vec<DailyGoal>,

    /// 仪表板上次刷新时间
    dashboard_last_refresh: Option<DateTime<Utc>>,

    /// 统计页面上次刷新时间
    stats_last_refresh: Option<DateTime<Utc>>,

    /// 详细记录上次刷新时间
    details_last_refresh: Option<DateTime<Utc>>,

    /// 分类页面上次刷新时间
    categories_last_refresh: Option<DateTime<Utc>>,

    /// 每日目标上次刷新时间
    daily_goals_last_refresh: Option<DateTime<Utc>>,

    /// 分类数据缓存
    categories_data_cache: Option<CategoriesDataPayload>,

    /// 主题类型
    theme_type: ThemeType,

    /// 当前主题
    theme: TaiLTheme,

    /// 图标缓存
    icon_cache: IconCache,

    /// 添加目标对话框
    add_goal_dialog: AddGoalDialog,

    /// 别名对话框
    alias_dialog: AliasDialog,

    /// 分类视图（持久化状态）
    categories_view: CategoriesView,

    /// 详细视图（持久化状态）
    details_view: DetailsView,

    /// 是否已应用主题
    theme_applied: bool,

    /// 窗口是否可见（用于检测工作区切换）
    was_visible: bool,

    /// 导航模式
    navigation_mode: NavigationMode,

    /// 默认统计视图
    default_stats_view: DefaultStatsView,

    /// 待处理的应用名称（用于获取应用分类）
    pending_app_name: Option<String>,
}

impl TaiLApp {
    /// 创建新的应用实例
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // 注意：字体配置已在 main.rs 中通过 setup_fonts() 完成
        // 不要在这里重复配置字体，否则会覆盖已设置的字体

        let config = DbConfig::default();
        tracing::info!("初始化数据库，路径: {}", config.path);

        let repo = Arc::new(Repository::new(&config).expect("Failed to initialize database"));

        // 创建异步桥接器（内部启动后台线程运行 tokio runtime）
        let async_bridge = AsyncBridge::new(repo);

        tracing::info!("TaiL GUI 应用初始化成功（使用异步桥接器）");

        let theme_type = ThemeType::default();
        let theme = theme_type.to_theme();

        // 初始化导航状态为今天的小时视图（根据默认视图设置）
        let local_now = Local::now();
        let current_year = local_now.year();
        let default_stats_view = DefaultStatsView::default();

        let navigation_state = match default_stats_view {
            DefaultStatsView::Today => {
                let mut state = TimeNavigationState::new(current_year);
                state.go_to_today(local_now.year(), local_now.month(), local_now.day());
                state
            }
            DefaultStatsView::Yesterday => {
                let yesterday = local_now.date_naive() - chrono::Duration::days(1);
                let mut state = TimeNavigationState::new(yesterday.year());
                state.go_to_yesterday(yesterday.year(), yesterday.month(), yesterday.day());
                state
            }
            DefaultStatsView::ThisWeek => {
                let mut state = TimeNavigationState::new(current_year);
                state.switch_to_this_week(current_year, local_now.month());
                state
            }
            DefaultStatsView::ThisMonth => {
                let mut state = TimeNavigationState::new(current_year);
                state.switch_to_this_month(current_year, local_now.month());
                state
            }
        };

        Self {
            current_view: View::Dashboard,
            stats_time_range: TimeRange::Today,
            navigation_state,
            stats_use_stacked_view: false,
            async_bridge,
            dashboard_usage_cache: Vec::new(),
            stats_usage_cache: Vec::new(),
            details_usage_cache: Vec::new(),
            daily_goals_cache: Vec::new(),
            dashboard_last_refresh: None,
            stats_last_refresh: None,
            details_last_refresh: None,
            categories_last_refresh: None,
            daily_goals_last_refresh: None,
            categories_data_cache: None,
            theme_type,
            theme: theme.clone(),
            icon_cache: IconCache::new(),
            add_goal_dialog: AddGoalDialog::new(),
            alias_dialog: AliasDialog::default(),
            categories_view: CategoriesView::new(theme.clone()),
            details_view: DetailsView::new(),
            theme_applied: false,
            was_visible: true,
            navigation_mode: NavigationMode::default(),
            default_stats_view,
            pending_app_name: None,
        }
    }

    /// 处理来自异步桥接器的响应
    fn process_responses(&mut self) {
        while let Some(response) = self.async_bridge.try_recv_response() {
            match response {
                GuiResponse::DashboardData(Ok(usage)) => {
                    tracing::debug!("仪表板数据获取成功，{} 条记录", usage.len());
                    self.dashboard_usage_cache = usage;
                    self.dashboard_last_refresh = Some(Utc::now());
                }
                GuiResponse::DashboardData(Err(e)) => {
                    tracing::error!("获取仪表板数据失败: {}", e);
                    self.dashboard_last_refresh = Some(Utc::now()); // 防止连续重试
                }

                GuiResponse::StatsData(Ok(usage)) => {
                    debug!(count = usage.len(), "统计数据获取成功");
                    self.stats_usage_cache = usage;
                    self.stats_last_refresh = Some(Utc::now());
                }
                GuiResponse::StatsData(Err(e)) => {
                    debug!(error = %e, "获取统计数据失败");
                    self.stats_last_refresh = Some(Utc::now());
                }

                GuiResponse::DetailsData(Ok(usage)) => {
                    debug!(count = usage.len(), "详细记录数据获取成功");
                    self.details_usage_cache = usage;
                    self.details_last_refresh = Some(Utc::now());
                }
                GuiResponse::DetailsData(Err(e)) => {
                    debug!(error = %e, "获取详细记录数据失败");
                    self.details_last_refresh = Some(Utc::now());
                }

                GuiResponse::CategoriesData(Ok(payload)) => {
                    self.categories_data_cache = Some(payload.clone());
                    self.categories_view.load_data(
                        payload.category_usage,
                        payload.categories,
                        payload.all_apps,
                        payload.app_usage,
                    );
                    self.categories_last_refresh = Some(Utc::now());
                }
                GuiResponse::CategoriesData(Err(e)) => {
                    tracing::error!("获取分类数据失败: {}", e);
                    self.categories_last_refresh = Some(Utc::now());
                }

                GuiResponse::DailyGoals(Ok(goals)) => {
                    self.daily_goals_cache = goals;
                    self.daily_goals_last_refresh = Some(Utc::now());
                }
                GuiResponse::DailyGoals(Err(e)) => {
                    tracing::error!("获取每日目标失败: {}", e);
                    self.daily_goals_last_refresh = Some(Utc::now());
                }

                GuiResponse::Aliases(Ok(aliases)) => {
                    self.alias_dialog.open_for_management(aliases);
                }
                GuiResponse::Aliases(Err(e)) => {
                    tracing::error!("获取别名失败: {}", e);
                }

                GuiResponse::Categories(Ok(categories)) => {
                    // 分类操作后的更新
                    if let Some(ref mut payload) = self.categories_data_cache {
                        payload.categories = categories;
                    }
                }
                GuiResponse::Categories(Err(e)) => {
                    tracing::error!("获取分类列表失败: {}", e);
                }

                GuiResponse::AppNames(Ok(app_names)) => {
                    // 应用名称列表更新
                    if let Some(ref mut payload) = self.categories_data_cache {
                        payload.all_apps = app_names;
                    }
                }
                GuiResponse::AppNames(Err(e)) => {
                    tracing::error!("获取应用名称失败: {}", e);
                }

                GuiResponse::AppCategories(Ok(categories)) => {
                    let category_ids: Vec<i64> = categories.iter().filter_map(|c| c.id).collect();
                    self.categories_view.set_app_categories(category_ids);
                }
                GuiResponse::AppCategories(Err(e)) => {
                    tracing::error!("获取应用分类失败: {}", e);
                }

                GuiResponse::Done(Ok(())) => {
                    // 操作成功完成
                }
                GuiResponse::Done(Err(e)) => {
                    tracing::error!("操作失败: {}", e);
                }

                GuiResponse::ShutdownAck => {
                    tracing::info!("异步桥接器已关闭");
                }
            }
        }
    }

    /// 请求刷新仪表板数据（非阻塞）
    fn request_dashboard_refresh(&mut self) {
        let now = Utc::now();
        // 每5秒刷新一次（减少数据库查询频率）
        if let Some(last) = self.dashboard_last_refresh {
            let elapsed = now.signed_duration_since(last).num_seconds();
            if elapsed < 5 {
                return;
            }
        }

        // 仪表板固定显示今天的数据
        let local_now = Local::now();
        let today_start = local_now
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .with_timezone(&Utc);

        let _ = self.async_bridge.send_command(GuiCommand::RefreshDashboard {
            start: today_start,
            end: now,
        });

        // 同时刷新每日目标
        if self.daily_goals_last_refresh.is_none()
            || Utc::now().signed_duration_since(self.daily_goals_last_refresh.unwrap()).num_seconds() >= 5
        {
            let _ = self.async_bridge.send_command(GuiCommand::GetDailyGoals);
        }
    }

    /// 请求刷新统计页面数据（非阻塞）
    fn request_stats_refresh(&mut self) {
        let now = Utc::now();
        // 每5秒刷新一次
        if let Some(last) = self.stats_last_refresh {
            let elapsed = now.signed_duration_since(last).num_seconds();
            if elapsed < 5 {
                return;
            }
        }

        let (start, end) = self.get_stats_time_range_bounds();
        debug!(
            time_range = ?self.stats_time_range,
            start = %start,
            end = %end,
            "请求统计数据"
        );

        let _ = self.async_bridge.send_command(GuiCommand::RefreshStats { start, end });
    }

    /// 请求刷新详细记录数据（非阻塞）
    fn request_details_refresh(&mut self) {
        let now = Utc::now();
        // 每10秒刷新一次（详细记录数据量大，减少刷新频率）
        if let Some(last) = self.details_last_refresh {
            let elapsed = now.signed_duration_since(last).num_seconds();
            if elapsed < 10 {
                return;
            }
        }

        let start = DateTime::from_timestamp(0, 0).unwrap();

        debug!(
            start = %start,
            end = %now,
            "请求详细记录数据"
        );

        let _ = self.async_bridge.send_command(GuiCommand::RefreshDetails { start, end: now });
    }

    /// 请求加载分类页面数据（非阻塞）
    fn request_categories_data(&mut self) {
        let now = Utc::now();
        let should_refresh = self.categories_view.needs_refresh()
            || self
                .categories_last_refresh
                .map(|last| now.signed_duration_since(last).num_seconds() >= 5)
                .unwrap_or(true);

        if should_refresh {
            let (start, end) = self.get_stats_time_range_bounds();
            let _ = self.async_bridge.send_command(GuiCommand::LoadCategoriesData { start, end });
            self.categories_view.clear_refresh_flag();
        }
    }

    /// 获取统计页面时间范围的开始和结束时间
    fn get_stats_time_range_bounds(&self) -> (DateTime<Utc>, DateTime<Utc>) {
        let now = Utc::now();
        let local_now = Local::now();

        match self.stats_time_range {
            TimeRange::Today => {
                let today_start = local_now
                    .date_naive()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_local_timezone(Local)
                    .unwrap()
                    .with_timezone(&Utc);
                (today_start, now)
            }
            TimeRange::Yesterday => {
                let local_yesterday = local_now - ChronoDuration::days(1);
                let yesterday_start = local_yesterday
                    .date_naive()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_local_timezone(Local)
                    .unwrap()
                    .with_timezone(&Utc);
                let yesterday_end = local_yesterday
                    .date_naive()
                    .and_hms_opt(23, 59, 59)
                    .unwrap()
                    .and_local_timezone(Local)
                    .unwrap()
                    .with_timezone(&Utc);
                (yesterday_start, yesterday_end)
            }
            TimeRange::Last7Days => {
                let local_week_ago = local_now - ChronoDuration::days(7);
                let week_ago_start = local_week_ago
                    .date_naive()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_local_timezone(Local)
                    .unwrap()
                    .with_timezone(&Utc);
                (week_ago_start, now)
            }
            TimeRange::Last30Days => {
                let local_month_ago = local_now - ChronoDuration::days(30);
                let month_ago_start = local_month_ago
                    .date_naive()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_local_timezone(Local)
                    .unwrap()
                    .with_timezone(&Utc);
                (month_ago_start, now)
            }
            TimeRange::Custom(start, end) => (start, end),
        }
    }

    /// 切换主题
    fn change_theme(&mut self, theme_type: ThemeType) {
        let theme_name = match theme_type {
            ThemeType::Light => "浅色",
            ThemeType::Dark => "深色",
            ThemeType::Auto => "自动",
            _ => "自定义主题",
        };
        info!(theme = theme_name, "主题切换");

        self.theme_type = theme_type;
        self.theme = theme_type.to_theme();
        self.theme_applied = false;
    }

    /// 应用默认统计视图
    fn apply_default_stats_view(&mut self) {
        let local_now = Local::now();
        match self.default_stats_view {
            DefaultStatsView::Today => {
                self.navigation_state.go_to_today(
                    local_now.year(),
                    local_now.month(),
                    local_now.day(),
                );
            }
            DefaultStatsView::Yesterday => {
                let yesterday = local_now.date_naive() - chrono::Duration::days(1);
                self.navigation_state.go_to_yesterday(
                    yesterday.year(),
                    yesterday.month(),
                    yesterday.day(),
                );
            }
            DefaultStatsView::ThisWeek => {
                self.navigation_state
                    .switch_to_this_week(local_now.year(), local_now.month());
            }
            DefaultStatsView::ThisMonth => {
                self.navigation_state
                    .switch_to_this_month(local_now.year(), local_now.month());
            }
        }
        // 清除缓存，触发数据重新加载
        self.stats_usage_cache.clear();
        self.stats_last_refresh = None;
    }

    /// 添加每日目标
    fn add_daily_goal(&mut self, goal: DailyGoal) {
        let _ = self.async_bridge.send_command(GuiCommand::AddDailyGoal(goal));
    }

    /// 删除每日目标
    fn delete_daily_goal(&mut self, app_name: &str) {
        let app_name = app_name.to_string();
        let _ = self.async_bridge.send_command(GuiCommand::DeleteDailyGoal(app_name));
    }

    /// 设置应用别名
    fn set_app_alias(&mut self, app_name: String, alias: String) {
        if alias.is_empty() {
            let _ = self.async_bridge.send_command(GuiCommand::DeleteAppAlias(app_name));
        } else {
            let _ = self.async_bridge.send_command(GuiCommand::SetAppAlias { app_name, alias });
        }
    }

    /// 打开别名管理对话框
    fn open_alias_management(&mut self) {
        let _ = self.async_bridge.send_command(GuiCommand::GetAllAliases);
    }

    /// 处理分类视图操作
    fn handle_category_action(&mut self, action: CategoryAction) {
        match action {
            CategoryAction::AddCategory(category) => {
                let _ = self.async_bridge.send_command(GuiCommand::AddCategory(category));
            }
            CategoryAction::UpdateCategory(category) => {
                let _ = self.async_bridge.send_command(GuiCommand::UpdateCategory(category));
            }
            CategoryAction::DeleteCategory(id) => {
                let _ = self.async_bridge.send_command(GuiCommand::DeleteCategory(id));
            }
            CategoryAction::SetAppCategories(app_name, category_ids) => {
                let _ = self.async_bridge.send_command(GuiCommand::SetAppCategories {
                    app_name,
                    category_ids,
                });
            }
            CategoryAction::RemoveAppFromCategory(app_name, category_id) => {
                let _ = self.async_bridge.send_command(GuiCommand::RemoveAppFromCategory {
                    app_name,
                    category_id,
                });
            }
            CategoryAction::LoadAppCategories(app_name) => {
                self.pending_app_name = Some(app_name.clone());
                let _ = self.async_bridge.send_command(GuiCommand::GetAppCategories(app_name));
            }
        }
    }
}

impl eframe::App for TaiLApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 首先处理来自异步桥接器的响应
        self.process_responses();

        // 应用主题
        if !self.theme_applied {
            self.theme.apply(ctx);
            self.theme_applied = true;
        }

        // 检测窗口焦点状态变化
        let has_focus = ctx.input(|i| i.focused);
        let just_got_focus = has_focus && !self.was_visible;
        self.was_visible = has_focus;

        // 如果窗口刚获得焦点，强制刷新数据
        if just_got_focus {
            self.dashboard_last_refresh = None;
            self.stats_last_refresh = None;
            tracing::debug!("窗口获得焦点，强制刷新数据");
        }

        // 只在窗口有焦点时请求重绘
        if has_focus {
            ctx.request_repaint_after(std::time::Duration::from_secs(5));
        }

        // 根据当前视图请求数据刷新（非阻塞）
        match self.current_view {
            View::Dashboard => self.request_dashboard_refresh(),
            View::Statistics => self.request_stats_refresh(),
            View::Categories => self.request_categories_data(),
            View::Details => self.request_details_refresh(),
            View::Settings => {
                // 设置页面也需要仪表板数据（用于显示目标）
                if self.daily_goals_last_refresh.is_none() {
                    let _ = self.async_bridge.send_command(GuiCommand::GetAllCategories);
                }
            }
        }

        // 处理添加目标对话框
        if let Some(goal) = self.add_goal_dialog.show(ctx, &self.theme) {
            self.add_daily_goal(goal);
        }

        // 处理别名对话框
        if let Some((app_name, alias)) = self.alias_dialog.show(ctx, &self.theme) {
            self.set_app_alias(app_name, alias);
        }

        // 根据导航模式显示导航栏
        let new_view = match self.navigation_mode {
            NavigationMode::Sidebar => {
                let mut nav =
                    SidebarNav::new(self.current_view, &self.theme, &mut self.navigation_mode);
                nav.show(ctx)
            }
            NavigationMode::TopTab => {
                let mut nav =
                    TopTabNav::new(self.current_view, &self.theme, &mut self.navigation_mode);
                nav.show(ctx)
            }
        };

        if let Some(view) = new_view {
            self.current_view = view;
        }

        // 主内容区域
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(self.theme.background_color)
                    .inner_margin(egui::Margin::same(self.theme.spacing)),
            )
            .show(ctx, |ui| {
                match self.current_view {
                    View::Dashboard => {
                        let mut view = DashboardView::new(
                            &self.dashboard_usage_cache,
                            &self.theme,
                            &mut self.icon_cache,
                        );
                        view.show(ui);
                    }
                    View::Statistics => {
                        let mut view = StatisticsView::new(
                            &self.stats_usage_cache,
                            &mut self.navigation_state,
                            &self.theme,
                            &mut self.icon_cache,
                            self.stats_use_stacked_view,
                        );
                        let (new_range, use_stacked) = view.show(ui);
                        if let Some(range) = new_range {
                            self.stats_time_range = range;
                            self.stats_last_refresh = None; // 强制刷新
                        }
                        self.stats_use_stacked_view = use_stacked;
                    }
                    View::Categories => {
                        // 使用持久化的分类视图
                        if let Some(action) = self.categories_view.show(ui) {
                            self.handle_category_action(action);
                        }
                    }
                    View::Details => {
                        // 更新数据并显示持久化的详细视图
                        self.details_view.update_data(&self.details_usage_cache);
                        self.details_view
                            .show(ui, &self.theme, &mut self.icon_cache);
                    }
                    View::Settings => {
                        let view = SettingsView::new(
                            &self.daily_goals_cache,
                            self.theme_type,
                            self.default_stats_view,
                            &self.theme,
                        );
                        match view.show(ui) {
                            SettingsAction::AddGoal => {
                                self.add_goal_dialog.open();
                            }
                            SettingsAction::DeleteGoal(app_name) => {
                                self.delete_daily_goal(&app_name);
                            }
                            SettingsAction::ChangeTheme(theme_type) => {
                                self.change_theme(theme_type);
                            }
                            SettingsAction::ChangeDefaultView(default_view) => {
                                self.default_stats_view = default_view;
                                self.apply_default_stats_view();
                            }
                            SettingsAction::ManageAliases => {
                                self.open_alias_management();
                            }
                            SettingsAction::None => {}
                        }
                    }
                }
            });
    }
}
