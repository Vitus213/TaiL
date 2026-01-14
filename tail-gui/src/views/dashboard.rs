//! TaiL GUI - 仪表板视图

use egui::{ScrollArea, Ui};
use tail_core::AppUsage;

use crate::components::{AppCard, EmptyState, PageHeader, StatCard, EnhancedProgressBar, SectionDivider};
use crate::icons::IconCache;
use crate::theme::TaiLTheme;

/// 仪表板视图
pub struct DashboardView<'a> {
    /// 应用使用数据
    app_usage: &'a [AppUsage],
    /// 主题
    theme: &'a TaiLTheme,
    /// 图标缓存
    icon_cache: &'a IconCache,
}

impl<'a> DashboardView<'a> {
    pub fn new(
        app_usage: &'a [AppUsage],
        theme: &'a TaiLTheme,
        icon_cache: &'a IconCache,
    ) -> Self {
        Self {
            app_usage,
            theme,
            icon_cache,
        }
    }

    /// 渲染仪表板
    pub fn show(&self, ui: &mut Ui) {
        // 页面标题
        ui.add(PageHeader::new("今日统计", "📊", self.theme)
            .subtitle(&Self::get_date_string()));
        
        ui.add_space(self.theme.spacing);

        // 统计卡片区域
        self.show_stat_cards(ui);
        
        ui.add_space(self.theme.spacing);

        // 分隔线
        ui.add(SectionDivider::new(self.theme).with_title("应用使用排行"));
        
        ui.add_space(self.theme.spacing / 2.0);

        // 应用列表
        self.show_app_list(ui);
    }

    /// 显示统计卡片
    fn show_stat_cards(&self, ui: &mut Ui) {
        let total_seconds: i64 = self.app_usage.iter()
            .map(|u| u.total_seconds)
            .sum();
        
        let app_count = self.app_usage.len();
        let avg_per_app = if app_count > 0 {
            total_seconds / app_count as i64
        } else {
            0
        };

        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = self.theme.spacing;
            
            // 总使用时间卡片
            ui.add(StatCard::new(
                "总使用时间",
                &Self::format_duration(total_seconds),
                "⏱️",
                self.theme,
            ).accent_color(self.theme.primary_color));

            // 应用数量卡片
            ui.add(StatCard::new(
                "活跃应用",
                &format!("{} 个", app_count),
                "📱",
                self.theme,
            ).accent_color(self.theme.accent_color));

            // 平均使用时间卡片
            ui.add(StatCard::new(
                "平均每应用",
                &Self::format_duration(avg_per_app),
                "📈",
                self.theme,
            ).accent_color(self.theme.warning_color));

            // 最常用应用卡片
            if let Some(top_app) = self.app_usage.first() {
                let icon = self.icon_cache.get_emoji(&top_app.app_name);
                ui.add(StatCard::new(
                    "最常用",
                    &top_app.app_name,
                    icon,
                    self.theme,
                ).subtitle(&Self::format_duration(top_app.total_seconds))
                 .accent_color(self.theme.success_color));
            }
        });
    }

    /// 显示应用列表
    fn show_app_list(&self, ui: &mut Ui) {
        if self.app_usage.is_empty() {
            ui.add(EmptyState::new(
                "📭",
                "暂无数据",
                "开始使用应用后，这里会显示使用统计",
                self.theme,
            ));
            return;
        }

        let total_seconds: i64 = self.app_usage.iter()
            .map(|u| u.total_seconds)
            .sum();

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = self.theme.spacing / 2.0;
                
                for (idx, usage) in self.app_usage.iter().enumerate() {
                    let percentage = if total_seconds > 0 {
                        (usage.total_seconds as f32 / total_seconds as f32) * 100.0
                    } else {
                        0.0
                    };

                    // 获取最近的窗口标题
                    let window_title = usage.window_events.last()
                        .map(|e| e.window_title.as_str());

                    let mut card = AppCard::new(
                        &usage.app_name,
                        &usage.app_name, // TODO: 使用别名
                        usage.total_seconds,
                        percentage,
                        idx + 1,
                        self.theme,
                    );

                    if let Some(title) = window_title {
                        card = card.with_window_title(title);
                    }

                    let response = ui.add(card);
                    
                    // 点击展开详情
                    if response.clicked() {
                        // TODO: 展开显示窗口标题历史
                    }
                    
                    // 右键菜单
                    response.context_menu(|ui| {
                        if ui.button("📝 重命名").clicked() {
                            // TODO: 打开重命名对话框
                            ui.close_menu();
                        }
                        if ui.button("🎯 设置目标").clicked() {
                            // TODO: 打开目标设置对话框
                            ui.close_menu();
                        }
                        if ui.button("📊 查看详情").clicked() {
                            // TODO: 跳转到详情页
                            ui.close_menu();
                        }
                    });
                }
            });
    }

    /// 格式化时长
    fn format_duration(seconds: i64) -> String {
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        let secs = seconds % 60;

        if hours > 0 {
            format!("{}h {}m", hours, minutes)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, secs)
        } else {
            format!("{}s", secs)
        }
    }

    /// 获取日期字符串
    fn get_date_string() -> String {
        use chrono::Local;
        Local::now().format("%Y年%m月%d日 %A").to_string()
    }
}

/// 今日总览组件
pub struct TodaySummary<'a> {
    total_seconds: i64,
    goal_seconds: Option<i64>,
    theme: &'a TaiLTheme,
}

impl<'a> TodaySummary<'a> {
    pub fn new(total_seconds: i64, theme: &'a TaiLTheme) -> Self {
        Self {
            total_seconds,
            goal_seconds: None,
            theme,
        }
    }

    pub fn with_goal(mut self, goal_seconds: i64) -> Self {
        self.goal_seconds = Some(goal_seconds);
        self
    }

    pub fn show(&self, ui: &mut Ui) {
        let hours = self.total_seconds / 3600;
        let minutes = (self.total_seconds % 3600) / 60;

        ui.vertical(|ui| {
            // 主要时间显示
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("{}h {}m", hours, minutes))
                        .size(self.theme.heading_size * 1.5)
                        .color(self.theme.text_color)
                );
            });

            // 目标进度
            if let Some(goal) = self.goal_seconds {
                let fraction = (self.total_seconds as f32 / goal as f32).min(1.5);
                let remaining = goal - self.total_seconds;
                
                ui.add_space(8.0);
                
                ui.add(EnhancedProgressBar::new(fraction.min(1.0), self.theme)
                    .height(10.0)
                    .show_percentage(true)
                    .label("今日目标"));

                if remaining > 0 {
                    ui.label(
                        egui::RichText::new(format!(
                            "距离目标还剩 {}",
                            Self::format_duration(remaining)
                        ))
                        .size(self.theme.small_size)
                        .color(self.theme.secondary_text_color)
                    );
                } else {
                    ui.label(
                        egui::RichText::new("🎉 已达成今日目标！")
                            .size(self.theme.small_size)
                            .color(self.theme.success_color)
                    );
                }
            }
        });
    }

    fn format_duration(seconds: i64) -> String {
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;

        if hours > 0 {
            format!("{}小时{}分钟", hours, minutes)
        } else {
            format!("{}分钟", minutes)
        }
    }
}