//! TaiL GUI - 主题模块

use eframe::egui::{self, Color32};
use serde::{Deserialize, Serialize};

/// TaiL 主题配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaiLTheme {
    // 主要颜色
    pub primary_color: Color32,
    pub accent_color: Color32,
    pub success_color: Color32,
    pub warning_color: Color32,
    pub danger_color: Color32,

    // 背景颜色
    pub background_color: Color32,
    pub card_background: Color32,
    pub card_hover_background: Color32,
    pub card_selected_background: Color32,

    // 文字颜色
    pub text_color: Color32,
    pub secondary_text_color: Color32,

    // 进度条颜色
    pub progress_background: Color32,

    // 分隔线颜色
    pub divider_color: Color32,

    // 字体大小
    pub heading_size: f32,
    pub body_size: f32,
    pub small_size: f32,

    // 间距
    pub spacing: f32,
    pub card_padding: f32,
    pub card_rounding: f32,
}

impl Default for TaiLTheme {
    fn default() -> Self {
        Self::dark()
    }
}

impl TaiLTheme {
    /// 深色主题
    pub fn dark() -> Self {
        Self {
            // 主要颜色
            primary_color: Color32::from_rgb(74, 144, 226), // #4A90E2
            accent_color: Color32::from_rgb(102, 187, 106), // 绿色
            success_color: Color32::from_rgb(102, 187, 106), // 绿色
            warning_color: Color32::from_rgb(255, 167, 38), // 橙色
            danger_color: Color32::from_rgb(239, 83, 80),   // 红色

            // 背景颜色
            background_color: Color32::from_rgb(30, 30, 30), // #1E1E1E
            card_background: Color32::from_rgb(45, 45, 45),  // #2D2D2D
            card_hover_background: Color32::from_rgb(55, 55, 55),
            card_selected_background: Color32::from_rgb(60, 60, 70),

            // 文字颜色
            text_color: Color32::from_rgb(230, 230, 230),
            secondary_text_color: Color32::from_rgb(160, 160, 160),

            // 进度条颜色
            progress_background: Color32::from_rgb(60, 60, 60),

            // 分隔线颜色
            divider_color: Color32::from_rgb(70, 70, 70),

            // 字体大小
            heading_size: 24.0,
            body_size: 16.0,
            small_size: 13.0,

            // 间距
            spacing: 16.0,
            card_padding: 16.0,
            card_rounding: 12.0,
        }
    }

    /// 浅色主题
    pub fn light() -> Self {
        Self {
            // 主要颜色
            primary_color: Color32::from_rgb(74, 144, 226),
            accent_color: Color32::from_rgb(76, 175, 80),
            success_color: Color32::from_rgb(76, 175, 80),
            warning_color: Color32::from_rgb(255, 152, 0),
            danger_color: Color32::from_rgb(244, 67, 54),

            // 背景颜色
            background_color: Color32::from_rgb(250, 250, 250),
            card_background: Color32::from_rgb(255, 255, 255),
            card_hover_background: Color32::from_rgb(245, 245, 245),
            card_selected_background: Color32::from_rgb(232, 240, 254),

            // 文字颜色
            text_color: Color32::from_rgb(33, 33, 33),
            secondary_text_color: Color32::from_rgb(117, 117, 117),

            // 进度条颜色
            progress_background: Color32::from_rgb(224, 224, 224),

            // 分隔线颜色
            divider_color: Color32::from_rgb(224, 224, 224),

            // 字体大小
            heading_size: 24.0,
            body_size: 16.0,
            small_size: 13.0,

            // 间距
            spacing: 16.0,
            card_padding: 16.0,
            card_rounding: 12.0,
        }
    }

    /// Catppuccin Mocha 主题
    pub fn catppuccin_mocha() -> Self {
        Self {
            // 主要颜色
            primary_color: Color32::from_rgb(137, 180, 250), // Blue
            accent_color: Color32::from_rgb(166, 227, 161),  // Green
            success_color: Color32::from_rgb(166, 227, 161), // Green
            warning_color: Color32::from_rgb(249, 226, 175), // Yellow
            danger_color: Color32::from_rgb(243, 139, 168),  // Red

            // 背景颜色
            background_color: Color32::from_rgb(30, 30, 46), // Base
            card_background: Color32::from_rgb(49, 50, 68),  // Surface0
            card_hover_background: Color32::from_rgb(69, 71, 90), // Surface1
            card_selected_background: Color32::from_rgb(88, 91, 112), // Surface2

            // 文字颜色
            text_color: Color32::from_rgb(205, 214, 244), // Text
            secondary_text_color: Color32::from_rgb(166, 173, 200), // Subtext0

            // 进度条颜色
            progress_background: Color32::from_rgb(69, 71, 90),

            // 分隔线颜色
            divider_color: Color32::from_rgb(88, 91, 112),

            // 字体大小
            heading_size: 24.0,
            body_size: 16.0,
            small_size: 13.0,

            // 间距
            spacing: 16.0,
            card_padding: 16.0,
            card_rounding: 12.0,
        }
    }

    /// Nord 主题
    pub fn nord() -> Self {
        Self {
            // 主要颜色
            primary_color: Color32::from_rgb(136, 192, 208), // Nord8
            accent_color: Color32::from_rgb(163, 190, 140),  // Nord14
            success_color: Color32::from_rgb(163, 190, 140), // Nord14
            warning_color: Color32::from_rgb(235, 203, 139), // Nord13
            danger_color: Color32::from_rgb(191, 97, 106),   // Nord11

            // 背景颜色
            background_color: Color32::from_rgb(46, 52, 64), // Nord0
            card_background: Color32::from_rgb(59, 66, 82),  // Nord1
            card_hover_background: Color32::from_rgb(67, 76, 94), // Nord2
            card_selected_background: Color32::from_rgb(76, 86, 106), // Nord3

            // 文字颜色
            text_color: Color32::from_rgb(236, 239, 244), // Nord6
            secondary_text_color: Color32::from_rgb(216, 222, 233), // Nord4

            // 进度条颜色
            progress_background: Color32::from_rgb(67, 76, 94),

            // 分隔线颜色
            divider_color: Color32::from_rgb(76, 86, 106),

            // 字体大小
            heading_size: 24.0,
            body_size: 16.0,
            small_size: 13.0,

            // 间距
            spacing: 16.0,
            card_padding: 16.0,
            card_rounding: 12.0,
        }
    }

    /// Tokyo Night 主题
    pub fn tokyo_night() -> Self {
        Self {
            // 主要颜色
            primary_color: Color32::from_rgb(122, 162, 255), // Blue
            accent_color: Color32::from_rgb(187, 154, 247),  // Purple
            success_color: Color32::from_rgb(86, 182, 194),  // Cyan
            warning_color: Color32::from_rgb(255, 179, 109), // Orange
            danger_color: Color32::from_rgb(247, 118, 142),  // Red

            // 背景颜色
            background_color: Color32::from_rgb(26, 27, 38), // Base
            card_background: Color32::from_rgb(34, 35, 48),  // Surface1
            card_hover_background: Color32::from_rgb(44, 45, 60), // Surface2
            card_selected_background: Color32::from_rgb(54, 55, 72), // Surface0

            // 文字颜色
            text_color: Color32::from_rgb(187, 187, 187), // White
            secondary_text_color: Color32::from_rgb(113, 119, 138), // Grey

            // 进度条颜色
            progress_background: Color32::from_rgb(44, 45, 60),

            // 分隔线颜色
            divider_color: Color32::from_rgb(54, 55, 72),

            // 字体大小
            heading_size: 24.0,
            body_size: 16.0,
            small_size: 13.0,

            // 间距
            spacing: 16.0,
            card_padding: 16.0,
            card_rounding: 12.0,
        }
    }

    /// Dracula 主题
    pub fn dracula() -> Self {
        Self {
            // 主要颜色
            primary_color: Color32::from_rgb(80, 250, 123), // Green
            accent_color: Color32::from_rgb(189, 147, 249), // Purple
            success_color: Color32::from_rgb(80, 250, 123), // Green
            warning_color: Color32::from_rgb(255, 184, 108), // Yellow
            danger_color: Color32::from_rgb(255, 85, 85),   // Red

            // 背景颜色
            background_color: Color32::from_rgb(40, 42, 54), // Background
            card_background: Color32::from_rgb(68, 71, 90),  // Current Line
            card_hover_background: Color32::from_rgb(88, 91, 112),
            card_selected_background: Color32::from_rgb(98, 114, 164),

            // 文字颜色
            text_color: Color32::from_rgb(248, 248, 242), // Foreground
            secondary_text_color: Color32::from_rgb(98, 114, 164), // Comment

            // 进度条颜色
            progress_background: Color32::from_rgb(68, 71, 90),

            // 分隔线颜色
            divider_color: Color32::from_rgb(98, 114, 164),

            // 字体大小
            heading_size: 24.0,
            body_size: 16.0,
            small_size: 13.0,

            // 间距
            spacing: 16.0,
            card_padding: 16.0,
            card_rounding: 12.0,
        }
    }

    /// 应用主题到 egui 上下文
    pub fn apply(&self, ctx: &egui::Context) {
        let mut visuals = egui::Visuals::dark();

        // 设置窗口背景
        visuals.panel_fill = self.background_color;
        visuals.window_fill = self.card_background;

        // 设置控件颜色
        visuals.widgets.noninteractive.bg_fill = self.card_background;
        visuals.widgets.inactive.bg_fill = self.card_background;
        visuals.widgets.hovered.bg_fill = self.card_hover_background;
        visuals.widgets.active.bg_fill = self.primary_color;

        // 设置文字颜色
        visuals.widgets.noninteractive.fg_stroke.color = self.text_color;
        visuals.widgets.inactive.fg_stroke.color = self.text_color;
        visuals.widgets.hovered.fg_stroke.color = self.text_color;
        visuals.widgets.active.fg_stroke.color = Color32::WHITE;

        // 设置选择颜色
        visuals.selection.bg_fill = self.primary_color.linear_multiply(0.3);
        visuals.selection.stroke.color = self.primary_color;

        // 设置超链接颜色
        visuals.hyperlink_color = self.primary_color;

        // 设置圆角
        visuals.window_rounding = egui::Rounding::same(self.card_rounding);
        visuals.widgets.noninteractive.rounding = egui::Rounding::same(8.0);
        visuals.widgets.inactive.rounding = egui::Rounding::same(8.0);
        visuals.widgets.hovered.rounding = egui::Rounding::same(8.0);
        visuals.widgets.active.rounding = egui::Rounding::same(8.0);

        // 设置滚动条样式
        visuals.widgets.noninteractive.bg_fill = self.card_background;
        visuals.widgets.inactive.fg_stroke.color = self.secondary_text_color;
        visuals.widgets.hovered.fg_stroke.color = self.text_color;

        ctx.set_visuals(visuals);

        // 设置字体大小
        let mut style = (*ctx.style()).clone();
        style.text_styles.insert(
            egui::TextStyle::Heading,
            egui::FontId::proportional(self.heading_size),
        );
        style.text_styles.insert(
            egui::TextStyle::Body,
            egui::FontId::proportional(self.body_size),
        );
        style.text_styles.insert(
            egui::TextStyle::Small,
            egui::FontId::proportional(self.small_size),
        );
        style.text_styles.insert(
            egui::TextStyle::Button,
            egui::FontId::proportional(self.body_size),
        );

        // 设置间距
        style.spacing.item_spacing = egui::vec2(self.spacing / 2.0, self.spacing / 2.0);
        style.spacing.window_margin = egui::Margin::same(self.spacing);
        style.spacing.button_padding = egui::vec2(12.0, 6.0);

        // 设置滚动条
        style.spacing.scroll.bar_width = 8.0;

        ctx.set_style(style);
    }
}

/// 主题类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ThemeType {
    Light,
    #[default]
    Dark,
    CatppuccinMocha,
    Nord,
    TokyoNight,
    Dracula,
    Auto,
}

impl ThemeType {
    /// 获取主题名称
    pub fn name(&self) -> &'static str {
        match self {
            ThemeType::Light => "浅色",
            ThemeType::Dark => "深色",
            ThemeType::CatppuccinMocha => "Catppuccin Mocha",
            ThemeType::Nord => "Nord",
            ThemeType::TokyoNight => "Tokyo Night",
            ThemeType::Dracula => "Dracula",
            ThemeType::Auto => "跟随系统",
        }
    }

    /// 获取所有主题类型
    pub fn all() -> &'static [ThemeType] {
        &[
            ThemeType::Light,
            ThemeType::Dark,
            ThemeType::CatppuccinMocha,
            ThemeType::Nord,
            ThemeType::TokyoNight,
            ThemeType::Dracula,
            ThemeType::Auto,
        ]
    }

    /// 转换为 TaiLTheme
    pub fn to_theme(&self) -> TaiLTheme {
        match self {
            ThemeType::Light => TaiLTheme::light(),
            ThemeType::Dark => TaiLTheme::dark(),
            ThemeType::CatppuccinMocha => TaiLTheme::catppuccin_mocha(),
            ThemeType::Nord => TaiLTheme::nord(),
            ThemeType::TokyoNight => TaiLTheme::tokyo_night(),
            ThemeType::Dracula => TaiLTheme::dracula(),
            ThemeType::Auto => {
                // 检测系统主题
                #[cfg(target_os = "linux")]
                {
                    if dark_light::detect() == dark_light::Mode::Dark {
                        TaiLTheme::dark()
                    } else {
                        TaiLTheme::light()
                    }
                }
                #[cfg(not(target_os = "linux"))]
                {
                    TaiLTheme::dark()
                }
            }
        }
    }
}

// 为 Color32 实现 Serialize 和 Deserialize
#[allow(dead_code)]
mod color32_serde {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(color: &Color32, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let rgba = color.to_array();
        serializer.serialize_str(&format!(
            "#{:02x}{:02x}{:02x}{:02x}",
            rgba[0], rgba[1], rgba[2], rgba[3]
        ))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Color32, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let s = s.trim_start_matches('#');

        if s.len() == 8 {
            let r = u8::from_str_radix(&s[0..2], 16).map_err(serde::de::Error::custom)?;
            let g = u8::from_str_radix(&s[2..4], 16).map_err(serde::de::Error::custom)?;
            let b = u8::from_str_radix(&s[4..6], 16).map_err(serde::de::Error::custom)?;
            let a = u8::from_str_radix(&s[6..8], 16).map_err(serde::de::Error::custom)?;
            Ok(Color32::from_rgba_unmultiplied(r, g, b, a))
        } else if s.len() == 6 {
            let r = u8::from_str_radix(&s[0..2], 16).map_err(serde::de::Error::custom)?;
            let g = u8::from_str_radix(&s[2..4], 16).map_err(serde::de::Error::custom)?;
            let b = u8::from_str_radix(&s[4..6], 16).map_err(serde::de::Error::custom)?;
            Ok(Color32::from_rgb(r, g, b))
        } else {
            Err(serde::de::Error::custom("Invalid color format"))
        }
    }
}
