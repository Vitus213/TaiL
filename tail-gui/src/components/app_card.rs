//! TaiL GUI - 应用卡片组件

use egui::{Color32, Pos2, Rect, Response, Rounding, Sense, Stroke, Ui, Vec2, Widget};

use crate::theme::TaiLTheme;

/// 应用卡片组件
pub struct AppCard<'a> {
    /// 应用名称
    app_name: &'a str,
    /// 显示名称（可能是别名）
    display_name: &'a str,
    /// 使用时长（秒）
    duration_secs: i64,
    /// 占比百分比
    percentage: f32,
    /// 排名
    rank: usize,
    /// 窗口标题（可选）
    window_title: Option<&'a str>,
    /// 主题
    theme: &'a TaiLTheme,
    /// 是否选中
    selected: bool,
}

impl<'a> AppCard<'a> {
    pub fn new(
        app_name: &'a str,
        display_name: &'a str,
        duration_secs: i64,
        percentage: f32,
        rank: usize,
        theme: &'a TaiLTheme,
    ) -> Self {
        Self {
            app_name,
            display_name,
            duration_secs,
            percentage,
            rank,
            window_title: None,
            theme,
            selected: false,
        }
    }

    pub fn with_window_title(mut self, title: &'a str) -> Self {
        self.window_title = Some(title);
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// 格式化时长
    fn format_duration(seconds: i64) -> String {
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        let secs = seconds % 60;

        if hours > 0 {
            format!("{}h {}m", hours, minutes)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, secs)
        } else {
            format!("{}s", secs)
        }
    }

    /// 获取进度条颜色（根据使用时长）
    fn get_progress_color(&self) -> Color32 {
        if self.percentage > 80.0 {
            self.theme.danger_color
        } else if self.percentage > 60.0 {
            self.theme.warning_color
        } else {
            self.theme.primary_color
        }
    }

    /// 获取应用图标（使用 emoji 作为后备）
    fn get_app_icon(&self) -> &'static str {
        let name_lower = self.app_name.to_lowercase();
        
        if name_lower.contains("code") || name_lower.contains("vscode") {
            "💻"
        } else if name_lower.contains("firefox") || name_lower.contains("chrome") || name_lower.contains("browser") {
            "🌐"
        } else if name_lower.contains("terminal") || name_lower.contains("konsole") || name_lower.contains("alacritty") {
            "⌨️"
        } else if name_lower.contains("discord") || name_lower.contains("slack") || name_lower.contains("telegram") {
            "💬"
        } else if name_lower.contains("spotify") || name_lower.contains("music") {
            "🎵"
        } else if name_lower.contains("file") || name_lower.contains("nautilus") || name_lower.contains("dolphin") {
            "📁"
        } else if name_lower.contains("steam") || name_lower.contains("game") {
            "🎮"
        } else if name_lower.contains("obs") || name_lower.contains("video") {
            "🎬"
        } else if name_lower.contains("gimp") || name_lower.contains("inkscape") || name_lower.contains("krita") {
            "🎨"
        } else if name_lower.contains("libreoffice") || name_lower.contains("office") {
            "📄"
        } else {
            "📱"
        }
    }
}

impl<'a> Widget for AppCard<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        // 根据是否有窗口标题调整卡片高度
        let card_height = if self.window_title.is_some() { 90.0 } else { 70.0 };
        let desired_size = Vec2::new(ui.available_width(), card_height);
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            
            // 卡片背景
            let bg_color = if response.hovered() {
                self.theme.card_hover_background
            } else if self.selected {
                self.theme.card_selected_background
            } else {
                self.theme.card_background
            };

            // 绘制卡片背景和阴影
            let shadow_rect = rect.translate(Vec2::new(2.0, 2.0));
            painter.rect_filled(
                shadow_rect,
                Rounding::same(self.theme.card_rounding),
                Color32::from_black_alpha(30),
            );
            
            painter.rect_filled(
                rect,
                Rounding::same(self.theme.card_rounding),
                bg_color,
            );

            // 卡片边框
            if self.selected {
                painter.rect_stroke(
                    rect,
                    Rounding::same(self.theme.card_rounding),
                    Stroke::new(2.0, self.theme.primary_color),
                );
            }

            let padding = self.theme.card_padding;
            let content_rect = rect.shrink(padding);

            // 左侧：排名和图标
            let icon_size = 40.0;
            let icon_rect = Rect::from_min_size(
                content_rect.min,
                Vec2::new(icon_size, icon_size),
            );
            
            // 绘制图标背景
            painter.rect_filled(
                icon_rect,
                Rounding::same(8.0),
                self.theme.primary_color.linear_multiply(0.2),
            );
            
            // 绘制图标
            let icon = self.get_app_icon();
            painter.text(
                icon_rect.center(),
                egui::Align2::CENTER_CENTER,
                icon,
                egui::FontId::proportional(24.0),
                self.theme.text_color,
            );

            // 排名徽章
            let rank_pos = Pos2::new(icon_rect.right() - 8.0, icon_rect.top() - 4.0);
            painter.circle_filled(rank_pos, 10.0, self.theme.accent_color);
            painter.text(
                rank_pos,
                egui::Align2::CENTER_CENTER,
                format!("{}", self.rank),
                egui::FontId::proportional(10.0),
                Color32::WHITE,
            );

            // 中间：应用名称和窗口标题
            let text_left = icon_rect.right() + 12.0;
            let text_width = content_rect.width() - icon_size - 120.0;

            // 应用名称
            painter.text(
                Pos2::new(text_left, content_rect.min.y + 6.0),
                egui::Align2::LEFT_TOP,
                self.display_name,
                egui::FontId::proportional(self.theme.body_size),
                self.theme.text_color,
            );

            // 窗口标题（如果有）- 放在应用名称下方
            let progress_y_offset = if let Some(title) = self.window_title {
                let truncated_title = if title.chars().count() > 50 {
                    // 使用字符边界安全截断，避免在多字节字符中间截断
                    let truncated: String = title.chars().take(47).collect();
                    format!("{}...", truncated)
                } else {
                    title.to_string()
                };
                painter.text(
                    Pos2::new(text_left, content_rect.min.y + 24.0),
                    egui::Align2::LEFT_TOP,
                    truncated_title,
                    egui::FontId::proportional(self.theme.small_size),
                    self.theme.secondary_text_color,
                );
                44.0 // 有窗口标题时，进度条位置更低
            } else {
                28.0 // 无窗口标题时，进度条位置较高
            };

            // 进度条 - 放在窗口标题下方
            let progress_height = 6.0;
            let progress_y = content_rect.min.y + progress_y_offset;
            let progress_rect = Rect::from_min_size(
                Pos2::new(text_left, progress_y),
                Vec2::new(text_width.max(100.0), progress_height),
            );
            
            // 进度条背景
            painter.rect_filled(
                progress_rect,
                Rounding::same(3.0),
                self.theme.progress_background,
            );
            
            // 进度条填充
            let fill_width = progress_rect.width() * (self.percentage / 100.0).min(1.0);
            let fill_rect = Rect::from_min_size(
                progress_rect.min,
                Vec2::new(fill_width, progress_height),
            );
            painter.rect_filled(
                fill_rect,
                Rounding::same(3.0),
                self.get_progress_color(),
            );

            // 右侧：时长和百分比
            let right_x = content_rect.max.x;
            
            // 时长
            painter.text(
                Pos2::new(right_x, content_rect.min.y + 6.0),
                egui::Align2::RIGHT_TOP,
                Self::format_duration(self.duration_secs),
                egui::FontId::proportional(self.theme.body_size),
                self.theme.text_color,
            );

            // 百分比
            painter.text(
                Pos2::new(right_x, content_rect.min.y + 28.0),
                egui::Align2::RIGHT_TOP,
                format!("{:.1}%", self.percentage),
                egui::FontId::proportional(self.theme.small_size),
                self.theme.secondary_text_color,
            );
        }

        response
    }
}

/// 简化的应用列表项（用于紧凑显示）
#[allow(dead_code)]
pub struct AppListItem<'a> {
    app_name: &'a str,
    duration_secs: i64,
    percentage: f32,
    theme: &'a TaiLTheme,
}

impl<'a> AppListItem<'a> {
    pub fn new(
        app_name: &'a str,
        duration_secs: i64,
        percentage: f32,
        theme: &'a TaiLTheme,
    ) -> Self {
        Self {
            app_name,
            duration_secs,
            percentage,
            theme,
        }
    }

    fn format_duration(seconds: i64) -> String {
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;

        if hours > 0 {
            format!("{}h {}m", hours, minutes)
        } else {
            format!("{}m", minutes)
        }
    }
}

impl<'a> Widget for AppListItem<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let desired_size = Vec2::new(ui.available_width(), 32.0);
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::hover());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            
            // 悬停背景
            if response.hovered() {
                painter.rect_filled(
                    rect,
                    Rounding::same(4.0),
                    self.theme.card_hover_background,
                );
            }

            let padding = 8.0;
            
            // 应用名称
            painter.text(
                Pos2::new(rect.min.x + padding, rect.center().y),
                egui::Align2::LEFT_CENTER,
                self.app_name,
                egui::FontId::proportional(self.theme.small_size),
                self.theme.text_color,
            );

            // 时长
            painter.text(
                Pos2::new(rect.max.x - padding, rect.center().y),
                egui::Align2::RIGHT_CENTER,
                Self::format_duration(self.duration_secs),
                egui::FontId::proportional(self.theme.small_size),
                self.theme.secondary_text_color,
            );
        }

        response
    }
}