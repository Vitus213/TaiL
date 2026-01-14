//! TaiL GUI - egui 应用

use chrono::{DateTime, Utc, Local, Duration as ChronoDuration};
use egui::{Color32, Rounding, Vec2};
use tail_core::{DbConfig, Repository, AppUsage, DailyGoal};
use tail_core::models::TimeRange;
use std::sync::Arc;

use crate::icons::IconCache;
use crate::theme::{TaiLTheme, ThemeType};
use crate::views::{DashboardView, StatisticsView, SettingsView, SettingsAction, AddGoalDialog};

/// TaiL GUI 应用
pub struct TaiLApp {
    /// 当前视图
    current_view: View,

    /// 统计页面选中的时间范围
    stats_time_range: TimeRange,

    /// 数据库仓库
    repo: Arc<Repository>,

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

    /// 主题类型
    theme_type: ThemeType,

    /// 当前主题
    theme: TaiLTheme,

    /// 图标缓存
    icon_cache: IconCache,

    /// 添加目标对话框
    add_goal_dialog: AddGoalDialog,

    /// 是否已应用主题
    theme_applied: bool,

    /// 窗口是否可见（用于检测工作区切换）
    was_visible: bool,
}

/// 视图类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum View {
    Dashboard,
    Statistics,
    Settings,
}

impl TaiLApp {
    /// 创建新的应用实例
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // 注意：字体配置已在 main.rs 中通过 setup_fonts() 完成
        // 不要在这里重复配置字体，否则会覆盖已设置的字体

        let config = DbConfig::default();
        tracing::info!("初始化数据库，路径: {}", config.path);
        
        let repo = Repository::new(&config)
            .expect("Failed to initialize database");
        
        tracing::info!("TaiL GUI 应用初始化成功");

        let theme_type = ThemeType::default();
        let theme = theme_type.to_theme();

        Self {
            current_view: View::Dashboard,
            stats_time_range: TimeRange::Today,
            repo: Arc::new(repo),
            dashboard_usage_cache: Vec::new(),
            stats_usage_cache: Vec::new(),
            daily_goals_cache: Vec::new(),
            dashboard_last_refresh: None,
            stats_last_refresh: None,
            theme_type,
            theme,
            icon_cache: IconCache::new(),
            add_goal_dialog: AddGoalDialog::new(),
            theme_applied: false,
            was_visible: true,
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
        let today_start = local_now.date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .with_timezone(&Utc);
        
        match self.repo.get_app_usage(today_start, now) {
            Ok(usage) => {
                tracing::debug!("仪表板获取 {} 条应用使用记录", usage.len());
                self.dashboard_usage_cache = usage;
            }
            Err(e) => {
                tracing::error!("获取仪表板数据失败: {}", e);
            }
        }

        // 刷新每日目标
        match self.repo.get_daily_goals() {
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
        
        match self.repo.get_app_usage(start, end) {
            Ok(usage) => {
                tracing::debug!("统计页面获取 {} 条应用使用记录", usage.len());
                self.stats_usage_cache = usage;
            }
            Err(e) => {
                tracing::error!("获取统计数据失败: {}", e);
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
                let today_start = local_now.date_naive()
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
                let yesterday_start = local_yesterday.date_naive()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_local_timezone(Local)
                    .unwrap()
                    .with_timezone(&Utc);
                let yesterday_end = local_yesterday.date_naive()
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
                let week_ago_start = local_week_ago.date_naive()
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
                let month_ago_start = local_month_ago.date_naive()
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
        self.theme_type = theme_type;
        self.theme = theme_type.to_theme();
        self.theme_applied = false;
    }

    /// 添加每日目标
    fn add_daily_goal(&mut self, goal: DailyGoal) {
        if self.repo.upsert_daily_goal(&goal).is_ok() {
            self.daily_goals_cache.push(goal);
        }
    }

    /// 删除每日目标
    fn delete_daily_goal(&mut self, app_name: &str) {
        if let Ok(()) = self.repo.delete_daily_goal(app_name) {
            self.daily_goals_cache.retain(|g| g.app_name != app_name);
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
            View::Settings => self.refresh_dashboard_data(), // 设置页面也刷新仪表板数据
        }

        // 处理添加目标对话框
        if let Some(goal) = self.add_goal_dialog.show(ctx, &self.theme) {
            self.add_daily_goal(goal);
        }

        // 顶部导航栏
        egui::TopBottomPanel::top("nav_bar")
            .frame(egui::Frame::none()
                .fill(self.theme.card_background)
                .inner_margin(egui::Margin::symmetric(16.0, 8.0)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Logo - 直接使用文字，不使用 emoji
                    ui.label(egui::RichText::new("TaiL")
                        .size(self.theme.heading_size)
                        .color(self.theme.text_color)
                        .strong());
                    
                    ui.add_space(24.0);

                    // 导航按钮
                    let nav_items = [
                        (View::Dashboard, "仪表板", "📊"),
                        (View::Statistics, "统计", "📈"),
                        (View::Settings, "设置", "⚙"),
                    ];

                    for (view, label, icon) in nav_items {
                        let is_selected = self.current_view == view;
                        
                        let button = egui::Button::new(
                            egui::RichText::new(format!("{} {}", icon, label))
                                .size(self.theme.body_size)
                                .color(if is_selected {
                                    Color32::WHITE
                                } else {
                                    self.theme.text_color
                                })
                        )
                        .fill(if is_selected {
                            self.theme.primary_color
                        } else {
                            Color32::TRANSPARENT
                        })
                        .rounding(Rounding::same(8.0))
                        .min_size(Vec2::new(100.0, 32.0));

                        if ui.add(button).clicked() {
                            self.current_view = view;
                        }
                    }

                    // 右侧按钮
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // 退出按钮
                        if ui.add(
                            egui::Button::new(
                                egui::RichText::new("✕")
                                    .size(16.0)
                                    .color(self.theme.secondary_text_color)
                            )
                            .fill(Color32::TRANSPARENT)
                            .rounding(Rounding::same(4.0))
                        ).on_hover_text("退出").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }

                        // 最小化按钮
                        if ui.add(
                            egui::Button::new(
                                egui::RichText::new("─")
                                    .size(16.0)
                                    .color(self.theme.secondary_text_color)
                            )
                            .fill(Color32::TRANSPARENT)
                            .rounding(Rounding::same(4.0))
                        ).on_hover_text("最小化").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                        }
                    });
                });
            });

        // 主内容区域
        egui::CentralPanel::default()
            .frame(egui::Frame::none()
                .fill(self.theme.background_color)
                .inner_margin(egui::Margin::same(self.theme.spacing)))
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
                            self.stats_time_range,
                            &self.theme,
                            &mut self.icon_cache,
                        );
                        if let Some(new_range) = view.show(ui) {
                            self.stats_time_range = new_range;
                            self.stats_last_refresh = None; // 强制刷新
                        }
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
                            SettingsAction::None => {}
                        }
                    }
                }
            });
    }
}
