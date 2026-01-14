//! TaiL GUI - 设置视图

use egui::{ScrollArea, Ui, Color32, Vec2, Rounding};
use tail_core::{DailyGoal, DbConfig};

use crate::components::{PageHeader, SectionDivider};
use crate::theme::{TaiLTheme, ThemeType};

/// 设置视图
pub struct SettingsView<'a> {
    /// 每日目标列表
    daily_goals: &'a [DailyGoal],
    /// 当前主题类型
    current_theme_type: ThemeType,
    /// 主题
    theme: &'a TaiLTheme,
}

/// 设置视图的操作
pub enum SettingsAction {
    /// 添加新目标
    AddGoal,
    /// 删除目标
    DeleteGoal(String),
    /// 切换主题
    ChangeTheme(ThemeType),
    /// 无操作
    None,
}

impl<'a> SettingsView<'a> {
    pub fn new(
        daily_goals: &'a [DailyGoal],
        current_theme_type: ThemeType,
        theme: &'a TaiLTheme,
    ) -> Self {
        Self {
            daily_goals,
            current_theme_type,
            theme,
        }
    }

    /// 渲染设置视图
    pub fn show(&self, ui: &mut Ui) -> SettingsAction {
        let mut action = SettingsAction::None;

        // 页面标题
        ui.add(PageHeader::new("设置", "⚙", self.theme)
            .subtitle("自定义您的 TaiL 体验"));
        
        ui.add_space(self.theme.spacing);

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                // 主题设置
                ui.add(SectionDivider::new(self.theme).with_title("外观"));
                ui.add_space(self.theme.spacing / 2.0);
                
                if let Some(new_theme) = self.show_theme_settings(ui) {
                    action = SettingsAction::ChangeTheme(new_theme);
                }

                ui.add_space(self.theme.spacing);

                // 每日目标设置
                ui.add(SectionDivider::new(self.theme).with_title("每日目标"));
                ui.add_space(self.theme.spacing / 2.0);
                
                if let Some(goal_action) = self.show_goal_settings(ui) {
                    action = goal_action;
                }

                ui.add_space(self.theme.spacing);

                // 数据设置
                ui.add(SectionDivider::new(self.theme).with_title("数据"));
                ui.add_space(self.theme.spacing / 2.0);
                self.show_data_settings(ui);

                ui.add_space(self.theme.spacing);

                // 关于
                ui.add(SectionDivider::new(self.theme).with_title("关于"));
                ui.add_space(self.theme.spacing / 2.0);
                self.show_about(ui);
            });

        action
    }

    /// 显示主题设置
    fn show_theme_settings(&self, ui: &mut Ui) -> Option<ThemeType> {
        let mut new_theme = None;

        // 主题卡片容器
        let card_width = ui.available_width();
        
        ui.allocate_ui_with_layout(
            Vec2::new(card_width, 80.0),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                // 绘制卡片背景
                let painter = ui.painter();
                let rect = ui.available_rect_before_wrap();
                painter.rect_filled(
                    rect,
                    Rounding::same(self.theme.card_rounding),
                    self.theme.card_background,
                );

                ui.add_space(self.theme.card_padding);

                ui.vertical(|ui| {
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("主题")
                        .size(self.theme.body_size)
                        .color(self.theme.text_color));
                    
                    ui.add_space(8.0);
                    
                    ui.horizontal(|ui| {
                        for theme_type in ThemeType::all() {
                            let is_selected = *theme_type == self.current_theme_type;
                            
                            let button = egui::Button::new(
                                egui::RichText::new(theme_type.name())
                                    .size(self.theme.small_size)
                            )
                            .fill(if is_selected {
                                self.theme.primary_color
                            } else {
                                self.theme.card_hover_background
                            })
                            .rounding(Rounding::same(6.0));

                            if ui.add(button).clicked() && !is_selected {
                                new_theme = Some(*theme_type);
                            }
                        }
                    });
                });
            },
        );

        new_theme
    }

    /// 显示目标设置
    fn show_goal_settings(&self, ui: &mut Ui) -> Option<SettingsAction> {
        let mut action = None;

        // 目标列表
        if self.daily_goals.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.label(egui::RichText::new("🎯")
                    .size(32.0)
                    .color(self.theme.secondary_text_color.linear_multiply(0.5)));
                ui.add_space(8.0);
                ui.label(egui::RichText::new("暂无每日目标")
                    .size(self.theme.body_size)
                    .color(self.theme.text_color));
                ui.label(egui::RichText::new("添加目标来追踪您的应用使用时间")
                    .size(self.theme.small_size)
                    .color(self.theme.secondary_text_color));
                ui.add_space(20.0);
            });
        } else {
            for goal in self.daily_goals {
                ui.horizontal(|ui| {
                    // 目标卡片
                    ui.allocate_ui_with_layout(
                        Vec2::new(ui.available_width() - 50.0, 60.0),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            let painter = ui.painter();
                            let rect = ui.available_rect_before_wrap();
                            painter.rect_filled(
                                rect,
                                Rounding::same(self.theme.card_rounding),
                                self.theme.card_background,
                            );

                            ui.add_space(self.theme.card_padding);

                            ui.vertical(|ui| {
                                ui.add_space(8.0);
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("🎯")
                                        .size(16.0));
                                    ui.label(egui::RichText::new(&goal.app_name)
                                        .size(self.theme.body_size)
                                        .color(self.theme.text_color));
                                });
                                ui.label(egui::RichText::new(format!("最多 {} 分钟/天", goal.max_minutes))
                                    .size(self.theme.small_size)
                                    .color(self.theme.secondary_text_color));
                            });
                        },
                    );

                    // 删除按钮
                    if ui.add(
                        egui::Button::new(egui::RichText::new("🗑").size(16.0))
                            .fill(Color32::TRANSPARENT)
                            .rounding(Rounding::same(4.0))
                    ).on_hover_text("删除目标").clicked() {
                        action = Some(SettingsAction::DeleteGoal(goal.app_name.clone()));
                    }
                });

                ui.add_space(8.0);
            }
        }

        ui.add_space(self.theme.spacing / 2.0);

        // 添加目标按钮
        if ui.add(
            egui::Button::new(
                egui::RichText::new("➕ 添加新目标")
                    .size(self.theme.body_size)
            )
            .fill(self.theme.primary_color)
            .rounding(Rounding::same(8.0))
            .min_size(Vec2::new(150.0, 36.0))
        ).clicked() {
            action = Some(SettingsAction::AddGoal);
        }

        action
    }

    /// 显示数据设置
    fn show_data_settings(&self, ui: &mut Ui) {
        let config = DbConfig::default();

        // 数据库位置卡片
        ui.allocate_ui_with_layout(
            Vec2::new(ui.available_width(), 80.0),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                let painter = ui.painter();
                let rect = ui.available_rect_before_wrap();
                painter.rect_filled(
                    rect,
                    Rounding::same(self.theme.card_rounding),
                    self.theme.card_background,
                );

                ui.add_space(self.theme.card_padding);

                ui.vertical(|ui| {
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("💾")
                            .size(16.0)
                            .family(egui::FontFamily::Proportional));
                        ui.label(egui::RichText::new("数据库位置")
                            .size(self.theme.body_size)
                            .color(self.theme.text_color));
                    });
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new(&config.path)
                        .size(self.theme.small_size)
                        .color(self.theme.secondary_text_color));
                });
            },
        );

        ui.add_space(self.theme.spacing / 2.0);

        // 数据操作按钮
        ui.horizontal(|ui| {
            if ui.add(
                egui::Button::new(
                    egui::RichText::new("导出数据")
                        .size(self.theme.small_size)
                )
                .rounding(Rounding::same(6.0))
            ).clicked() {
                // TODO: 实现数据导出
            }

            if ui.add(
                egui::Button::new(
                    egui::RichText::new("清除数据")
                        .size(self.theme.small_size)
                        .color(self.theme.danger_color)
                )
                .fill(Color32::TRANSPARENT)
                .rounding(Rounding::same(6.0))
            ).clicked() {
                // TODO: 实现数据清除（需要确认对话框）
            }
        });
    }

    /// 显示关于信息
    fn show_about(&self, ui: &mut Ui) {
        ui.allocate_ui_with_layout(
            Vec2::new(ui.available_width(), 120.0),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                let painter = ui.painter();
                let rect = ui.available_rect_before_wrap();
                painter.rect_filled(
                    rect,
                    Rounding::same(self.theme.card_rounding),
                    self.theme.card_background,
                );

                ui.add_space(self.theme.card_padding);

                ui.vertical(|ui| {
                    ui.add_space(12.0);
                    
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new("TaiL")
                                .size(self.theme.heading_size)
                                .color(self.theme.text_color));
                            ui.label(egui::RichText::new("时间追踪工具")
                                .size(self.theme.small_size)
                                .color(self.theme.secondary_text_color));
                        });
                    });

                    ui.add_space(12.0);

                    ui.label(egui::RichText::new("版本 0.1.0")
                        .size(self.theme.small_size)
                        .color(self.theme.secondary_text_color));
                    
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("专为 Linux/Wayland (Hyprland) 设计")
                            .size(self.theme.small_size)
                            .color(self.theme.secondary_text_color));
                    });

                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        if ui.link("GitHub").clicked() {
                            // TODO: 打开 GitHub 链接
                        }
                        ui.label(" | ");
                        if ui.link("文档").clicked() {
                            // TODO: 打开文档链接
                        }
                        ui.label(" | ");
                        if ui.link("反馈").clicked() {
                            // TODO: 打开反馈链接
                        }
                    });
                });
            },
        );
    }
}

/// 添加目标对话框
pub struct AddGoalDialog {
    /// 应用名称
    pub app_name: String,
    /// 最大分钟数
    pub max_minutes: i32,
    /// 是否显示
    pub visible: bool,
}

impl Default for AddGoalDialog {
    fn default() -> Self {
        Self {
            app_name: String::new(),
            max_minutes: 60,
            visible: false,
        }
    }
}

impl AddGoalDialog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self) {
        self.visible = true;
        self.app_name.clear();
        self.max_minutes = 60;
    }

    pub fn close(&mut self) {
        self.visible = false;
    }

    /// 显示对话框，返回是否确认添加
    pub fn show(&mut self, ctx: &egui::Context, theme: &TaiLTheme) -> Option<DailyGoal> {
        if !self.visible {
            return None;
        }

        let mut result = None;
        let mut should_close = false;

        egui::Window::new("添加每日目标")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.set_min_width(300.0);

                ui.add_space(8.0);

                ui.label(egui::RichText::new("应用名称")
                    .size(theme.small_size)
                    .color(theme.secondary_text_color));
                ui.add(
                    egui::TextEdit::singleline(&mut self.app_name)
                        .hint_text("例如: firefox, code")
                        .desired_width(f32::INFINITY)
                );

                ui.add_space(12.0);

                ui.label(egui::RichText::new("每日最大使用时间（分钟）")
                    .size(theme.small_size)
                    .color(theme.secondary_text_color));
                ui.add(egui::Slider::new(&mut self.max_minutes, 1..=480)
                    .suffix(" 分钟"));

                // 时间预览
                let hours = self.max_minutes / 60;
                let mins = self.max_minutes % 60;
                let time_str = if hours > 0 {
                    format!("= {} 小时 {} 分钟", hours, mins)
                } else {
                    format!("= {} 分钟", mins)
                };
                ui.label(egui::RichText::new(time_str)
                    .size(theme.small_size)
                    .color(theme.secondary_text_color));

                ui.add_space(16.0);

                ui.horizontal(|ui| {
                    if ui.add(
                        egui::Button::new("取消")
                            .fill(theme.card_hover_background)
                            .min_size(Vec2::new(80.0, 32.0))
                    ).clicked() {
                        should_close = true;
                    }

                    ui.add_space(8.0);

                    let can_add = !self.app_name.trim().is_empty();
                    if ui.add_enabled(
                        can_add,
                        egui::Button::new("确定")
                            .fill(theme.primary_color)
                            .min_size(Vec2::new(80.0, 32.0))
                    ).clicked() {
                        result = Some(DailyGoal {
                            id: None,
                            app_name: self.app_name.trim().to_string(),
                            max_minutes: self.max_minutes,
                            notify_enabled: true,
                        });
                        should_close = true;
                    }
                });
            });

        if should_close {
            self.close();
        }

        result
    }
}