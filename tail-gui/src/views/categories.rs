//! TaiL GUI - 分类视图

use egui::{ScrollArea, Ui, Stroke, Rounding, Vec2};
use tail_core::{CategoryUsage, Category, Repository, CATEGORY_ICONS};
use chrono::{DateTime, Utc};

use crate::components::{PageHeader, StatCard, EmptyState, SectionDivider};
use crate::theme::TaiLTheme;
use crate::icons::ui_icons::categories as icons;

/// 分类视图状态
pub struct CategoriesView {
    /// 分类使用统计数据
    category_usage: Vec<CategoryUsage>,
    /// 所有分类列表
    categories: Vec<Category>,
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
}

impl CategoriesView {
    pub fn new(theme: TaiLTheme) -> Self {
        Self {
            category_usage: Vec::new(),
            categories: Vec::new(),
            theme,
            show_add_dialog: false,
            show_edit_dialog: false,
            show_assign_dialog: false,
            new_category_name: String::new(),
            new_category_icon: "🗀".to_string(),
            selected_category_id: None,
            selected_app_name: None,
            selected_category_ids: Vec::new(),
            all_apps: Vec::new(),
            show_icon_picker: false,
            needs_refresh: false,
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

    /// 加载分类数据
    pub fn load_data(&mut self, repo: &Repository, start: DateTime<Utc>, end: DateTime<Utc>) {
        // 加载分类使用统计
        if let Ok(usage) = repo.get_category_usage(start, end) {
            self.category_usage = usage;
        }

        // 加载所有分类
        if let Ok(cats) = repo.get_categories() {
            self.categories = cats;
        }

        // 加载所有应用名称
        if let Ok(apps) = repo.get_all_app_names() {
            self.all_apps = apps;
        }
    }

    /// 渲染分类视图
    pub fn show(&mut self, ui: &mut Ui, repo: &Repository) {
        // 页面标题
        ui.add(PageHeader::new("应用分类", icons::PAGE_ICON, &self.theme)
            .subtitle("按分类查看应用使用时间"));
        
        ui.add_space(self.theme.spacing);

        // 工具栏
        self.show_toolbar(ui);
        
        ui.add_space(self.theme.spacing);

        // 统计卡片
        self.show_stat_cards(ui);
        
        ui.add_space(self.theme.spacing);

        // 分隔线
        ui.add(SectionDivider::new(&self.theme).with_title("分类统计"));
        
        ui.add_space(self.theme.spacing / 2.0);

        // 分类列表和柱形图
        self.show_category_list(ui, repo);

        // 对话框
        self.show_dialogs(ui, repo);
    }

    /// 显示工具栏
    fn show_toolbar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui.button("+ 添加分类").clicked() {
                self.show_add_dialog = true;
                self.new_category_name.clear();
                self.new_category_icon = "🗀".to_string();
            }

            ui.add_space(self.theme.spacing / 2.0);

            if ui.button("# 管理应用分类").clicked() {
                self.show_assign_dialog = true;
            }
        });
    }

    /// 显示统计卡片
    fn show_stat_cards(&self, ui: &mut Ui) {
        let total_seconds: i64 = self.category_usage.iter()
            .map(|c| c.total_seconds)
            .sum();
        
        let category_count = self.categories.len();
        let categorized_apps: usize = self.category_usage.iter()
            .map(|c| c.app_count)
            .sum();

        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = self.theme.spacing;
            
            // 总分类数
            ui.add(StatCard::new(
                "分类总数",
                &format!("{} 个", category_count),
                icons::CATEGORY_COUNT,
                &self.theme,
            ).accent_color(self.theme.primary_color));

            // 已分类应用数
            ui.add(StatCard::new(
                "已分类应用",
                &format!("{} 个", categorized_apps),
                icons::APP_COUNT,
                &self.theme,
            ).accent_color(self.theme.accent_color));

            // 总使用时间
            ui.add(StatCard::new(
                "总使用时间",
                &Self::format_duration(total_seconds),
                icons::TOTAL_TIME,
                &self.theme,
            ).accent_color(self.theme.success_color));

            // 最常用分类
            if let Some(top_category) = self.category_usage.first() {
                ui.add(StatCard::new(
                    "最常用分类",
                    &top_category.category.name,
                    &top_category.category.icon,
                    &self.theme,
                ).subtitle(&Self::format_duration(top_category.total_seconds))
                 .accent_color(self.theme.warning_color));
            }
        });
    }

    /// 显示分类列表
    fn show_category_list(&mut self, ui: &mut Ui, repo: &Repository) {
        if self.category_usage.is_empty() {
            ui.add(EmptyState::new(
                icons::EMPTY_STATE,
                "暂无分类数据",
                "创建分类并为应用分配分类后，这里会显示统计信息",
                &self.theme,
            ));
            return;
        }

        let total_seconds: i64 = self.category_usage.iter()
            .map(|c| c.total_seconds)
            .sum();

        // 收集需要的数据，避免借用冲突
        let category_data: Vec<_> = self.category_usage.iter()
            .map(|usage| {
                let percentage = if total_seconds > 0 {
                    (usage.total_seconds as f32 / total_seconds as f32) * 100.0
                } else {
                    0.0
                };
                (
                    usage.category.id,
                    usage.category.name.clone(),
                    usage.category.icon.clone(),
                    usage.total_seconds,
                    usage.app_count,
                    usage.apps.clone(),
                    percentage,
                )
            })
            .collect();

        ScrollArea::vertical()
            .id_source("category_list_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = self.theme.spacing;
                
                for (cat_id, cat_name, cat_icon, total_secs, app_count, apps, percentage) in &category_data {
                    self.show_category_card_data(ui, *cat_id, cat_name, cat_icon, *total_secs, *app_count, apps, *percentage, repo);
                }
            });
    }

    /// 显示单个分类卡片（使用预提取的数据）
    fn show_category_card_data(
        &mut self,
        ui: &mut Ui,
        cat_id: Option<i64>,
        cat_name: &str,
        cat_icon: &str,
        total_secs: i64,
        app_count: usize,
        apps: &[tail_core::AppUsageInCategory],
        percentage: f32,
        repo: &Repository,
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
                        ui.label(
                            egui::RichText::new(cat_icon)
                                .size(self.theme.heading_size)
                        );
                        ui.label(
                            egui::RichText::new(cat_name)
                                .size(self.theme.heading_size)
                                .color(self.theme.text_color)
                        );

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            // 操作按钮
                            if ui.small_button("[编辑]").clicked() {
                                self.selected_category_id = cat_id;
                                self.new_category_name = cat_name.to_string();
                                self.new_category_icon = cat_icon.to_string();
                                self.show_edit_dialog = true;
                            }

                            if ui.small_button("[删除]").clicked() {
                                if let Some(id) = cat_id {
                                    let _ = repo.delete_category(id);
                                    self.needs_refresh = true; // 标记需要刷新
                                }
                            }

                            ui.add_space(self.theme.spacing);

                            // 时间和百分比
                            ui.label(
                                egui::RichText::new(format!("{:.1}%", percentage))
                                    .size(self.theme.body_size)
                                    .color(self.theme.secondary_text_color)
                            );
                            ui.label(
                                egui::RichText::new(Self::format_duration(total_secs))
                                    .size(self.theme.heading_size)
                                    .color(self.theme.primary_color)
                            );
                        });
                    });

                    ui.add_space(self.theme.spacing / 2.0);

                    // 柱形图
                    self.show_bar_chart(ui, percentage);

                    ui.add_space(self.theme.spacing / 2.0);

                    // 应用列表
                    if !apps.is_empty() {
                        ui.label(
                            egui::RichText::new(format!("包含 {} 个应用:", app_count))
                                .size(self.theme.small_size)
                                .color(self.theme.secondary_text_color)
                        );
                        
                        ui.add_space(self.theme.spacing / 4.0);

                        for app in apps {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(&app.app_name)
                                        .size(self.theme.body_size)
                                        .color(self.theme.text_color)
                                );
                                ui.label(
                                    egui::RichText::new(Self::format_duration(app.total_seconds))
                                        .size(self.theme.small_size)
                                        .color(self.theme.secondary_text_color)
                                );
                                // 从分类中移除应用的按钮
                                if let Some(id) = cat_id {
                                    if ui.small_button("✕").on_hover_text("从此分类中移除").clicked() {
                                        let _ = repo.remove_app_from_category(&app.app_name, id);
                                        self.needs_refresh = true;
                                    }
                                }
                            });
                        }
                    }
                });
            });
    }

    /// 显示柱形图
    fn show_bar_chart(&self, ui: &mut Ui, percentage: f32) {
        let height = 20.0;
        let (rect, _response) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), height),
            egui::Sense::hover()
        );

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            
            // 背景
            painter.rect_filled(
                rect,
                Rounding::same(self.theme.card_rounding / 2.0),
                self.theme.progress_background
            );

            // 进度条
            let bar_width = rect.width() * (percentage / 100.0);
            let bar_rect = egui::Rect::from_min_size(
                rect.min,
                Vec2::new(bar_width, height)
            );
            
            painter.rect_filled(
                bar_rect,
                Rounding::same(self.theme.card_rounding / 2.0),
                self.theme.primary_color
            );
        }
    }

    /// 显示对话框
    fn show_dialogs(&mut self, ui: &mut Ui, repo: &Repository) {
        // 添加分类对话框
        if self.show_add_dialog {
            self.show_add_category_dialog(ui, repo);
        }

        // 编辑分类对话框
        if self.show_edit_dialog {
            self.show_edit_category_dialog(ui, repo);
        }

        // 应用归类对话框
        if self.show_assign_dialog {
            self.show_assign_apps_dialog(ui, repo);
        }
    }

    /// 显示添加分类对话框
    fn show_add_category_dialog(&mut self, ui: &mut Ui, repo: &Repository) {
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

                    ui.add_space(self.theme.spacing);

                    ui.horizontal(|ui| {
                        if ui.button("确定").clicked() {
                            if !self.new_category_name.is_empty() {
                                let category = Category {
                                    id: None,
                                    name: self.new_category_name.clone(),
                                    icon: self.new_category_icon.clone(),
                                    color: None,
                                };
                                let _ = repo.insert_category(&category);
                                self.show_add_dialog = false;
                                self.show_icon_picker = false;
                                self.needs_refresh = true; // 标记需要刷新
                            }
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
    fn show_edit_category_dialog(&mut self, ui: &mut Ui, repo: &Repository) {
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

                    ui.add_space(self.theme.spacing);

                    ui.horizontal(|ui| {
                        if ui.button("保存").clicked() {
                            if let Some(id) = self.selected_category_id {
                                if !self.new_category_name.is_empty() {
                                    let category = Category {
                                        id: Some(id),
                                        name: self.new_category_name.clone(),
                                        icon: self.new_category_icon.clone(),
                                        color: None,
                                    };
                                    let _ = repo.update_category(&category);
                                    self.show_edit_dialog = false;
                                    self.show_icon_picker = false;
                                    self.needs_refresh = true; // 标记需要刷新
                                }
                            }
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
    fn show_assign_apps_dialog(&mut self, ui: &mut Ui, repo: &Repository) {
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
                                if ui.selectable_label(
                                    self.selected_app_name.as_ref() == Some(app_name),
                                    app_name
                                ).clicked() {
                                    // 选择新应用时，加载该应用当前的分类
                                    self.selected_app_name = Some(app_name.clone());
                                    let current_categories = repo.get_app_categories(app_name).unwrap_or_default();
                                    self.selected_category_ids = current_categories.iter()
                                        .filter_map(|c| c.id)
                                        .collect();
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
                                        let mut is_selected = self.selected_category_ids.contains(&cat_id);
                                        
                                        if ui.checkbox(&mut is_selected, format!("{} {}", category.icon, category.name)).changed() {
                                            if is_selected {
                                                if !self.selected_category_ids.contains(&cat_id) {
                                                    self.selected_category_ids.push(cat_id);
                                                }
                                            } else {
                                                self.selected_category_ids.retain(|&id| id != cat_id);
                                            }
                                        }
                                    }
                                }
                            });

                        ui.add_space(self.theme.spacing);

                        if ui.button("保存").clicked() {
                            tracing::info!("保存应用分类: app={}, categories={:?}", app_name, self.selected_category_ids);
                            match repo.set_app_categories(&app_name, &self.selected_category_ids) {
                                Ok(_) => {
                                    tracing::info!("保存成功");
                                    self.needs_refresh = true; // 标记需要刷新
                                }
                                Err(e) => tracing::error!("保存失败: {:?}", e),
                            }
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

    /// 格式化时长
    fn format_duration(seconds: i64) -> String {
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;

        if hours > 0 {
            format!("{}h {}m", hours, minutes)
        } else if minutes > 0 {
            format!("{}m", minutes)
        } else {
            format!("{}s", seconds)
        }
    }
}
