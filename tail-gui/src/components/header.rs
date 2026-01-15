//! TaiL GUI - 头部组件

use egui::{Color32, Pos2, Response, Rounding, Sense, Ui, Vec2, Widget};

use crate::theme::TaiLTheme;

/// 统计卡片组件
pub struct StatCard<'a> {
    /// 标题
    title: &'a str,
    /// 主要值
    value: &'a str,
    /// 副标题（可选）
    subtitle: Option<&'a str>,
    /// 图标
    icon: &'a str,
    /// 主题
    theme: &'a TaiLTheme,
    /// 强调色（可选）
    accent_color: Option<Color32>,
    /// 是否使用大尺寸
    large_size: bool,
}

impl<'a> StatCard<'a> {
    pub fn new(title: &'a str, value: &'a str, icon: &'a str, theme: &'a TaiLTheme) -> Self {
        Self {
            title,
            value,
            subtitle: None,
            icon,
            theme,
            accent_color: None,
            large_size: false,
        }
    }

    pub fn subtitle(mut self, subtitle: &'a str) -> Self {
        self.subtitle = Some(subtitle);
        self
    }

    pub fn with_subtitle_option(mut self, subtitle: Option<&'a str>) -> Self {
        self.subtitle = subtitle;
        self
    }

    pub fn accent_color(mut self, color: Color32) -> Self {
        self.accent_color = Some(color);
        self
    }

    pub fn large_size(mut self, large: bool) -> Self {
        self.large_size = large;
        self
    }
}

impl<'a> Widget for StatCard<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let desired_size = if self.large_size {
            Vec2::new(200.0, 110.0)
        } else {
            Vec2::new(180.0, 100.0)
        };
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::hover());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let padding = self.theme.card_padding;

            // 卡片背景
            let bg_color = if response.hovered() {
                self.theme.card_hover_background
            } else {
                self.theme.card_background
            };

            // 阴影
            let shadow_rect = rect.translate(Vec2::new(2.0, 2.0));
            painter.rect_filled(
                shadow_rect,
                Rounding::same(self.theme.card_rounding),
                Color32::from_black_alpha(20),
            );

            painter.rect_filled(rect, Rounding::same(self.theme.card_rounding), bg_color);

            let content_rect = rect.shrink(padding);
            let accent = self.accent_color.unwrap_or(self.theme.primary_color);

            // 图标（直接显示 emoji，不绘制背景）
            let icon_font_size = if self.large_size { 26.0 } else { 22.0 };
            painter.text(
                Pos2::new(content_rect.min.x, content_rect.min.y + 4.0),
                egui::Align2::LEFT_TOP,
                self.icon,
                egui::FontId::proportional(icon_font_size),
                accent,
            );

            // 标题
            painter.text(
                Pos2::new(content_rect.min.x + 36.0, content_rect.min.y + 6.0),
                egui::Align2::LEFT_TOP,
                self.title,
                egui::FontId::proportional(self.theme.small_size),
                self.theme.secondary_text_color,
            );

            // 主要值
            let value_font_size = if self.large_size {
                self.theme.heading_size * 1.2
            } else {
                self.theme.heading_size
            };
            painter.text(
                Pos2::new(content_rect.min.x + 36.0, content_rect.min.y + 24.0),
                egui::Align2::LEFT_TOP,
                self.value,
                egui::FontId::proportional(value_font_size),
                self.theme.text_color,
            );

            // 副标题
            if let Some(subtitle) = self.subtitle {
                painter.text(
                    Pos2::new(content_rect.max.x, content_rect.max.y),
                    egui::Align2::RIGHT_BOTTOM,
                    subtitle,
                    egui::FontId::proportional(self.theme.small_size - 2.0),
                    self.theme.secondary_text_color,
                );
            }
        }

        response
    }
}

/// 页面标题组件
pub struct PageHeader<'a> {
    /// 标题
    title: &'a str,
    /// 图标
    icon: &'a str,
    /// 副标题（可选）
    subtitle: Option<&'a str>,
    /// 主题
    theme: &'a TaiLTheme,
}

impl<'a> PageHeader<'a> {
    pub fn new(title: &'a str, icon: &'a str, theme: &'a TaiLTheme) -> Self {
        Self {
            title,
            icon,
            subtitle: None,
            theme,
        }
    }

    pub fn subtitle(mut self, subtitle: &'a str) -> Self {
        self.subtitle = Some(subtitle);
        self
    }
}

impl<'a> Widget for PageHeader<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let height = if self.subtitle.is_some() { 60.0 } else { 45.0 };
        let desired_size = Vec2::new(ui.available_width(), height);
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::hover());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // 图标和标题
            let icon_x = rect.min.x;
            let title_x = icon_x + 35.0;

            painter.text(
                Pos2::new(icon_x, rect.min.y + 5.0),
                egui::Align2::LEFT_TOP,
                self.icon,
                egui::FontId::proportional(28.0),
                self.theme.primary_color,
            );

            painter.text(
                Pos2::new(title_x, rect.min.y + 5.0),
                egui::Align2::LEFT_TOP,
                self.title,
                egui::FontId::proportional(self.theme.heading_size),
                self.theme.text_color,
            );

            // 副标题
            if let Some(subtitle) = self.subtitle {
                painter.text(
                    Pos2::new(title_x, rect.min.y + 35.0),
                    egui::Align2::LEFT_TOP,
                    subtitle,
                    egui::FontId::proportional(self.theme.small_size),
                    self.theme.secondary_text_color,
                );
            }
        }

        response
    }
}

/// 分隔线组件
pub struct SectionDivider<'a> {
    /// 标题（可选）
    title: Option<&'a str>,
    /// 主题
    theme: &'a TaiLTheme,
}

impl<'a> SectionDivider<'a> {
    pub fn new(theme: &'a TaiLTheme) -> Self {
        Self { title: None, theme }
    }

    pub fn with_title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }
}

impl<'a> Widget for SectionDivider<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let height = if self.title.is_some() { 30.0 } else { 16.0 };
        let desired_size = Vec2::new(ui.available_width(), height);
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::hover());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let line_y = rect.center().y;

            if let Some(title) = self.title {
                // 带标题的分隔线（加粗）
                let text_galley = painter.layout_no_wrap(
                    title.to_string(),
                    egui::FontId::proportional(self.theme.body_size),
                    self.theme.text_color,
                );
                let text_width = text_galley.rect.width();
                let text_x = rect.center().x - text_width / 2.0;

                // 左边线
                painter.line_segment(
                    [
                        Pos2::new(rect.min.x, line_y),
                        Pos2::new(text_x - 10.0, line_y),
                    ],
                    egui::Stroke::new(1.0, self.theme.divider_color),
                );

                // 文字
                painter.galley(
                    Pos2::new(text_x, rect.min.y + 8.0),
                    text_galley,
                    self.theme.text_color,
                );

                // 右边线
                painter.line_segment(
                    [
                        Pos2::new(text_x + text_width + 10.0, line_y),
                        Pos2::new(rect.max.x, line_y),
                    ],
                    egui::Stroke::new(1.0, self.theme.divider_color),
                );
            } else {
                // 简单分隔线
                painter.line_segment(
                    [Pos2::new(rect.min.x, line_y), Pos2::new(rect.max.x, line_y)],
                    egui::Stroke::new(1.0, self.theme.divider_color),
                );
            }
        }

        response
    }
}

/// 空状态组件
pub struct EmptyState<'a> {
    /// 图标
    icon: &'a str,
    /// 标题
    title: &'a str,
    /// 描述
    description: &'a str,
    /// 主题
    theme: &'a TaiLTheme,
}

impl<'a> EmptyState<'a> {
    pub fn new(icon: &'a str, title: &'a str, description: &'a str, theme: &'a TaiLTheme) -> Self {
        Self {
            icon,
            title,
            description,
            theme,
        }
    }
}

impl<'a> Widget for EmptyState<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let desired_size = Vec2::new(ui.available_width(), 200.0);
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::hover());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let center_x = rect.center().x;

            // 图标
            painter.text(
                Pos2::new(center_x, rect.min.y + 40.0),
                egui::Align2::CENTER_CENTER,
                self.icon,
                egui::FontId::proportional(48.0),
                self.theme.secondary_text_color.linear_multiply(0.5),
            );

            // 标题
            painter.text(
                Pos2::new(center_x, rect.min.y + 100.0),
                egui::Align2::CENTER_CENTER,
                self.title,
                egui::FontId::proportional(self.theme.body_size),
                self.theme.text_color,
            );

            // 描述
            painter.text(
                Pos2::new(center_x, rect.min.y + 130.0),
                egui::Align2::CENTER_CENTER,
                self.description,
                egui::FontId::proportional(self.theme.small_size),
                self.theme.secondary_text_color,
            );
        }

        response
    }
}
