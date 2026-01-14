//! TaiL GUI - egui 应用

use chrono::{DateTime, Utc, Duration as ChronoDuration};
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

    /// 选中的时间范围
    time_range: TimeRange,

    /// 数据库仓库
    repo: Arc<Repository>,

    /// 应用使用数据缓存
    app_usage_cache: Vec<AppUsage>,

    /// 每日目标缓存
    daily_goals_cache: Vec<DailyGoal>,

    /// 上次刷新时间
    last_refresh: Option<DateTime<Utc>>,

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
            time_range: TimeRange::Today,
            repo: Arc::new(repo),
            app_usage_cache: Vec::new(),
            daily_goals_cache: Vec::new(),
            last_refresh: None,
            theme_type,
            theme,
            icon_cache: IconCache::new(),
            add_goal_dialog: AddGoalDialog::new(),
            theme_applied: false,
        }
    }

    /// 刷新数据
    fn refresh_data(&mut self) {
        let now = Utc::now();
        // 每2秒刷新一次
        if let Some(last) = self.last_refresh {
            let elapsed = now.signed_duration_since(last).num_seconds();
            if elapsed < 2 {
                return;
            }
        }

        let (start, end) = self.get_time_range_bounds();
        
        // 刷新应用使用数据
        match self.repo.get_app_usage(start, end) {
            Ok(usage) => {
                tracing::debug!("获取 {} 条应用使用记录", usage.len());
                self.app_usage_cache = usage;
            }
            Err(e) => {
                tracing::error!("获取应用使用数据失败: {}", e);
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

        self.last_refresh = Some(now);
    }

    /// 获取时间范围的开始和结束时间
    fn get_time_range_bounds(&self) -> (DateTime<Utc>, DateTime<Utc>) {
        let now = Utc::now();
        match self.time_range {
            TimeRange::Today => {
                let today_start = now.date_naive()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc();
                (today_start, now)
            }
            TimeRange::Yesterday => {
                let yesterday = now - ChronoDuration::days(1);
                let yesterday_start = yesterday.date_naive()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc();
                let yesterday_end = yesterday.date_naive()
                    .and_hms_opt(23, 59, 59)
                    .unwrap()
                    .and_utc();
                (yesterday_start, yesterday_end)
            }
            TimeRange::Last7Days => {
                let week_ago = now - ChronoDuration::days(7);
                (week_ago, now)
            }
            TimeRange::Last30Days => {
                let month_ago = now - ChronoDuration::days(30);
                (month_ago, now)
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
        if let Ok(_) = self.repo.upsert_daily_goal(&goal) {
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

        // 请求持续重绘
        ctx.request_repaint();

        // 刷新数据
        self.refresh_data();

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
                    // Logo
                    ui.label(egui::RichText::new("🦎")
                        .size(24.0));
                    ui.label(egui::RichText::new("TaiL")
                        .size(self.theme.heading_size)
                        .color(self.theme.text_color)
                        .strong());
                    
                    ui.add_space(24.0);

                    // 导航按钮
                    let nav_items = [
                        (View::Dashboard, "仪表板", "📊"),
                        (View::Statistics, "统计", "📈"),
                        (View::Settings, "设置", "⚙️"),
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
                        let view = DashboardView::new(
                            &self.app_usage_cache,
                            &self.theme,
                            &self.icon_cache,
                        );
                        view.show(ui);
                    }
                    View::Statistics => {
                        let view = StatisticsView::new(
                            &self.app_usage_cache,
                            self.time_range,
                            &self.theme,
                            &self.icon_cache,
                        );
                        if let Some(new_range) = view.show(ui) {
                            self.time_range = new_range;
                            self.last_refresh = None; // 强制刷新
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
