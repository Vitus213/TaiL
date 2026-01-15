//! TaiL GUI - 仪表板视图

use chrono::Local;
use chrono::Timelike;
use egui::{ScrollArea, Ui};
use tail_core::AppUsage;

use crate::components::{
    AppCard, EmptyState, EnhancedProgressBar, PageHeader, SectionDivider, StackedBarChart,
    StackedBarChartConfig, StatCard, TimeSlotData,
};
use crate::icons::IconCache;
use crate::theme::TaiLTheme;
use crate::utils::duration;

/// 仪表板视图
pub struct DashboardView<'a> {
    /// 应用使用数据
    app_usage: &'a [AppUsage],
    /// 主题
    theme: &'a TaiLTheme,
    /// 图标缓存（可变引用）
    icon_cache: &'a mut IconCache,
    /// 悬停的时间槽索引
    hovered_slot: Option<usize>,
}

impl<'a> DashboardView<'a> {
    pub fn new(
        app_usage: &'a [AppUsage],
        theme: &'a TaiLTheme,
        icon_cache: &'a mut IconCache,
    ) -> Self {
        Self {
            app_usage,
            theme,
            icon_cache,
            hovered_slot: None,
        }
    }

    /// 渲染仪表板
    pub fn show(&mut self, ui: &mut Ui) {
        // 页面标题
        ui.add(PageHeader::new("今日统计", "📅", self.theme).subtitle(&Self::get_date_string()));

        ui.add_space(self.theme.spacing);

        // KPI 卡片区域
        self.show_kpi_cards(ui);

        ui.add_space(self.theme.spacing);

        // 分隔线
        ui.add(SectionDivider::new(self.theme).with_title("时间分布 · 24小时"));

        ui.add_space(self.theme.spacing / 2.0);

        // 堆叠柱状图（iPhone 风格）
        self.show_stacked_chart(ui);

        ui.add_space(self.theme.spacing);

        // 分隔线
        ui.add(SectionDivider::new(self.theme).with_title("应用使用排行"));

        ui.add_space(self.theme.spacing / 2.0);

        // 应用列表
        self.show_app_list(ui);
    }

    /// 显示 KPI 卡片（增强版）
    fn show_kpi_cards(&self, ui: &mut Ui) {
        // 过滤掉空名称的应用
        let valid_apps: Vec<_> = self
            .app_usage
            .iter()
            .filter(|u| !u.app_name.is_empty())
            .collect();

        let total_seconds: i64 = valid_apps.iter().map(|u| u.total_seconds).sum();

        let app_count = valid_apps.len();
        let avg_per_app = if app_count > 0 {
            total_seconds / app_count as i64
        } else {
            0
        };

        // 计算生产力评分（基于分类，这里简化为使用最常用应用的占比）
        let productivity_score = self.calculate_productivity_score(&valid_apps, total_seconds);

        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = self.theme.spacing;

            // 总使用时间卡片 + 生产力评分（在副标题中显示）
            let first_card_subtitle = if total_seconds > 0 {
                Some(format!("生产力 {}%", productivity_score))
            } else {
                None
            };
            ui.add(
                StatCard::new(
                    "总使用时间",
                    &duration::format_duration(total_seconds),
                    "⏱️",
                    self.theme,
                )
                .accent_color(self.theme.primary_color)
                .with_subtitle_option(first_card_subtitle.as_deref()),
            );

            // 活跃应用数量
            ui.add(
                StatCard::new("活跃应用", &format!("{} 个", app_count), "📱", self.theme)
                    .accent_color(self.theme.accent_color),
            );

            // 平均每应用时长
            ui.add(
                StatCard::new(
                    "平均每应用",
                    &duration::format_duration(avg_per_app),
                    "📈",
                    self.theme,
                )
                .accent_color(self.theme.warning_color),
            );

            // 最常用应用
            if let Some(top_app) = valid_apps.first() {
                let icon = self.icon_cache.get_emoji(&top_app.app_name);
                let percentage = if total_seconds > 0 {
                    (top_app.total_seconds as f32 / total_seconds as f32) * 100.0
                } else {
                    0.0
                };
                ui.add(
                    StatCard::new("最常用", &top_app.app_name, icon, self.theme)
                        .subtitle(&format!(
                            "{} · {}%",
                            duration::format_duration(top_app.total_seconds),
                            percentage as u32
                        ))
                        .accent_color(self.theme.success_color),
                );
            }
        });
    }

    /// 计算生产力评分
    fn calculate_productivity_score(&self, valid_apps: &[&AppUsage], total_seconds: i64) -> u32 {
        if total_seconds == 0 {
            return 0;
        }

        // 简化的评分逻辑：基于应用使用分布
        // 理想情况下应该使用分类数据（工作、开发类应用得分更高）
        let mut score = 50u32; // 基础分

        // 如果最常用应用占比超过 50%，可能是专注工作
        if let Some(top_app) = valid_apps.first() {
            let top_ratio = (top_app.total_seconds as f32 / total_seconds as f32) * 100.0;
            if top_ratio > 50.0 {
                score += 20;
            }
        }

        // 应用数量适中（5-15个）表示多样化工作
        let app_count = valid_apps.len();
        if (5..=15).contains(&app_count) {
            score += 15;
        } else if app_count > 15 {
            score += 5; // 太多应用可能表示频繁切换
        }

        // 总时长适中（4-10小时）表示良好工作日
        let hours = total_seconds / 3600;
        if (4..=10).contains(&hours) {
            score += 15;
        }

        score.min(100)
    }

    /// 显示堆叠柱状图
    fn show_stacked_chart(&mut self, ui: &mut Ui) {
        let time_slots = self.create_time_slots();

        if time_slots.iter().all(|s| s.total_seconds == 0) {
            ui.add(EmptyState::new(
                "📊",
                "暂无时间分布数据",
                "活动数据会在这里显示",
                self.theme,
            ));
            return;
        }

        let config = StackedBarChartConfig {
            max_bar_height: 180.0,
            ..Default::default()
        };

        let chart = StackedBarChart::new(&time_slots, self.theme).with_config(config);

        self.hovered_slot = chart.show(ui);

        // 显示悬停提示
        if let Some(idx) = self.hovered_slot
            && let Some(slot) = time_slots.get(idx)
        {
            let mut top_apps: Vec<_> = slot
                .app_durations
                .iter()
                .map(|(k, v)| (k.clone(), *v))
                .collect();
            top_apps.sort_by(|a, b| b.1.cmp(&a.1));

            use crate::components::StackedBarTooltip;
            let tooltip = StackedBarTooltip {
                hour: slot.hour,
                total_seconds: slot.total_seconds,
                top_apps,
            };
            tooltip.show(ui, self.theme);
        }
    }

    /// 创建时间槽数据（按小时分组）
    fn create_time_slots(&self) -> Vec<TimeSlotData> {
        let mut slots: Vec<TimeSlotData> = (0..24).map(TimeSlotData::new).collect();

        for usage in self.app_usage {
            if usage.app_name.is_empty() {
                continue;
            }

            for event in &usage.window_events {
                if event.is_afk {
                    continue;
                }

                let local_time = event.timestamp.with_timezone(&Local);
                let hour = local_time.hour();
                if hour < 24
                    && let Some(slot) = slots.get_mut(hour as usize)
                {
                    slot.add_app(usage.app_name.clone(), event.duration_secs);
                }
            }
        }

        slots
    }

    /// 显示应用列表
    fn show_app_list(&mut self, ui: &mut Ui) {
        if self.app_usage.is_empty() {
            ui.add(EmptyState::new(
                "📭",
                "暂无数据",
                "开始使用应用后，这里会显示使用统计",
                self.theme,
            ));
            return;
        }

        let total_seconds: i64 = self.app_usage.iter().map(|u| u.total_seconds).sum();

        // 收集需要的数据，避免借用冲突
        // 过滤掉空名称的应用
        let app_data: Vec<_> = self
            .app_usage
            .iter()
            .enumerate()
            .filter(|(_, usage)| !usage.app_name.is_empty())
            .map(|(idx, usage)| {
                let percentage = if total_seconds > 0 {
                    (usage.total_seconds as f32 / total_seconds as f32) * 100.0
                } else {
                    0.0
                };
                let window_title = usage.window_events.last().map(|e| e.window_title.clone());
                (
                    idx,
                    usage.app_name.clone(),
                    usage.total_seconds,
                    percentage,
                    window_title,
                )
            })
            .collect();

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = self.theme.spacing / 2.0;

                for (idx, app_name, total_secs, percentage, window_title) in app_data {
                    let mut card = AppCard::new(
                        &app_name,
                        &app_name, // TODO: 使用别名
                        total_secs,
                        percentage,
                        idx + 1,
                        self.theme,
                        self.icon_cache,
                        ui.ctx(),
                    );

                    if let Some(ref title) = window_title {
                        card = card.with_window_title(title);
                    }

                    let response = card.show(ui);

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
                        .color(self.theme.text_color),
                );
            });

            // 目标进度
            if let Some(goal) = self.goal_seconds {
                let fraction = (self.total_seconds as f32 / goal as f32).min(1.5);
                let remaining = goal - self.total_seconds;

                ui.add_space(8.0);

                ui.add(
                    EnhancedProgressBar::new(fraction.min(1.0), self.theme)
                        .height(10.0)
                        .show_percentage(true)
                        .label("今日目标"),
                );

                if remaining > 0 {
                    ui.label(
                        egui::RichText::new(format!(
                            "距离目标还剩 {}",
                            duration::format_duration(remaining)
                        ))
                        .size(self.theme.small_size)
                        .color(self.theme.secondary_text_color),
                    );
                } else {
                    ui.label(
                        egui::RichText::new("🎉 已达成今日目标！")
                            .size(self.theme.small_size)
                            .color(self.theme.success_color),
                    );
                }
            }
        });
    }
}
