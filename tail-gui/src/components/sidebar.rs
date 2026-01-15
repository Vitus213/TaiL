//! TaiL GUI - 侧边栏导航组件
//!
//! Tai 风格的侧边导航栏

use egui::{Color32, Context, Rect, Rounding, Sense, Ui, Vec2};

use super::navigation::{NavigationMode, View};
use crate::theme::TaiLTheme;

/// 侧边栏导航组件
pub struct SidebarNav<'a> {
    /// 当前选中的视图
    pub current_view: View,
    /// 主题
    pub theme: &'a TaiLTheme,
    /// 导航模式（可变引用，用于切换）
    pub nav_mode: &'a mut NavigationMode,
}

impl<'a> SidebarNav<'a> {
    /// 创建新的侧边栏
    pub fn new(current_view: View, theme: &'a TaiLTheme, nav_mode: &'a mut NavigationMode) -> Self {
        Self {
            current_view,
            theme,
            nav_mode,
        }
    }

    /// 显示侧边栏，返回是否需要切换视图
    pub fn show(&mut self, ctx: &Context) -> Option<View> {
        let mut new_view = None;

        egui::SidePanel::left("sidebar")
            .frame(
                egui::Frame::none()
                    .fill(self.theme.card_background)
                    .inner_margin(egui::Margin::ZERO),
            )
            .exact_width(80.0)
            .show(ctx, |ui| {
                ui.add_space(8.0);

                // Logo 区域
                ui.vertical_centered(|ui: &mut egui::Ui| {
                    ui.label(
                        egui::RichText::new("TaiL")
                            .size(self.theme.heading_size)
                            .color(self.theme.primary_color)
                            .strong(),
                    );
                });
                ui.add_space(8.0);

                // 分隔线
                ui.add_space(4.0);
                ui.add(egui::Separator::default().horizontal());
                ui.add_space(8.0);

                // 顶部导航项
                let top_views = [
                    View::Dashboard,
                    View::Statistics,
                    View::Details,
                    View::Categories,
                ];

                for view in top_views {
                    if self.show_nav_item(ui, view) {
                        new_view = Some(view);
                    }
                }

                ui.add_space(16.0);

                // 底部导航项（设置）
                ui.vertical_centered(|ui: &mut egui::Ui| {
                    // 分隔线
                    ui.add(egui::Separator::default().horizontal());
                    ui.add_space(8.0);

                    if self.show_nav_item(ui, View::Settings) {
                        new_view = Some(View::Settings);
                    }
                });
            });

        new_view
    }

    /// 显示单个导航项
    fn show_nav_item(&mut self, ui: &mut Ui, view: View) -> bool {
        let is_selected = self.current_view == view;

        let desired_size = Vec2::new(ui.available_width(), 48.0);
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // 背景
            let bg_color = if is_selected {
                self.theme.primary_color
            } else if response.hovered() {
                self.theme.card_hover_background
            } else {
                Color32::TRANSPARENT
            };

            if bg_color != Color32::TRANSPARENT {
                painter.rect_filled(rect, Rounding::same(8.0), bg_color);
            }

            // 图标和文字
            let icon = view.sidebar_icon();
            let label = view.label();
            let text_color = if is_selected {
                Color32::WHITE
            } else {
                self.theme.text_color
            };

            let text_str = format!("{} {}", icon, label);
            let font_size = self.theme.body_size;

            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                text_str,
                egui::FontId::proportional(font_size),
                text_color,
            );

            // 选中指示器（左侧小条）
            if is_selected {
                let indicator_rect = Rect::from_min_size(rect.min, Vec2::new(3.0, rect.height()));
                painter.rect_filled(indicator_rect, Rounding::same(0.0), Color32::WHITE);
            }
        }

        response.clicked()
    }
}
