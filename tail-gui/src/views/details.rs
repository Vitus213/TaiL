//! TaiL GUI - 详细视图
//!
//! 提供详细的应用使用记录列表，支持搜索、过滤和右键菜单

use chrono::{DateTime, Datelike, Local, Utc};
use egui::{ScrollArea, TextEdit, Ui, Vec2};
use tail_core::AppUsage;

use crate::components::{EmptyState, PageHeader, SectionDivider};
use crate::icons::{AppIcon, IconCache};
use crate::theme::TaiLTheme;
use crate::utils::duration;

/// 详细视图
pub struct DetailsView {
    /// 搜索关键词
    search_query: String,
    /// 选中的应用（用于右键菜单）
    selected_app: Option<String>,
    /// 时间过滤状态
    time_filter: TimeFilter,
    /// 数据缓存（扁平化的窗口事件）
    flat_data: Vec<WindowEventRecord>,
}

/// 时间过滤器
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeFilter {
    All,
    Today,
    ThisWeek,
    ThisMonth,
}

/// 窗口事件记录（用于列表显示）
#[derive(Debug, Clone)]
pub struct WindowEventRecord {
    pub app_name: String,
    pub window_title: String,
    pub start_time: DateTime<Utc>,
    pub duration_secs: i64,
    pub is_afk: bool,
}

impl Default for DetailsView {
    fn default() -> Self {
        Self::new()
    }
}

impl DetailsView {
    pub fn new() -> Self {
        Self {
            search_query: String::new(),
            selected_app: None,
            time_filter: TimeFilter::All,
            flat_data: Vec::new(),
        }
    }

    /// 更新扁平化数据
    pub fn update_data(&mut self, app_usage: &[AppUsage]) {
        self.flat_data.clear();
        for usage in app_usage {
            for event in &usage.window_events {
                self.flat_data.push(WindowEventRecord {
                    app_name: usage.app_name.clone(),
                    window_title: event.window_title.clone(),
                    start_time: event.timestamp,
                    duration_secs: event.duration_secs,
                    is_afk: event.is_afk,
                });
            }
        }
        // 按开始时间降序排序
        self.flat_data
            .sort_by(|a, b| b.start_time.cmp(&a.start_time));
    }

    /// 渲染详细视图
    pub fn show(&mut self, ui: &mut Ui, theme: &TaiLTheme, icon_cache: &mut IconCache) {
        // 页面标题
        ui.add(PageHeader::new("详细记录", "📋", theme));
        ui.add_space(theme.spacing);

        // 搜索和过滤区域
        self.show_filters(ui, theme);
        ui.add_space(theme.spacing);

        // 分隔线
        ui.add(SectionDivider::new(theme).with_title("记录列表"));
        ui.add_space(theme.spacing / 2.0);

        // 数据列表
        self.show_data_list(ui, theme, icon_cache);
    }

    /// 显示搜索和过滤区域
    fn show_filters(&mut self, ui: &mut Ui, theme: &TaiLTheme) {
        ui.horizontal(|ui| {
            // 搜索框
            ui.label(egui::RichText::new("🔍").size(theme.body_size));
            ui.add_space(4.0);
            let response = ui.add_sized(
                Vec2::new(300.0, 24.0),
                TextEdit::singleline(&mut self.search_query)
                    .hint_text("搜索应用或窗口标题...")
                    .frame(true),
            );
            if response.lost_focus() || response.changed() {
                ui.ctx().request_repaint();
            }
        });

        ui.add_space(8.0);

        // 时间过滤器
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("时间范围:")
                    .size(theme.small_size)
                    .color(theme.secondary_text_color),
            );
            ui.add_space(8.0);

            let filters = [
                (TimeFilter::All, "全部"),
                (TimeFilter::Today, "今天"),
                (TimeFilter::ThisWeek, "本周"),
                (TimeFilter::ThisMonth, "本月"),
            ];

            for (filter, label) in filters {
                let is_selected = self.time_filter == filter;
                if ui
                    .add(
                        egui::Button::new(egui::RichText::new(label).size(theme.small_size).color(
                            if is_selected {
                                egui::Color32::WHITE
                            } else {
                                theme.text_color
                            },
                        ))
                        .fill(if is_selected {
                            theme.primary_color
                        } else {
                            egui::Color32::TRANSPARENT
                        })
                        .stroke(if is_selected {
                            egui::Stroke::NONE
                        } else {
                            egui::Stroke::new(1.0, theme.divider_color)
                        })
                        .rounding(4.0)
                        .min_size(Vec2::new(60.0, 24.0)),
                    )
                    .clicked()
                {
                    self.time_filter = filter;
                    ui.ctx().request_repaint();
                }
                ui.add_space(4.0);
            }
        });
    }

    /// 显示数据列表
    fn show_data_list(&mut self, ui: &mut Ui, theme: &TaiLTheme, icon_cache: &mut IconCache) {
        // 收集过滤后的数据（克隆以避免借用问题）
        let filtered_data: Vec<WindowEventRecord> = self
            .filter_data()
            .iter()
            .take(500)
            .map(|r| (*r).clone())
            .collect();

        if filtered_data.is_empty() {
            ui.add(EmptyState::new(
                "🔍",
                "没有找到匹配的记录",
                "尝试调整搜索关键词或时间范围",
                theme,
            ));
            return;
        }

        // 列表头部
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            ui.add_space(26.0); // 图标宽度
            ui.label(
                egui::RichText::new("应用")
                    .size(theme.small_size)
                    .color(theme.secondary_text_color),
            );
            ui.label(
                egui::RichText::new("窗口标题")
                    .size(theme.small_size)
                    .color(theme.secondary_text_color),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new("时长")
                        .size(theme.small_size)
                        .color(theme.secondary_text_color),
                );
                ui.label(
                    egui::RichText::new("时间")
                        .size(theme.small_size)
                        .color(theme.secondary_text_color),
                );
            });
        });

        ui.add_space(8.0);

        // 数据列表
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 8.0;

                for record in filtered_data.iter() {
                    self.show_record_row(ui, record, theme, icon_cache);
                }
            });
    }

    /// 显示单行记录
    fn show_record_row(
        &mut self,
        ui: &mut Ui,
        record: &WindowEventRecord,
        theme: &TaiLTheme,
        icon_cache: &mut IconCache,
    ) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;

            // 应用图标（使用真实应用图标）
            AppIcon::new(&record.app_name)
                .size(20.0)
                .show(ui, icon_cache);

            // 应用名
            ui.label(
                egui::RichText::new(&record.app_name)
                    .size(theme.body_size)
                    .color(theme.text_color),
            );

            // 窗口标题（按字符截断，避免 UTF-8 字符边界问题）
            let title = if record.window_title.chars().count() > 50 {
                format!(
                    "{}...",
                    record.window_title.chars().take(47).collect::<String>()
                )
            } else {
                record.window_title.clone()
            };
            ui.label(
                egui::RichText::new(title)
                    .size(theme.body_size)
                    .color(theme.secondary_text_color),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // 时长
                let duration_str = duration::format_duration(record.duration_secs);
                ui.label(
                    egui::RichText::new(duration_str)
                        .size(theme.small_size)
                        .color(theme.text_color),
                );

                // 时间
                let local_time = record.start_time.with_timezone(&Local);
                let time_str = local_time.format("%H:%M").to_string();
                ui.label(
                    egui::RichText::new(time_str)
                        .size(theme.small_size)
                        .color(theme.secondary_text_color),
                );
            });
        });

        // 右键菜单（通过添加隐藏的可点击区域）
        let response = ui.allocate_rect(
            egui::Rect::from_min_max(ui.min_rect().min, ui.min_rect().max),
            egui::Sense::click(),
        );

        response.context_menu(|ui| {
            ui.label(
                egui::RichText::new(&record.app_name)
                    .strong()
                    .size(theme.body_size),
            );
            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            if ui.button("📝 设置别名").clicked() {
                self.selected_app = Some(record.app_name.clone());
                ui.close_menu();
                // TODO: 打开别名设置对话框
            }
            if ui.button("📁 关联分类").clicked() {
                self.selected_app = Some(record.app_name.clone());
                ui.close_menu();
                // TODO: 打开分类选择对话框
            }
            if ui.button("🚫 忽略此应用").clicked() {
                self.selected_app = Some(record.app_name.clone());
                ui.close_menu();
                // TODO: 标记为忽略
            }
        });
    }

    /// 过滤数据
    fn filter_data(&self) -> Vec<&WindowEventRecord> {
        let mut result: Vec<&WindowEventRecord> = self
            .flat_data
            .iter()
            .filter(|record| {
                // 过滤 AFK 事件
                if record.is_afk {
                    return false;
                }

                // 搜索过滤
                if !self.search_query.is_empty() {
                    let query = self.search_query.to_lowercase();
                    let app_match = record.app_name.to_lowercase().contains(&query);
                    let title_match = record.window_title.to_lowercase().contains(&query);
                    if !app_match && !title_match {
                        return false;
                    }
                }

                // 时间过滤
                match self.time_filter {
                    TimeFilter::All => true,
                    TimeFilter::Today => {
                        let now = Local::now();
                        let today_start = now
                            .date_naive()
                            .and_hms_opt(0, 0, 0)
                            .unwrap()
                            .and_local_timezone(Local)
                            .unwrap()
                            .with_timezone(&Utc);
                        record.start_time >= today_start
                    }
                    TimeFilter::ThisWeek => {
                        // 本周：从本周一到今天
                        let now = Local::now();
                        let weekday = now.weekday().num_days_from_monday();
                        let week_start = now.date_naive() - chrono::Duration::days(weekday as i64);
                        let week_start_utc = week_start
                            .and_hms_opt(0, 0, 0)
                            .unwrap()
                            .and_local_timezone(Local)
                            .unwrap()
                            .with_timezone(&Utc);
                        record.start_time >= week_start_utc
                    }
                    TimeFilter::ThisMonth => {
                        // 本月：从本月1号到今天
                        let now = Local::now();
                        let month_start = now
                            .date_naive()
                            .with_day(1)
                            .unwrap()
                            .and_hms_opt(0, 0, 0)
                            .unwrap()
                            .and_local_timezone(Local)
                            .unwrap()
                            .with_timezone(&Utc);
                        record.start_time >= month_start
                    }
                }
            })
            .collect();

        // 最多显示 1000 条
        result.truncate(1000);
        result
    }
}
