//! TaiL GUI - 统一样式组件
//!
//! 提供卡片、按钮等常用 UI 组件的统一样式

use egui::{Color32, Painter, Rect, Rounding, Vec2};

use crate::theme::TaiLTheme;

/// 卡片样式工具
pub struct CardStyle<'a> {
    pub theme: &'a TaiLTheme,
}

impl<'a> CardStyle<'a> {
    pub fn new(theme: &'a TaiLTheme) -> Self {
        Self { theme }
    }

    /// 绘制卡片背景和阴影
    pub fn draw_card_background(
        &self,
        painter: &Painter,
        rect: Rect,
        hovered: bool,
        selected: bool,
    ) {
        // 阴影
        let shadow_rect = rect.translate(Vec2::new(2.0, 2.0));
        painter.rect_filled(
            shadow_rect,
            Rounding::same(self.theme.card_rounding),
            Color32::from_black_alpha(30),
        );

        // 背景
        let bg_color = if selected {
            self.theme.card_selected_background
        } else if hovered {
            self.theme.card_hover_background
        } else {
            self.theme.card_background
        };

        painter.rect_filled(rect, Rounding::same(self.theme.card_rounding), bg_color);
    }

    /// 绘制进度条
    pub fn draw_progress_bar(&self, painter: &Painter, rect: Rect, fraction: f32, color: Color32) {
        // 背景
        painter.rect_filled(
            rect,
            Rounding::same(rect.height() / 2.0),
            self.theme.progress_background,
        );

        // 填充
        let fill_width = rect.width() * fraction.clamp(0.0, 1.0);
        if fill_width > 0.0 {
            let fill_rect = Rect::from_min_size(rect.min, Vec2::new(fill_width, rect.height()));
            painter.rect_filled(fill_rect, Rounding::same(rect.height() / 2.0), color);
        }
    }

    /// 获取进度条颜色（根据百分比）
    pub fn get_progress_color(&self, percentage: f32) -> Color32 {
        if percentage > 80.0 {
            self.theme.danger_color
        } else if percentage > 60.0 {
            self.theme.warning_color
        } else {
            self.theme.primary_color
        }
    }
}

/// 按钮样式工具
pub struct ButtonStyle<'a> {
    pub theme: &'a TaiLTheme,
}

impl<'a> ButtonStyle<'a> {
    pub fn new(theme: &'a TaiLTheme) -> Self {
        Self { theme }
    }

    /// 主按钮样式（使用 egui::Button 构建）
    pub fn primary_button(&self, text: impl Into<String>) -> egui::Button<'_> {
        egui::Button::new(egui::RichText::new(text).color(Color32::WHITE))
            .fill(self.theme.primary_color)
            .rounding(egui::Rounding::same(8.0))
    }

    /// 次要按钮样式
    pub fn secondary_button(&self, text: impl Into<String>) -> egui::Button<'_> {
        egui::Button::new(egui::RichText::new(text).color(self.theme.text_color))
            .fill(Color32::TRANSPARENT)
            .rounding(egui::Rounding::same(8.0))
            .stroke(egui::Stroke::new(1.0, self.theme.divider_color))
    }
}
