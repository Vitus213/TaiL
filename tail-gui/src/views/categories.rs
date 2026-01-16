//! TaiL GUI - 分类视图

use chrono::{DateTime, Utc};
use egui::{Color32, Rounding, ScrollArea, Stroke, Ui, Vec2};
use std::collections::HashSet;
use tail_core::{AppUsage, AppUsageInCategory, CATEGORY_ICONS, Category, CategoryUsage};

use crate::components::{EmptyState, PageHeader, SectionDivider, StatCard};
use crate::components::chart::{ChartDataBuilder, ChartGroupMode, ChartTimeGranularity, StackedBarChart, StackedBarChartConfig, StackedBarTooltip};
use crate::icons::ui_icons::categories as icons;
use crate::theme::TaiLTheme;
use crate::utils::duration;

/// 预定义颜色选项
const CATEGORY_COLORS: &[(&str, Color32)] = &[
    ("蓝色", Color32::from_rgb(74, 144, 226)),
    ("青色", Color32::from_rgb(52, 168, 83)),
    ("绿色", Color32::from_rgb(76, 175, 80)),
    ("黄色", Color32::from_rgb(255, 205, 86)),
    ("橙色", Color32::from_rgb(255, 152, 0)),
    ("红色", Color32::from_rgb(255, 99, 71)),
    ("紫色", Color32::from_rgb(155, 89, 182)),
    ("粉色", Color32::from_rgb(233, 30, 99)),
    ("青绿", Color32::from_rgb(0, 200, 150)),
    ("灰色", Color32::from_rgb(120, 144, 156)),
];

/// 分类视图操作
#[derive(Debug)]
pub enum CategoryAction {
    /// 添加分类
    AddCategory(Category),
    /// 更新分类
    UpdateCategory(Category),
    /// 删除分类
    DeleteCategory(i64),
    /// 为应用设置分类
    SetAppCategories(String, Vec<i64>),
    /// 从分类中移除应用
    RemoveAppFromCategory(String, i64),
    /// 加载应用当前分类
    LoadAppCategories(String),
}

/// 分类视图状态
pub struct CategoriesView {
    /// 分类使用统计数据
    category_usage: Vec<CategoryUsage>,
    /// 所有分类列表
    categories: Vec<Category>,
    /// 应用使用数据（用于堆叠柱形图）
    app_usage: Vec<AppUsage>,
    /// 主题
    theme: TaiLTheme,
    /// 是否显示添加分类对话框
    show_add_dialog: bool,
    /// 是否显示编辑分类对话框
    show_edit_dialog: bool,
    /// 是否显示应用归类对话框
    show_assign_dialog: bool,
    /// 新分类名称
    new_category_name: String,
    /// 新分类图标
    new_category_icon: String,
    /// 新分类颜色
    new_category_color: Option<String>,
    /// 选中的分类 ID（用于编辑）
    selected_category_id: Option<i64>,
    /// 选中的应用名称（用于归类）
    selected_app_name: Option<String>,
    /// 当前应用选中的分类 ID 列表（用于归类对话框）
    selected_category_ids: Vec<i64>,
    /// 所有应用名称列表
    all_apps: Vec<String>,
    /// 图标选择器是否展开
    show_icon_picker: bool,
    /// 是否需要刷新数据
    needs_refresh: bool,
    /// 悬停的时间槽索引
    hovered_slot: Option<usize>,
    /// 待处理的操作
    pending_action: Option<CategoryAction>,
    /// 加载的应用分类（用于回调响应）
    loaded_app_categories: Vec<i64>,
}

impl CategoriesView {
    pub fn new(theme: TaiLTheme) -> Self {
        Self {
            category_usage: Vec::new(),
            categories: Vec::new(),
            app_usage: Vec::new(),
            theme,
            show_add_dialog: false,
            show_edit_dialog: false,
            show_assign_dialog: false,
            new_category_name: String::new(),
            new_category_icon: "🗀".to_string(),
            new_category_color: Some("#4A90E2".to_string()),
            selected_category_id: None,
            selected_app_name: None,
            selected_category_ids: Vec::new(),
            all_apps: Vec::new(),
            show_icon_picker: false,
            needs_refresh: false,
            hovered_slot: None,
            pending_action: None,
            loaded_app_categories: Vec::new(),
        }
    }

    /// 检查是否需要刷新数据
    pub fn needs_refresh(&self) -> bool {
        self.needs_refresh
    }

    /// 清除刷新标志
    pub fn clear_refresh_flag(&mut self) {
        self.needs_refresh = false;
    }

    /// 加载分类数据（接收预加载的数据）
    pub fn load_data(
        &mut self,
        category_usage: Vec<CategoryUsage>,
        categories: Vec<Category>,
        all_apps: Vec<String>,
        app_usage: Vec<AppUsage>,
    ) {
        self.category_usage = category_usage;
        self.categories = categories;
        self.all_apps = all_apps;
        self.app_usage = app_usage;
    }

    /// 设置加载的应用分类（响应 LoadAppCategories 操作）
    pub fn set_app_categories(&mut self, category_ids: Vec<i64>) {
        self.selected_category_ids = category_ids;
    }

    /// 取出并清除待处理的操作
    pub fn take_action(&mut self) -> Option<CategoryAction> {
        self.pending_action.take()
    }

    /// 渲染分类视图
    pub fn show(&mut self, ui: &mut Ui) -> Option<CategoryAction> {
        // 页面标题
        ui.add(
            PageHeader::new("应用分类", icons::PAGE_ICON, &self.theme)
                .subtitle("按分类查看应用使用时间"),
        );

        ui.add_space(self.theme.spacing);

        // 工具栏
        self.show_toolbar(ui);

        ui.add_space(self.theme.spacing);

        // 统计卡片
        self.show_stat_cards(ui);

        ui.add_space(self.theme.spacing);

        // 时间分布堆叠柱形图（按分类）
        ui.add(SectionDivider::new(&self.theme).with_title("时间分布 · 按分类堆叠"));
        ui.add_space(self.theme.spacing / 2.0);
        self.show_stacked_chart(ui);

        ui.add_space(self.theme.spacing);

        // 分隔线
        ui.add(SectionDivider::new(&self.theme).with_title("分类统计"));

        ui.add_space(self.theme.spacing / 2.0);

        // 分类列表和柱形图
        self.show_category_list(ui);

        // 对话框
        self.show_dialogs(ui);

        // 取出待处理的操作
        self.take_action()
    }

    /// 显示工具栏
    fn show_toolbar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui.button("+ 添加分类").clicked() {
                self.show_add_dialog = true;
                self.new_category_name.clear();
                self.new_category_icon = "🗀".to_string();
                self.new_category_color = Some("#4A90E2".to_string());
            }

            ui.add_space(self.theme.spacing / 2.0);

            if ui.button("# 管理应用分类").clicked() {
                self.show_assign_dialog = true;
            }
        });
    }

    /// 显示统计卡片
    fn show_stat_cards(&self, ui: &mut Ui) {
        let total_seconds: i64 = self.category_usage.iter().map(|c| c.total_seconds).sum();

        let category_count = self.categories.len();
        let categorized_apps: usize = self.category_usage.iter().map(|c| c.app_count).sum();

        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = self.theme.spacing;

            // 总分类数
            ui.add(
                StatCard::new(
                    "分类总数",
                    &format!("{} 个", category_count),
                    icons::CATEGORY_COUNT,
                    &self.theme,
                )
                .accent_color(self.theme.primary_color),
            );

            // 已分类应用数
            ui.add(
                StatCard::new(
                    "已分类应用",
                    &format!("{} 个", categorized_apps),
                    icons::APP_COUNT,
                    &self.theme,
                )
                .accent_color(self.theme.accent_color),
            );

            // 总使用时间
            ui.add(
                StatCard::new(
                    "总使用时间",
                    &duration::format_duration(total_seconds),
                    icons::TOTAL_TIME,
                    &self.theme,
                )
                .accent_color(self.theme.success_color),
            );

            // 最常用分类
            if let Some(top_category) = self.category_usage.first() {
                ui.add(
                    StatCard::new(
                        "最常用分类",
                        &top_category.category.name,
                        &top_category.category.icon,
                        &self.theme,
                    )
                    .subtitle(&duration::format_duration(top_category.total_seconds))
                    .accent_color(self.theme.warning_color),
                );
            }
        });
    }

    /// 显示堆叠柱状图（按分类堆叠）
    fn show_stacked_chart(&mut self, ui: &mut Ui) {
        if self.app_usage.is_empty() {
            ui.add(EmptyState::new(
                "📊",
                "暂无时间分布数据",
                "活动数据会在这里显示",
                &self.theme,
            ));
            return;
        }

        // 不使用 with_repository，仅使用已有数据
        let chart_data = ChartDataBuilder::new(&self.app_usage)
            .with_granularity(ChartTimeGranularity::Day)
            .with_group_mode(ChartGroupMode::ByCategory)
            .build();

        if chart_data.time_slots.iter().all(|s| s.total_seconds == 0) {
            ui.add(EmptyState::new(
                "📊",
                "暂无时间分布数据",
                "活动数据会在这里显示",
                &self.theme,
            ));
            return;
        }

        let config = StackedBarChartConfig {
            max_bar_height: 180.0,
            ..Default::default()
        };

        let chart = StackedBarChart::new(&chart_data, &self.theme).with_config(config);
        self.hovered_slot = chart.show(ui);

        // 显示悬停提示
        if let Some(idx) = self.hovered_slot
            && let Some(slot) = chart_data.time_slots.get(idx)
        {
            let tooltip = StackedBarTooltip::new(slot);
            tooltip.show(ui, &self.theme);
        }
    }

    /// 显示分类列表
    fn show_category_list(&mut self, ui: &mut Ui) {
        if self.category_usage.is_empty() && self.all_apps.is_empty() {
            ui.add(EmptyState::new(
                icons::EMPTY_STATE,
                "暂无分类数据",
                "创建分类并为应用分配分类后，这里会显示统计信息",
                &self.theme,
            ));
            return;
        }

        let total_seconds: i64 = self.category_usage.iter().map(|c| c.total_seconds).sum();

        // 收集需要的数据，避免借用冲突
        let category_data: Vec<_> = self
            .category_usage
            .iter()
            .map(|usage| {
                let percentage = if total_seconds > 0 {
                    (usage.total_seconds as f32 / total_seconds as f32) * 100.0
                } else {
                    0.0
                };
                let color = usage
                    .category
                    .color
                    .as_ref()
                    .and_then(|c| Self::parse_color(c))
                    .unwrap_or(self.theme.primary_color);
                let color_str = usage.category.color.clone();
                (
                    usage.category.id,
                    usage.category.name.clone(),
                    usage.category.icon.clone(),
                    usage.total_seconds,
                    usage.app_count,
                    usage.apps.clone(),
                    percentage,
                    color,
                    color_str,
                )
            })
            .collect();

        // 收集所有已分类的应用名称
        let mut classified_apps = HashSet::new();
        for usage in &self.category_usage {
            for app in &usage.apps {
                classified_apps.insert(app.app_name.as_str());
            }
        }

        // 找出未分类的应用
        let unclassified_apps: Vec<_> = self
            .all_apps
            .iter()
            .filter(|app| !classified_apps.contains(app.as_str()))
            .cloned()
            .collect();

        ScrollArea::vertical()
            .id_source("category_list_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = self.theme.spacing;

                for (
                    cat_id,
                    cat_name,
                    cat_icon,
                    total_secs,
                    app_count,
                    apps,
                    percentage,
                    color,
                    color_str,
                ) in &category_data
                {
                    self.show_category_card_data(
                        ui,
                        *cat_id,
                        cat_name,
                        cat_icon,
                        *total_secs,
                        *app_count,
                        apps,
                        *percentage,
                        *color,
                        color_str.clone(),
                    );
                }

                // 未分类应用区域
                if !unclassified_apps.is_empty() {
                    ui.add_space(self.theme.spacing);

                    egui::Frame::none()
                        .fill(self.theme.card_background)
                        .rounding(Rounding::same(self.theme.card_rounding))
                        .stroke(Stroke::new(1.0, self.theme.divider_color))
                        .inner_margin(self.theme.spacing)
                        .show(ui, |ui| {
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new("📥").size(self.theme.heading_size),
                                    );
                                    ui.label(
                                        egui::RichText::new("未分类应用")
                                            .size(self.theme.heading_size)
                                            .color(self.theme.text_color),
                                    );
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "({} 个)",
                                            unclassified_apps.len()
                                        ))
                                        .size(self.theme.body_size)
                                        .color(self.theme.secondary_text_color),
                                    );
                                });

                                ui.add_space(self.theme.spacing / 2.0);

                                ScrollArea::vertical()
                                    .id_source("unclassified_apps")
                                    .auto_shrink([false; 2])
                                    .max_height(200.0)
                                    .show(ui, |ui| {
                                        for app_name in &unclassified_apps {
                                            ui.horizontal(|ui| {
                                                ui.label(
                                                    egui::RichText::new(app_name)
                                                        .size(self.theme.body_size)
                                                        .color(self.theme.text_color),
                                                );
                                                ui.with_layout(
                                                    egui::Layout::right_to_left(
                                                        egui::Align::Center,
                                                    ),
                                                    |ui| {
                                                        if ui.small_button("归类").clicked() {
                                                            self.selected_app_name =
                                                                Some(app_name.clone());
                                                            self.selected_category_ids.clear();
                                                            self.show_assign_dialog = true;
                                                            // 触发加载应用分类操作
                                                            self.pending_action = Some(
                                                                CategoryAction::LoadAppCategories(
                                                                    app_name.clone(),
                                                                ),
                                                            );
                                                        }
                                                    },
                                                );
                                            });
                                        }
                                    });
                            });
                        });
                }
            });
    }

    /// 显示单个分类卡片（使用预提取的数据）
    #[allow(clippy::too_many_arguments)]
    fn show_category_card_data(
        &mut self,
        ui: &mut Ui,
        cat_id: Option<i64>,
        cat_name: &str,
        cat_icon: &str,
        total_secs: i64,
        app_count: usize,
        apps: &[AppUsageInCategory],
        percentage: f32,
        color: Color32,
        color_str: Option<String>,
    ) {
        egui::Frame::none()
            .fill(self.theme.card_background)
            .rounding(Rounding::same(self.theme.card_rounding))
            .stroke(Stroke::new(1.0, self.theme.divider_color))
            .inner_margin(self.theme.spacing)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    // 分类标题行
                    ui.horizontal(|ui| {
                        // 图标和名称
                        ui.label(egui::RichText::new(cat_icon).size(self.theme.heading_size));
                        ui.label(
                            egui::RichText::new(cat_name)
                                .size(self.theme.heading_size)
                                .color(self.theme.text_color),
                        );

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            // 操作按钮
                            if ui.small_button("[编辑]").clicked() {
                                self.selected_category_id = cat_id;
                                self.new_category_name = cat_name.to_string();
                                self.new_category_icon = cat_icon.to_string();
                                self.new_category_color = color_str.or_else(|| {
                                    Some(format!(
                                        "#{:02X}{:02X}{:02X}",
                                        color.r(),
                                        color.g(),
                                        color.b()
                                    ))
                                });
                                self.show_edit_dialog = true;
                            }

                            if ui.small_button("[删除]").clicked()
                                && let Some(id) = cat_id
                            {
                                self.pending_action = Some(CategoryAction::DeleteCategory(id));
                                self.needs_refresh = true;
                            }

                            ui.add_space(self.theme.spacing);

                            // 时间和百分比
                            ui.label(
                                egui::RichText::new(format!("{:.1}%", percentage))
                                    .size(self.theme.body_size)
                                    .color(self.theme.secondary_text_color),
                            );
                            ui.label(
                                egui::RichText::new(duration::format_duration(total_secs))
                                    .size(self.theme.heading_size)
                                    .color(color),
                            );
                        });
                    });

                    ui.add_space(self.theme.spacing / 2.0);

                    // 柱形图（使用分类颜色）
                    self.show_bar_chart(ui, percentage, color);

                    ui.add_space(self.theme.spacing / 2.0);

                    // 应用列表
                    if !apps.is_empty() {
                        ui.label(
                            egui::RichText::new(format!("包含 {} 个应用:", app_count))
                                .size(self.theme.small_size)
                                .color(self.theme.secondary_text_color),
                        );

                        ui.add_space(self.theme.spacing / 4.0);

                        for app in apps {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(&app.app_name)
                                        .size(self.theme.body_size)
                                        .color(self.theme.text_color),
                                );
                                ui.label(
                                    egui::RichText::new(duration::format_duration(
                                        app.total_seconds,
                                    ))
                                    .size(self.theme.small_size)
                                    .color(self.theme.secondary_text_color),
                                );
                                // 从分类中移除应用的按钮
                                if let Some(id) = cat_id
                                    && ui
                                        .small_button("✕")
                                        .on_hover_text("从此分类中移除")
                                        .clicked()
                                {
                                    self.pending_action = Some(
                                        CategoryAction::RemoveAppFromCategory(
                                            app.app_name.clone(),
                                            id,
                                        ),
                                    );
                                    self.needs_refresh = true;
                                }
                            });
                        }
                    }
                });
            });
    }

    /// 显示柱形图
    fn show_bar_chart(&self, ui: &mut Ui, percentage: f32, color: Color32) {
        let height = 20.0;
        let (rect, _response) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), height),
            egui::Sense::hover(),
        );

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // 背景
            painter.rect_filled(
                rect,
                Rounding::same(self.theme.card_rounding / 2.0),
                self.theme.progress_background,
            );

            // 进度条
            let bar_width = rect.width() * (percentage / 100.0);
            let bar_rect = egui::Rect::from_min_size(rect.min, Vec2::new(bar_width, height));

            painter.rect_filled(
                bar_rect,
                Rounding::same(self.theme.card_rounding / 2.0),
                color,
            );
        }
    }

    /// 解析颜色字符串为 Color32
    fn parse_color(hex: &str) -> Option<Color32> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return None;
        }
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some(Color32::from_rgb(r, g, b))
    }

    /// 显示对话框
    fn show_dialogs(&mut self, ui: &mut Ui) {
        // 添加分类对话框
        if self.show_add_dialog {
            self.show_add_category_dialog(ui);
        }

        // 编辑分类对话框
        if self.show_edit_dialog {
            self.show_edit_category_dialog(ui);
        }

        // 应用归类对话框
        if self.show_assign_dialog {
            self.show_assign_apps_dialog(ui);
        }
    }

    /// 显示添加分类对话框
    fn show_add_category_dialog(&mut self, ui: &mut Ui) {
        egui::Window::new("添加分类")
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.vertical(|ui| {
                    ui.label("分类名称:");
                    ui.text_edit_singleline(&mut self.new_category_name);

                    ui.add_space(self.theme.spacing / 2.0);

                    ui.label("选择图标:");
                    ui.horizontal_wrapped(|ui| {
                        ui.label(&self.new_category_icon);
                        if ui.button("选择...").clicked() {
                            self.show_icon_picker = !self.show_icon_picker;
                        }
                    });

                    if self.show_icon_picker {
                        ui.add_space(self.theme.spacing / 4.0);
                        ScrollArea::vertical()
                            .id_source("add_category_icon_picker")
                            .max_height(200.0)
                            .show(ui, |ui| {
                                ui.horizontal_wrapped(|ui| {
                                    for icon in CATEGORY_ICONS {
                                        if ui.button(*icon).clicked() {
                                            self.new_category_icon = icon.to_string();
                                            self.show_icon_picker = false;
                                        }
                                    }
                                });
                            });
                    }

                    ui.add_space(self.theme.spacing / 2.0);

                    // 颜色选择器
                    ui.label("选择颜色:");
                    ui.horizontal_wrapped(|ui| {
                        for (_name, color) in CATEGORY_COLORS {
                            let is_selected = self
                                .new_category_color
                                .as_ref()
                                .and_then(|c| Self::parse_color(c))
                                .map(|c| c == *color)
                                .unwrap_or(false);

                            let (rect, response) =
                                ui.allocate_exact_size(Vec2::splat(24.0), egui::Sense::click());
                            let painter = ui.painter();

                            painter.rect_filled(rect, egui::Rounding::same(4.0), *color);

                            if is_selected {
                                painter.rect_stroke(
                                    rect,
                                    egui::Rounding::same(4.0),
                                    egui::Stroke::new(2.0, self.theme.text_color),
                                );
                            }

                            if response.clicked() {
                                self.new_category_color = Some(format!(
                                    "#{:02X}{:02X}{:02X}",
                                    color.r(),
                                    color.g(),
                                    color.b()
                                ));
                            }
                        }
                    });

                    ui.add_space(self.theme.spacing);

                    ui.horizontal(|ui| {
                        if ui.button("确定").clicked() && !self.new_category_name.is_empty() {
                            let category = Category {
                                id: None,
                                name: self.new_category_name.clone(),
                                icon: self.new_category_icon.clone(),
                                color: self.new_category_color.clone(),
                            };
                            self.pending_action = Some(CategoryAction::AddCategory(category));
                            self.show_add_dialog = false;
                            self.show_icon_picker = false;
                            self.needs_refresh = true;
                        }

                        if ui.button("取消").clicked() {
                            self.show_add_dialog = false;
                            self.show_icon_picker = false;
                        }
                    });
                });
            });
    }

    /// 显示编辑分类对话框
    fn show_edit_category_dialog(&mut self, ui: &mut Ui) {
        egui::Window::new("编辑分类")
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.vertical(|ui| {
                    ui.label("分类名称:");
                    ui.text_edit_singleline(&mut self.new_category_name);

                    ui.add_space(self.theme.spacing / 2.0);

                    ui.label("选择图标:");
                    ui.horizontal_wrapped(|ui| {
                        ui.label(&self.new_category_icon);
                        if ui.button("选择...").clicked() {
                            self.show_icon_picker = !self.show_icon_picker;
                        }
                    });

                    if self.show_icon_picker {
                        ui.add_space(self.theme.spacing / 4.0);
                        ScrollArea::vertical()
                            .id_source("edit_category_icon_picker")
                            .max_height(200.0)
                            .show(ui, |ui| {
                                ui.horizontal_wrapped(|ui| {
                                    for icon in CATEGORY_ICONS {
                                        if ui.button(*icon).clicked() {
                                            self.new_category_icon = icon.to_string();
                                            self.show_icon_picker = false;
                                        }
                                    }
                                });
                            });
                    }

                    ui.add_space(self.theme.spacing / 2.0);

                    // 颜色选择器
                    ui.label("选择颜色:");
                    ui.horizontal_wrapped(|ui| {
                        for (_name, color) in CATEGORY_COLORS {
                            let is_selected = self
                                .new_category_color
                                .as_ref()
                                .and_then(|c| Self::parse_color(c))
                                .map(|c| c == *color)
                                .unwrap_or(false);

                            let (rect, response) =
                                ui.allocate_exact_size(Vec2::splat(24.0), egui::Sense::click());
                            let painter = ui.painter();

                            painter.rect_filled(rect, egui::Rounding::same(4.0), *color);

                            if is_selected {
                                painter.rect_stroke(
                                    rect,
                                    egui::Rounding::same(4.0),
                                    egui::Stroke::new(2.0, self.theme.text_color),
                                );
                            }

                            if response.clicked() {
                                self.new_category_color = Some(format!(
                                    "#{:02X}{:02X}{:02X}",
                                    color.r(),
                                    color.g(),
                                    color.b()
                                ));
                            }
                        }
                    });

                    ui.add_space(self.theme.spacing);

                    ui.horizontal(|ui| {
                        if ui.button("保存").clicked()
                            && let Some(id) = self.selected_category_id
                            && !self.new_category_name.is_empty()
                        {
                            let category = Category {
                                id: Some(id),
                                name: self.new_category_name.clone(),
                                icon: self.new_category_icon.clone(),
                                color: self.new_category_color.clone(),
                            };
                            self.pending_action = Some(CategoryAction::UpdateCategory(category));
                            self.show_edit_dialog = false;
                            self.show_icon_picker = false;
                            self.needs_refresh = true;
                        }

                        if ui.button("取消").clicked() {
                            self.show_edit_dialog = false;
                            self.show_icon_picker = false;
                        }
                    });
                });
            });
    }

    /// 显示应用归类对话框
    fn show_assign_apps_dialog(&mut self, ui: &mut Ui) {
        egui::Window::new("管理应用分类")
            .collapsible(false)
            .resizable(true)
            .default_width(400.0)
            .show(ui.ctx(), |ui| {
                ui.vertical(|ui| {
                    ui.label("选择应用:");

                    // 克隆 all_apps 以避免借用冲突
                    let all_apps = self.all_apps.clone();

                    ScrollArea::vertical()
                        .id_source("assign_apps_list")
                        .max_height(300.0)
                        .show(ui, |ui| {
                            for app_name in &all_apps {
                                if ui
                                    .selectable_label(
                                        self.selected_app_name.as_ref() == Some(app_name),
                                        app_name,
                                    )
                                    .clicked()
                                {
                                    // 选择新应用时，触发加载该应用当前的分类
                                    self.selected_app_name = Some(app_name.clone());
                                    self.pending_action = Some(
                                        CategoryAction::LoadAppCategories(app_name.clone()),
                                    );
                                }
                            }
                        });

                    ui.add_space(self.theme.spacing);

                    if let Some(ref app_name) = self.selected_app_name.clone() {
                        ui.label(format!("为 '{}' 选择分类:", app_name));

                        ui.add_space(self.theme.spacing / 2.0);

                        // 克隆 categories 以避免借用冲突
                        let categories = self.categories.clone();

                        ScrollArea::vertical()
                            .id_source("assign_category_list")
                            .max_height(200.0)
                            .show(ui, |ui| {
                                for category in &categories {
                                    if let Some(cat_id) = category.id {
                                        let mut is_selected =
                                            self.selected_category_ids.contains(&cat_id);

                                        if ui
                                            .checkbox(
                                                &mut is_selected,
                                                format!("{} {}", category.icon, category.name),
                                            )
                                            .changed()
                                        {
                                            if is_selected {
                                                if !self.selected_category_ids.contains(&cat_id) {
                                                    self.selected_category_ids.push(cat_id);
                                                }
                                            } else {
                                                self.selected_category_ids
                                                    .retain(|&id| id != cat_id);
                                            }
                                        }
                                    }
                                }
                            });

                        ui.add_space(self.theme.spacing);

                        if ui.button("保存").clicked() {
                            tracing::info!(
                                "保存应用分类: app={}, categories={:?}",
                                app_name,
                                self.selected_category_ids
                            );
                            self.pending_action = Some(CategoryAction::SetAppCategories(
                                app_name.clone(),
                                self.selected_category_ids.clone(),
                            ));
                            self.needs_refresh = true;
                            self.show_assign_dialog = false;
                            self.selected_app_name = None;
                            self.selected_category_ids.clear();
                        }
                    }

                    ui.add_space(self.theme.spacing);

                    if ui.button("取消").clicked() {
                        self.show_assign_dialog = false;
                        self.selected_app_name = None;
                        self.selected_category_ids.clear();
                    }
                });
            });
    }
}
