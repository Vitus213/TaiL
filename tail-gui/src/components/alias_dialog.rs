//! 别名管理对话框

use egui;

use crate::theme::TaiLTheme;

/// 别名管理对话框状态
#[derive(Default)]
pub struct AliasDialog {
    /// 是否显示对话框
    pub is_open: bool,
    /// 当前应用名称
    pub app_name: String,
    /// 当前别名
    pub alias: String,
    /// 所有应用别名列表
    pub aliases: Vec<(String, String)>,
    /// 编辑模式（true=编辑现有，false=添加新）
    pub is_edit_mode: bool,
    /// 是否正在加载
    pub loading: bool,
}

impl AliasDialog {
    /// 打开对话框以设置新别名
    pub fn open_for_app(&mut self, app_name: String, current_alias: Option<String>) {
        let has_alias = current_alias.is_some();
        self.is_open = true;
        self.app_name = app_name;
        self.alias = current_alias.unwrap_or_default();
        self.is_edit_mode = has_alias;
    }

    /// 打开对话框以管理所有别名
    pub fn open_for_management(&mut self, aliases: Vec<(String, String)>) {
        self.is_open = true;
        self.aliases = aliases;
        self.is_edit_mode = false;
    }

    /// 关闭对话框
    pub fn close(&mut self) {
        self.is_open = false;
        self.app_name.clear();
        self.alias.clear();
        self.aliases.clear();
        self.is_edit_mode = false;
    }

    /// 显示对话框，返回需要保存的别名 (Some((app_name, alias))) 或 None
    pub fn show(&mut self, ctx: &egui::Context, theme: &TaiLTheme) -> Option<(String, String)> {
        if !self.is_open {
            return None;
        }

        let mut result = None;

        egui::Window::new(if self.is_edit_mode {
            "编辑别名"
        } else {
            "设置别名"
        })
        .collapsible(false)
        .resizable(false)
        .default_width(400.0)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                if !self.aliases.is_empty() && self.app_name.is_empty() {
                    // 管理所有别名模式
                    ui.label(
                        egui::RichText::new("所有应用别名")
                            .size(theme.heading_size)
                            .color(theme.text_color),
                    );
                    ui.add_space(theme.spacing);

                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .max_height(300.0)
                        .show(ui, |ui| {
                            for (app_name, alias) in &self.aliases {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new(app_name)
                                            .size(theme.body_size)
                                            .color(theme.text_color),
                                    );
                                    ui.label("→");
                                    ui.label(
                                        egui::RichText::new(alias)
                                            .size(theme.body_size)
                                            .color(theme.primary_color)
                                            .strong(),
                                    );
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if ui.small_button("删除").clicked() {
                                                result = Some((app_name.clone(), String::new()));
                                            }
                                        },
                                    );
                                });
                            }
                        });

                    ui.add_space(theme.spacing);
                    ui.horizontal(|ui| {
                        if ui.button("关闭").clicked() {
                            self.close();
                        }
                    });
                } else {
                    // 单个别名设置模式
                    ui.label(
                        egui::RichText::new("应用名称")
                            .size(theme.small_size)
                            .color(theme.secondary_text_color),
                    );
                    ui.label(
                        egui::RichText::new(&self.app_name)
                            .size(theme.body_size)
                            .color(theme.text_color),
                    );

                    ui.add_space(theme.spacing / 2.0);

                    ui.label(
                        egui::RichText::new("别名")
                            .size(theme.small_size)
                            .color(theme.secondary_text_color),
                    );
                    ui.text_edit_singleline(&mut self.alias);

                    ui.add_space(4.0);

                    // 别名字符计数
                    let max_length = 15;
                    let remaining = max_length - self.alias.chars().count();
                    let color = if remaining < 3 {
                        theme.warning_color
                    } else {
                        theme.secondary_text_color
                    };
                    ui.horizontal(|ui| {
                        if self.alias.chars().count() > max_length {
                            ui.label(
                                egui::RichText::new(format!(
                                    "超出 {} 个字符",
                                    self.alias.chars().count() - max_length
                                ))
                                .size(theme.small_size)
                                .color(theme.warning_color),
                            );
                        } else {
                            ui.label(
                                egui::RichText::new(format!("{} / {}", remaining, max_length))
                                    .size(theme.small_size)
                                    .color(color),
                            );
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if !self.alias.is_empty()
                                && self.alias.chars().count() <= max_length
                                && ui
                                    .button(if self.is_edit_mode {
                                        "保存"
                                    } else {
                                        "设置"
                                    })
                                    .clicked()
                            {
                                result = Some((self.app_name.clone(), self.alias.clone()));
                            }
                            if ui.button("取消").clicked() {
                                self.close();
                            }
                        });
                    });

                    ui.add_space(theme.spacing / 2.0);

                    // 提示信息
                    ui.label(
                        egui::RichText::new("💡 别名将替代应用名显示在统计中")
                            .size(theme.small_size)
                            .color(theme.secondary_text_color),
                    );
                }
            });
        });

        if result.is_some() {
            self.close();
        }

        result
    }
}
