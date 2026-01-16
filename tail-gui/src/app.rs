//! TaiL GUI - egui 应用

use chrono::{DateTime, Datelike, Duration as ChronoDuration, Local, Utc};
use std::sync::Arc;
use tail_core::models::{TimeNavigationState, TimeRange};
use tail_core::{AppUsage, DailyGoal, Repository};
use tail_core::db::Config as DbConfig;
use tail_core::traits::{AppUsageQuery, DailyGoalRepository, AliasRepository, CategoryRepository, CategoryUsageQuery};
use tracing::{debug, info, warn};

use crate::components::{AliasDialog, NavigationMode, SidebarNav, TopTabNav, View};
use crate::icons::IconCache;
use crate::theme::{TaiLTheme, ThemeType};
use crate::views::{
    AddGoalDialog, CategoriesView, CategoryAction, DashboardView, DetailsView,
    SettingsAction, SettingsView, StatisticsView,
};

/// TaiL GUI 应用
pub struct TaiLApp {
    /// 当前视图
    current_view: View,

    /// 统计页面选中的时间范围
    stats_time_range: TimeRange,

    /// 时间导航状态
    navigation_state: TimeNavigationState,

    /// 统计页面是否使用堆叠视图
    stats_use_stacked_view: bool,

    /// 数据库仓库
    repo: Arc<Repository>,

    /// Tokio runtime（用于处理异步数据库调用）
    runtime: tokio::runtime::Runtime,

    /// 仪表板数据缓存（固定为今天）
    dashboard_usage_cache: Vec<AppUsage>,

    /// 统计页面数据缓存
    stats_usage_cache: Vec<AppUsage>,

    /// 每日目标缓存
    daily_goals_cache: Vec<DailyGoal>,

    /// 仪表板上次刷新时间
    dashboard_last_refresh: Option<DateTime<Utc>>,

    /// 统计页面上次刷新时间
    stats_last_refresh: Option<DateTime<Utc>>,

    /// 分类页面上次刷新时间
    categories_last_refresh: Option<DateTime<Utc>>,

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
}

impl TaiLApp {
    /// 创建新的应用实例
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // 注意：字体配置已在 main.rs 中通过 setup_fonts() 完成
        // 不要在这里重复配置字体，否则会覆盖已设置的字体

        let config = DbConfig::default();
        tracing::info!("初始化数据库，路径: {}", config.path);

        let repo = Repository::new(&config).expect("Failed to initialize database");

        // 创建 tokio runtime 用于异步数据库调用
        let runtime = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

        tracing::info!("TaiL GUI 应用初始化成功");

        let theme_type = ThemeType::default();
        let theme = theme_type.to_theme();

        // 初始化导航状态为当前年份的月视图
        let current_year = Local::now().year();
        let navigation_state = TimeNavigationState::new(current_year);

        Self {
            current_view: View::Dashboard,
            stats_time_range: TimeRange::Today,
            navigation_state,
            stats_use_stacked_view: false,
            repo: Arc::new(repo),
            runtime,
            dashboard_usage_cache: Vec::new(),
            stats_usage_cache: Vec::new(),
            daily_goals_cache: Vec::new(),
            dashboard_last_refresh: None,
            stats_last_refresh: None,
            categories_last_refresh: None,
            theme_type,
            theme: theme.clone(),
            icon_cache: IconCache::new(),
            add_goal_dialog: AddGoalDialog::new(),
            alias_dialog: AliasDialog::default(),
            categories_view: CategoriesView::new(theme.clone()),
            details_view: DetailsView::new(),
            theme_applied: false,
            was_visible: true,
            navigation_mode: NavigationMode::default(), // 默认为侧边栏模式
        }
    }

    /// 刷新仪表板数据（固定为今天）
    fn refresh_dashboard_data(&mut self) {
        let now = Utc::now();
        // 每5秒刷新一次（减少数据库查询频率）
        if let Some(last) = self.dashboard_last_refresh {
            let elapsed = now.signed_duration_since(last).num_seconds();
            if elapsed < 5 {
                return;
            }
        }

        // 仪表板固定显示今天的数据（使用本地时间计算"今天"的开始）
        let local_now = Local::now();
        let today_start = local_now
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .with_timezone(&Utc);

        // 使用 tokio runtime 处理异步调用
        match self.runtime.block_on(async {
            AppUsageQuery::get_app_usage(&self.repo.usage_service(), today_start, now).await
        }) {
            Ok(usage) => {
                tracing::debug!("仪表板获取 {} 条应用使用记录", usage.len());
                self.dashboard_usage_cache = usage;
            }
            Err(e) => {
                tracing::error!("获取仪表板数据失败: {}", e);
            }
        }

        // 刷新每日目标
        match self.runtime.block_on(async {
            DailyGoalRepository::get_all(&self.repo.goal_service()).await
        }) {
            Ok(goals) => {
                self.daily_goals_cache = goals;
            }
            Err(e) => {
                tracing::error!("获取每日目标失败: {}", e);
            }
        }

        self.dashboard_last_refresh = Some(now);
    }

    /// 刷新统计页面数据
    fn refresh_stats_data(&mut self) {
        let now = Utc::now();
        // 每5秒刷新一次（减少数据库查询频率）
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
            "刷新统计数据"
        );

        // 使用 tokio runtime 处理异步调用
        match self.runtime.block_on(async {
            AppUsageQuery::get_app_usage(&self.repo.usage_service(), start, end).await
        }) {
            Ok(usage) => {
                debug!(
                    count = usage.len(),
                    "统计数据获取成功"
                );
                for (i, u) in usage.iter().take(3).enumerate() {
                    debug!(
                        index = i,
                        app_name = %u.app_name,
                        total_seconds = u.total_seconds,
                        events_count = u.window_events.len(),
                        "应用使用记录"
                    );
                }
                self.stats_usage_cache = usage;
            }
            Err(e) => {
                debug!(error = %e, "获取统计数据失败");
            }
        }

        self.stats_last_refresh = Some(now);
    }

    /// 获取统计页面时间范围的开始和结束时间
    fn get_stats_time_range_bounds(&self) -> (DateTime<Utc>, DateTime<Utc>) {
        let now = Utc::now();
        let local_now = Local::now();

        match self.stats_time_range {
            TimeRange::Today => {
                // 使用本地时间计算"今天"的开始
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
                // 使用本地时间计算"昨天"
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
                // 使用本地时间计算7天前的开始
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
                // 使用本地时间计算30天前的开始
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
            _ => {
                // 使用 Debug 输出其他主题名称
                "自定义主题"
            }
        };
        info!(theme = theme_name, "主题切换");

        self.theme_type = theme_type;
        self.theme = theme_type.to_theme();
        self.theme_applied = false;
    }

    /// 添加每日目标
    fn add_daily_goal(&mut self, goal: DailyGoal) {
        let _ = self.runtime.block_on(async {
            DailyGoalRepository::upsert(&self.repo.goal_service(), &goal).await
        });
        self.daily_goals_cache.push(goal);
    }

    /// 删除每日目标
    fn delete_daily_goal(&mut self, app_name: &str) {
        let app_name = app_name.to_string();
        if self.runtime.block_on(async {
            DailyGoalRepository::delete(&self.repo.goal_service(), &app_name).await
        }).is_ok() {
            self.daily_goals_cache.retain(|g| g.app_name != app_name);
        }
    }

    /// 设置应用别名
    fn set_app_alias(&mut self, app_name: String, alias: String) {
        if alias.is_empty() {
            // 删除别名
            let _ = self.runtime.block_on(async {
                AliasRepository::delete(&self.repo.aliases(), &app_name).await
            });
        } else {
            let _ = self.runtime.block_on(async {
                AliasRepository::set(&self.repo.aliases(), &app_name, &alias).await
            });
        }
    }

    /// 打开别名管理对话框
    fn open_alias_management(&mut self) {
        if let Ok(aliases) = self.runtime.block_on(async {
            AliasRepository::get_all(&self.repo.aliases()).await
        }) {
            self.alias_dialog.open_for_management(aliases);
        }
    }

    /// 加载分类页面数据
    fn load_categories_data(&mut self, start: DateTime<Utc>, end: DateTime<Utc>) {
        // 加载分类使用统计
        let category_usage: Vec<tail_core::CategoryUsage> = self
            .runtime
            .block_on(async {
                CategoryUsageQuery::get_category_usage(&self.repo.usage_service(), start, end).await
            })
            .unwrap_or_default();

        // 加载所有分类
        let categories = self
            .runtime
            .block_on(async { CategoryRepository::get_all(&self.repo.category_service()).await })
            .unwrap_or_default();

        // 加载所有应用名称
        let all_apps = self
            .runtime
            .block_on(async { CategoryRepository::get_all_app_names(&self.repo.category_service()).await })
            .unwrap_or_default();

        // 加载应用使用数据（用于堆叠柱形图）
        let app_usage = self
            .runtime
            .block_on(async { AppUsageQuery::get_app_usage(&self.repo.usage_service(), start, end).await })
            .unwrap_or_default();

        // 将数据加载到视图
        self.categories_view.load_data(category_usage, categories, all_apps, app_usage);
    }

    /// 处理分类视图操作
    fn handle_category_action(&mut self, action: CategoryAction) {
        match action {
            CategoryAction::AddCategory(category) => {
                let _ = self.runtime.block_on(async {
                    CategoryRepository::insert(&self.repo.category_service(), &category).await
                });
            }
            CategoryAction::UpdateCategory(category) => {
                let _ = self.runtime.block_on(async {
                    CategoryRepository::update(&self.repo.category_service(), &category).await
                });
            }
            CategoryAction::DeleteCategory(id) => {
                let _ = self.runtime.block_on(async {
                    CategoryRepository::delete(&self.repo.category_service(), id).await
                });
            }
            CategoryAction::SetAppCategories(app_name, category_ids) => {
                let _ = self.runtime.block_on(async {
                    CategoryRepository::set_app_categories(&self.repo.category_service(), &app_name, &category_ids).await
                });
            }
            CategoryAction::RemoveAppFromCategory(app_name, category_id) => {
                let _ = self.runtime.block_on(async {
                    CategoryRepository::remove_app_from_category(&self.repo.category_service(), &app_name, category_id).await
                });
            }
            CategoryAction::LoadAppCategories(app_name) => {
                if let Ok(categories) = self.runtime.block_on(async {
                    CategoryRepository::get_app_categories(&self.repo.category_service(), &app_name).await
                }) {
                    let category_ids: Vec<i64> = categories.iter().filter_map(|c| c.id).collect();
                    self.categories_view.set_app_categories(category_ids);
                }
            }
        }
    }
}

impl eframe::App for TaiLApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
        // 这样可以避免在窗口不可见时阻塞事件循环
        if has_focus {
            ctx.request_repaint_after(std::time::Duration::from_secs(5));
        }
        // 注意：当窗口没有焦点时，不请求重绘
        // 当用户切换回来时，系统会自动触发重绘

        // 根据当前视图刷新对应数据
        match self.current_view {
            View::Dashboard => self.refresh_dashboard_data(),
            View::Statistics => self.refresh_stats_data(),
            View::Categories => self.refresh_dashboard_data(), // 分类页面也刷新仪表板数据
            View::Details => self.refresh_dashboard_data(),    // 详细页面也刷新仪表板数据
            View::Settings => self.refresh_dashboard_data(),   // 设置页面也刷新仪表板数据
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
                        // 检查是否需要刷新数据
                        let now = Utc::now();
                        let should_refresh = self.categories_view.needs_refresh()
                            || self
                                .categories_last_refresh
                                .map(|last| now.signed_duration_since(last).num_seconds() >= 5)
                                .unwrap_or(true);

                        if should_refresh {
                            let (start, end) = self.get_stats_time_range_bounds();
                            self.load_categories_data(start, end);
                            self.categories_last_refresh = Some(now);
                            self.categories_view.clear_refresh_flag();
                        }

                        // 使用持久化的分类视图，处理返回的操作
                        if let Some(action) = self.categories_view.show(ui) {
                            self.handle_category_action(action);
                        }
                    }
                    View::Details => {
                        // 更新数据并显示持久化的详细视图
                        self.details_view.update_data(&self.dashboard_usage_cache);
                        self.details_view.show(ui, &self.theme, &mut self.icon_cache);
                    }
                    View::Settings => {
                        let view = SettingsView::new(
                            &self.daily_goals_cache,
                            self.theme_type,
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
