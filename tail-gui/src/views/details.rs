//! TaiL GUI - 详细视图
//!
//! 提供详细的应用使用记录列表，支持搜索、过滤和右键菜单

use chrono::{DateTime, Datelike, Local, NaiveDate, Utc};
use egui::{ScrollArea, TextEdit, Ui, Vec2};
use tail_core::AppUsage;
use tail_core::time::range::TimeRangeCalculator;

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
    /// 自定义时间范围 - 开始日期
    custom_start_date: Option<NaiveDate>,
    /// 自定义时间范围 - 结束日期
    custom_end_date: Option<NaiveDate>,
    /// 是否显示自定义时间范围选择器
    show_custom_range: bool,
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
    Custom,
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
        // 默认自定义范围为最近7天
        let now = Local::now();
        let today = now.date_naive();
        Self {
            search_query: String::new(),
            selected_app: None,
            time_filter: TimeFilter::All,
            custom_start_date: Some(today - chrono::Duration::days(7)),
            custom_end_date: Some(today),
            show_custom_range: false,
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
                (TimeFilter::Custom, "自定义"),
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
                    // 点击自定义按钮时展开选择器
                    if filter == TimeFilter::Custom {
                        self.show_custom_range = true;
                    }
                    ui.ctx().request_repaint();
                }
                ui.add_space(4.0);
            }
        });

        // 自定义时间范围选择器
        if self.time_filter == TimeFilter::Custom && self.show_custom_range {
            ui.add_space(8.0);
            self.show_custom_date_range(ui, theme);
        }
    }

    /// 显示自定义日期范围选择器
    fn show_custom_date_range(&mut self, ui: &mut Ui, theme: &TaiLTheme) {
        egui::Frame {
            fill: egui::Color32::from_rgb(50, 50, 60),
            stroke: egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 100, 120)),
            rounding: egui::Rounding::same(8.0),
            inner_margin: egui::Margin::symmetric(12.0, 8.0),
            outer_margin: egui::Margin::ZERO,
            shadow: egui::epaint::Shadow::NONE,
        }
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new("开始日期")
                            .size(theme.small_size)
                            .color(egui::Color32::WHITE),
                    );
                    ui.add_space(4.0);
                    self.show_date_picker(ui, theme, true);
                });

                ui.add_space(16.0);

                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new("结束日期")
                            .size(theme.small_size)
                            .color(egui::Color32::WHITE),
                    );
                    ui.add_space(4.0);
                    self.show_date_picker(ui, theme, false);
                });

                ui.add_space(16.0);

                // 快捷选择按钮
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new("快捷选择")
                            .size(theme.small_size)
                            .color(egui::Color32::WHITE),
                    );
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("最近7天").color(egui::Color32::WHITE),
                                )
                                .fill(egui::Color32::from_rgb(80, 80, 100))
                                .stroke(egui::Stroke::new(
                                    1.0,
                                    egui::Color32::from_rgb(120, 120, 140),
                                ))
                                .rounding(4.0),
                            )
                            .clicked()
                        {
                            let now = Local::now();
                            let today = now.date_naive();
                            self.custom_start_date = Some(today - chrono::Duration::days(7));
                            self.custom_end_date = Some(today);
                            ui.ctx().request_repaint();
                        }
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("最近30天").color(egui::Color32::WHITE),
                                )
                                .fill(egui::Color32::from_rgb(80, 80, 100))
                                .stroke(egui::Stroke::new(
                                    1.0,
                                    egui::Color32::from_rgb(120, 120, 140),
                                ))
                                .rounding(4.0),
                            )
                            .clicked()
                        {
                            let now = Local::now();
                            let today = now.date_naive();
                            self.custom_start_date = Some(today - chrono::Duration::days(30));
                            self.custom_end_date = Some(today);
                            ui.ctx().request_repaint();
                        }
                    });
                    ui.horizontal(|ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("本月").color(egui::Color32::WHITE),
                                )
                                .fill(egui::Color32::from_rgb(80, 80, 100))
                                .stroke(egui::Stroke::new(
                                    1.0,
                                    egui::Color32::from_rgb(120, 120, 140),
                                ))
                                .rounding(4.0),
                            )
                            .clicked()
                        {
                            let now = Local::now();
                            self.custom_start_date =
                                Some(NaiveDate::from_ymd_opt(now.year(), now.month(), 1).unwrap());
                            let last_day = if now.month() == 12 {
                                NaiveDate::from_ymd_opt(now.year() + 1, 1, 1).unwrap()
                                    - chrono::Duration::days(1)
                            } else {
                                NaiveDate::from_ymd_opt(now.year(), now.month() + 1, 1).unwrap()
                                    - chrono::Duration::days(1)
                            };
                            self.custom_end_date = Some(last_day);
                            ui.ctx().request_repaint();
                        }
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("上月").color(egui::Color32::WHITE),
                                )
                                .fill(egui::Color32::from_rgb(80, 80, 100))
                                .stroke(egui::Stroke::new(
                                    1.0,
                                    egui::Color32::from_rgb(120, 120, 140),
                                ))
                                .rounding(4.0),
                            )
                            .clicked()
                        {
                            let now = Local::now();
                            let (year, month) = if now.month() == 1 {
                                (now.year() - 1, 12)
                            } else {
                                (now.year(), now.month() - 1)
                            };
                            self.custom_start_date =
                                Some(NaiveDate::from_ymd_opt(year, month, 1).unwrap());
                            let last_day = if month == 12 {
                                NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
                                    - chrono::Duration::days(1)
                            } else {
                                NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap()
                                    - chrono::Duration::days(1)
                            };
                            self.custom_end_date = Some(last_day);
                            ui.ctx().request_repaint();
                        }
                    });
                });
            });

            // 显示当前选择的时间范围
            if let (Some(start), Some(end)) = (self.custom_start_date, self.custom_end_date) {
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);
                let days = (end - start).num_days() + 1;
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!(
                            "📅 {} ~ {} (共 {} 天)",
                            start.format("%Y-%m-%d"),
                            end.format("%Y-%m-%d"),
                            days
                        ))
                        .size(theme.body_size)
                        .color(egui::Color32::WHITE)
                        .strong(),
                    );
                });
            }
        });
    }

    /// 显示日期选择器
    fn show_date_picker(&mut self, ui: &mut Ui, theme: &TaiLTheme, is_start: bool) {
        let date = if is_start {
            self.custom_start_date
        } else {
            self.custom_end_date
        };

        if let Some(d) = date {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;

                // 年份选择
                let mut year = d.year();
                let btn = ui.add_sized(
                    Vec2::new(24.0, 22.0),
                    egui::Button::new(
                        egui::RichText::new("<")
                            .size(theme.body_size)
                            .color(egui::Color32::WHITE),
                    )
                    .fill(egui::Color32::from_rgb(80, 80, 100))
                    .stroke(egui::Stroke::new(
                        1.0,
                        egui::Color32::from_rgb(120, 120, 140),
                    ))
                    .rounding(4.0),
                );
                if btn.hovered() {
                    ui.ctx().request_repaint();
                }
                if btn.clicked() {
                    year -= 1;
                    self.update_date(is_start, year, d.month(), d.day(), ui.ctx());
                }
                ui.add_sized(
                    Vec2::new(50.0, 22.0),
                    egui::Label::new(
                        egui::RichText::new(format!("{:04}", year))
                            .size(theme.body_size)
                            .color(egui::Color32::WHITE)
                            .strong(),
                    ),
                );
                let btn = ui.add_sized(
                    Vec2::new(24.0, 22.0),
                    egui::Button::new(
                        egui::RichText::new(">")
                            .size(theme.body_size)
                            .color(egui::Color32::WHITE),
                    )
                    .fill(egui::Color32::from_rgb(80, 80, 100))
                    .stroke(egui::Stroke::new(
                        1.0,
                        egui::Color32::from_rgb(120, 120, 140),
                    ))
                    .rounding(4.0),
                );
                if btn.hovered() {
                    ui.ctx().request_repaint();
                }
                if btn.clicked() {
                    year += 1;
                    self.update_date(is_start, year, d.month(), d.day(), ui.ctx());
                }

                // 分隔符
                ui.label(
                    egui::RichText::new("-")
                        .size(theme.body_size)
                        .color(egui::Color32::from_gray(180)),
                );

                // 月份选择
                let mut month = d.month();
                let btn = ui.add_sized(
                    Vec2::new(24.0, 22.0),
                    egui::Button::new(
                        egui::RichText::new("<")
                            .size(theme.body_size)
                            .color(egui::Color32::WHITE),
                    )
                    .fill(egui::Color32::from_rgb(80, 80, 100))
                    .stroke(egui::Stroke::new(
                        1.0,
                        egui::Color32::from_rgb(120, 120, 140),
                    ))
                    .rounding(4.0),
                );
                if month > 1 && btn.clicked() {
                    month -= 1;
                    self.update_date(is_start, year, month, d.day(), ui.ctx());
                }
                ui.add_sized(
                    Vec2::new(30.0, 22.0),
                    egui::Label::new(
                        egui::RichText::new(format!("{:02}", month))
                            .size(theme.body_size)
                            .color(egui::Color32::WHITE)
                            .strong(),
                    ),
                );
                let btn = ui.add_sized(
                    Vec2::new(24.0, 22.0),
                    egui::Button::new(
                        egui::RichText::new(">")
                            .size(theme.body_size)
                            .color(egui::Color32::WHITE),
                    )
                    .fill(egui::Color32::from_rgb(80, 80, 100))
                    .stroke(egui::Stroke::new(
                        1.0,
                        egui::Color32::from_rgb(120, 120, 140),
                    ))
                    .rounding(4.0),
                );
                if month < 12 && btn.clicked() {
                    month += 1;
                    self.update_date(is_start, year, month, d.day(), ui.ctx());
                }

                // 分隔符
                ui.label(
                    egui::RichText::new("-")
                        .size(theme.body_size)
                        .color(egui::Color32::from_gray(180)),
                );

                // 日期选择
                let mut day = d.day();
                let days_in_month = Self::days_in_month(year, month);
                let btn = ui.add_sized(
                    Vec2::new(24.0, 22.0),
                    egui::Button::new(
                        egui::RichText::new("<")
                            .size(theme.body_size)
                            .color(egui::Color32::WHITE),
                    )
                    .fill(egui::Color32::from_rgb(80, 80, 100))
                    .stroke(egui::Stroke::new(
                        1.0,
                        egui::Color32::from_rgb(120, 120, 140),
                    ))
                    .rounding(4.0),
                );
                if day > 1 && btn.clicked() {
                    day -= 1;
                    self.update_date(is_start, year, month, day, ui.ctx());
                }
                ui.add_sized(
                    Vec2::new(30.0, 22.0),
                    egui::Label::new(
                        egui::RichText::new(format!("{:02}", day))
                            .size(theme.body_size)
                            .color(egui::Color32::WHITE)
                            .strong(),
                    ),
                );
                let btn = ui.add_sized(
                    Vec2::new(24.0, 22.0),
                    egui::Button::new(
                        egui::RichText::new(">")
                            .size(theme.body_size)
                            .color(egui::Color32::WHITE),
                    )
                    .fill(egui::Color32::from_rgb(80, 80, 100))
                    .stroke(egui::Stroke::new(
                        1.0,
                        egui::Color32::from_rgb(120, 120, 140),
                    ))
                    .rounding(4.0),
                );
                if day < days_in_month && btn.clicked() {
                    day += 1;
                    self.update_date(is_start, year, month, day, ui.ctx());
                }

                // 星期几显示
                let weekday = d.weekday();
                let weekday_names = ["一", "二", "三", "四", "五", "六", "日"];
                ui.label(
                    egui::RichText::new(format!(
                        " 周{}",
                        weekday_names[weekday.num_days_from_monday() as usize]
                    ))
                    .size(theme.small_size)
                    .color(egui::Color32::from_gray(200)),
                );
            });
        }
    }

    /// 更新日期
    fn update_date(
        &mut self,
        is_start: bool,
        year: i32,
        month: u32,
        day: u32,
        ctx: &egui::Context,
    ) {
        let days_in_month = Self::days_in_month(year, month);
        let day = day.min(days_in_month);

        if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
            if is_start {
                self.custom_start_date = Some(date);
                // 确保开始日期不晚于结束日期
                if let Some(end) = self.custom_end_date
                    && date > end
                {
                    self.custom_end_date = Some(date);
                }
            } else {
                self.custom_end_date = Some(date);
                // 确保结束日期不早于开始日期
                if let Some(start) = self.custom_start_date
                    && date < start
                {
                    self.custom_start_date = Some(date);
                }
            }
            ctx.request_repaint();
        }
    }

    /// 获取某年某月的天数
    fn days_in_month(year: i32, month: u32) -> u32 {
        if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap()
        }
        .signed_duration_since(NaiveDate::from_ymd_opt(year, month, 1).unwrap())
        .num_days() as u32
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

                // 时间过滤 - 使用统一的时间范围计算器
                match self.time_filter {
                    TimeFilter::All => true,
                    TimeFilter::Today => {
                        let range = TimeRangeCalculator::today();
                        record.start_time >= range.start
                    }
                    TimeFilter::ThisWeek => {
                        let range = TimeRangeCalculator::this_week();
                        record.start_time >= range.start
                    }
                    TimeFilter::ThisMonth => {
                        let range = TimeRangeCalculator::this_month();
                        record.start_time >= range.start
                    }
                    TimeFilter::Custom => {
                        // 自定义时间范围
                        if let (Some(start_date), Some(end_date)) =
                            (self.custom_start_date, self.custom_end_date)
                        {
                            // 计算开始和结束时间的 UTC 时间戳
                            let start_utc = start_date
                                .and_hms_opt(0, 0, 0)
                                .unwrap()
                                .and_local_timezone(Local)
                                .unwrap()
                                .with_timezone(&Utc);
                            let end_utc = end_date
                                .and_hms_opt(23, 59, 59)
                                .unwrap()
                                .and_local_timezone(Local)
                                .unwrap()
                                .with_timezone(&Utc);
                            record.start_time >= start_utc && record.start_time <= end_utc
                        } else {
                            true
                        }
                    }
                }
            })
            .collect();

        // 最多显示 1000 条
        result.truncate(1000);
        result
    }
}
