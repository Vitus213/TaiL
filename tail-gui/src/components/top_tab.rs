//! TaiL GUI - 顶部 Tab 导航组件
//!
//! 传统风格的顶部 Tab 导航

use egui::{Color32, Context, Rounding, Vec2};

use super::navigation::{NavigationMode, View};
use crate::theme::TaiLTheme;

/// 顶部 Tab 导航组件
pub struct TopTabNav<'a> {
    /// 当前选中的视图
    pub current_view: View,
    /// 主题
    pub theme: &'a TaiLTheme,
    /// 导航模式（可变引用，用于切换）
    pub nav_mode: &'a mut NavigationMode,
}

impl<'a> TopTabNav<'a> {
    /// 创建新的顶部 Tab 导航
    pub fn new(current_view: View, theme: &'a TaiLTheme, nav_mode: &'a mut NavigationMode) -> Self {
        Self {
            current_view,
            theme,
            nav_mode,
        }
    }

    /// 显示顶部导航栏，返回是否需要切换视图
    pub fn show(&mut self, ctx: &Context) -> Option<View> {
        let mut new_view = None;

        egui::TopBottomPanel::top("top_nav_bar")
            .frame(
                egui::Frame::none()
                    .fill(self.theme.card_background)
                    .inner_margin(egui::Margin::symmetric(16.0, 8.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Logo
                    ui.label(
                        egui::RichText::new("TaiL")
                            .size(self.theme.heading_size)
                            .color(self.theme.text_color)
                            .strong(),
                    );

                    ui.add_space(24.0);

                    // 导航按钮
                    let nav_items = View::ALL;

                    for view in nav_items {
                        let is_selected = self.current_view == *view;
                        let label = view.label();
                        let icon = view.icon();

                        let button = egui::Button::new(
                            egui::RichText::new(format!("{} {}", icon, label))
                                .size(self.theme.body_size)
                                .color(if is_selected {
                                    Color32::WHITE
                                } else {
                                    self.theme.text_color
                                }),
                        )
                        .fill(if is_selected {
                            self.theme.primary_color
                        } else {
                            Color32::TRANSPARENT
                        })
                        .rounding(Rounding::same(8.0))
                        .min_size(Vec2::new(100.0, 32.0));

                        if ui.add(button).clicked() {
                            new_view = Some(*view);
                        }
                    }

                    // 右侧按钮
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // 退出按钮
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("✕")
                                        .size(16.0)
                                        .color(self.theme.secondary_text_color),
                                )
                                .fill(Color32::TRANSPARENT)
                                .rounding(Rounding::same(4.0)),
                            )
                            .on_hover_text("退出")
                            .clicked()
                        {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }

                        // 最小化按钮
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("─")
                                        .size(16.0)
                                        .color(self.theme.secondary_text_color),
                                )
                                .fill(Color32::TRANSPARENT)
                                .rounding(Rounding::same(4.0)),
                            )
                            .on_hover_text("最小化")
                            .clicked()
                        {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                        }
                    });
                });
            });

        new_view
    }
}
