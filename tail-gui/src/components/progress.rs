//! TaiL GUI - 进度条组件

use egui::{Color32, Pos2, Rect, Response, Rounding, Sense, Ui, Vec2, Widget};

use crate::theme::TaiLTheme;

/// 增强的进度条组件
pub struct EnhancedProgressBar<'a> {
    /// 进度值 (0.0 - 1.0)
    fraction: f32,
    /// 目标值（可选，用于显示目标线）
    target: Option<f32>,
    /// 是否显示百分比文本
    show_percentage: bool,
    /// 是否使用渐变色
    use_gradient: bool,
    /// 高度
    height: f32,
    /// 主题
    theme: &'a TaiLTheme,
    /// 自定义颜色（可选）
    custom_color: Option<Color32>,
    /// 标签文本（可选）
    label: Option<&'a str>,
}

impl<'a> EnhancedProgressBar<'a> {
    pub fn new(fraction: f32, theme: &'a TaiLTheme) -> Self {
        Self {
            fraction: fraction.clamp(0.0, 1.0),
            target: None,
            show_percentage: false,
            use_gradient: true,
            height: 8.0,
            theme,
            custom_color: None,
            label: None,
        }
    }

    pub fn with_target(mut self, target: f32) -> Self {
        self.target = Some(target.clamp(0.0, 1.0));
        self
    }

    pub fn show_percentage(mut self, show: bool) -> Self {
        self.show_percentage = show;
        self
    }

    pub fn use_gradient(mut self, use_gradient: bool) -> Self {
        self.use_gradient = use_gradient;
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub fn color(mut self, color: Color32) -> Self {
        self.custom_color = Some(color);
        self
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    /// 获取进度条颜色
    fn get_color(&self) -> Color32 {
        if let Some(color) = self.custom_color {
            return color;
        }

        if self.use_gradient {
            // 根据进度值返回渐变色
            if self.fraction > 0.9 {
                self.theme.danger_color
            } else if self.fraction > 0.7 {
                self.theme.warning_color
            } else if self.fraction > 0.5 {
                // 橙黄色过渡
                Color32::from_rgb(255, 180, 50)
            } else {
                self.theme.success_color
            }
        } else {
            self.theme.primary_color
        }
    }
}

impl<'a> Widget for EnhancedProgressBar<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let total_height = if self.label.is_some() || self.show_percentage {
            self.height + 20.0
        } else {
            self.height
        };

        let desired_size = Vec2::new(ui.available_width(), total_height);
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::hover());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // 标签和百分比
            let bar_top = if self.label.is_some() || self.show_percentage {
                // 绘制标签
                if let Some(label) = self.label {
                    painter.text(
                        Pos2::new(rect.min.x, rect.min.y),
                        egui::Align2::LEFT_TOP,
                        label,
                        egui::FontId::proportional(self.theme.small_size),
                        self.theme.text_color,
                    );
                }

                // 绘制百分比
                if self.show_percentage {
                    painter.text(
                        Pos2::new(rect.max.x, rect.min.y),
                        egui::Align2::RIGHT_TOP,
                        format!("{:.1}%", self.fraction * 100.0),
                        egui::FontId::proportional(self.theme.small_size),
                        self.theme.secondary_text_color,
                    );
                }

                rect.min.y + 18.0
            } else {
                rect.min.y
            };

            // 进度条区域
            let bar_rect = Rect::from_min_size(
                Pos2::new(rect.min.x, bar_top),
                Vec2::new(rect.width(), self.height),
            );

            // 背景
            painter.rect_filled(
                bar_rect,
                Rounding::same(self.height / 2.0),
                self.theme.progress_background,
            );

            // 填充
            if self.fraction > 0.0 {
                let fill_width = bar_rect.width() * self.fraction;
                let fill_rect =
                    Rect::from_min_size(bar_rect.min, Vec2::new(fill_width, self.height));

                painter.rect_filled(
                    fill_rect,
                    Rounding::same(self.height / 2.0),
                    self.get_color(),
                );
            }

            // 目标线
            if let Some(target) = self.target {
                let target_x = bar_rect.min.x + bar_rect.width() * target;
                painter.line_segment(
                    [
                        Pos2::new(target_x, bar_rect.min.y - 2.0),
                        Pos2::new(target_x, bar_rect.max.y + 2.0),
                    ],
                    egui::Stroke::new(2.0, self.theme.accent_color),
                );
            }
        }

        response
    }
}

/// 圆形进度指示器
pub struct CircularProgress<'a> {
    /// 进度值 (0.0 - 1.0)
    fraction: f32,
    /// 半径
    radius: f32,
    /// 线宽
    stroke_width: f32,
    /// 主题
    theme: &'a TaiLTheme,
    /// 中心文本
    center_text: Option<String>,
}

impl<'a> CircularProgress<'a> {
    pub fn new(fraction: f32, theme: &'a TaiLTheme) -> Self {
        Self {
            fraction: fraction.clamp(0.0, 1.0),
            radius: 40.0,
            stroke_width: 6.0,
            theme,
            center_text: None,
        }
    }

    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    pub fn stroke_width(mut self, width: f32) -> Self {
        self.stroke_width = width;
        self
    }

    pub fn center_text(mut self, text: impl Into<String>) -> Self {
        self.center_text = Some(text.into());
        self
    }

    fn get_color(&self) -> Color32 {
        if self.fraction > 0.9 {
            self.theme.danger_color
        } else if self.fraction > 0.7 {
            self.theme.warning_color
        } else {
            self.theme.success_color
        }
    }
}

impl<'a> Widget for CircularProgress<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let size = self.radius * 2.0 + self.stroke_width;
        let desired_size = Vec2::splat(size);
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::hover());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let center = rect.center();

            // 背景圆环
            painter.circle_stroke(
                center,
                self.radius,
                egui::Stroke::new(self.stroke_width, self.theme.progress_background),
            );

            // 进度圆弧
            if self.fraction > 0.0 {
                let start_angle = -std::f32::consts::FRAC_PI_2; // 从顶部开始
                let end_angle = start_angle + self.fraction * std::f32::consts::TAU;

                // 使用多个线段绘制圆弧
                let segments = (self.fraction * 60.0).max(1.0) as usize;
                let angle_step = (end_angle - start_angle) / segments as f32;

                for i in 0..segments {
                    let a1 = start_angle + i as f32 * angle_step;
                    let a2 = start_angle + (i + 1) as f32 * angle_step;

                    let p1 = center + Vec2::new(a1.cos(), a1.sin()) * self.radius;
                    let p2 = center + Vec2::new(a2.cos(), a2.sin()) * self.radius;

                    painter.line_segment(
                        [p1, p2],
                        egui::Stroke::new(self.stroke_width, self.get_color()),
                    );
                }
            }

            // 中心文本
            if let Some(text) = &self.center_text {
                painter.text(
                    center,
                    egui::Align2::CENTER_CENTER,
                    text,
                    egui::FontId::proportional(self.theme.body_size),
                    self.theme.text_color,
                );
            }
        }

        response
    }
}

/// 目标进度卡片
pub struct GoalProgressCard<'a> {
    /// 应用名称
    app_name: &'a str,
    /// 当前使用时间（分钟）
    current_minutes: i32,
    /// 目标时间（分钟）
    target_minutes: i32,
    /// 主题
    theme: &'a TaiLTheme,
}

impl<'a> GoalProgressCard<'a> {
    pub fn new(
        app_name: &'a str,
        current_minutes: i32,
        target_minutes: i32,
        theme: &'a TaiLTheme,
    ) -> Self {
        Self {
            app_name,
            current_minutes,
            target_minutes,
            theme,
        }
    }

    fn format_time(minutes: i32) -> String {
        let hours = minutes / 60;
        let mins = minutes % 60;
        if hours > 0 {
            format!("{}h {}m", hours, mins)
        } else {
            format!("{}m", mins)
        }
    }
}

impl<'a> Widget for GoalProgressCard<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let desired_size = Vec2::new(ui.available_width(), 70.0);
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

            painter.rect_filled(rect, Rounding::same(self.theme.card_rounding), bg_color);

            let content_rect = rect.shrink(padding);
            let fraction = (self.current_minutes as f32 / self.target_minutes as f32).min(1.5);

            // 状态颜色
            let status_color = if fraction > 1.0 {
                self.theme.danger_color
            } else if fraction > 0.8 {
                self.theme.warning_color
            } else {
                self.theme.success_color
            };

            // 应用名称
            painter.text(
                Pos2::new(content_rect.min.x, content_rect.min.y),
                egui::Align2::LEFT_TOP,
                self.app_name,
                egui::FontId::proportional(self.theme.body_size),
                self.theme.text_color,
            );

            // 时间信息
            let time_text = format!(
                "{} / {}",
                Self::format_time(self.current_minutes),
                Self::format_time(self.target_minutes)
            );
            painter.text(
                Pos2::new(content_rect.max.x, content_rect.min.y),
                egui::Align2::RIGHT_TOP,
                time_text,
                egui::FontId::proportional(self.theme.small_size),
                self.theme.secondary_text_color,
            );

            // 进度条
            let bar_height = 8.0;
            let bar_y = content_rect.max.y - bar_height;
            let bar_rect = Rect::from_min_size(
                Pos2::new(content_rect.min.x, bar_y),
                Vec2::new(content_rect.width(), bar_height),
            );

            // 背景
            painter.rect_filled(
                bar_rect,
                Rounding::same(bar_height / 2.0),
                self.theme.progress_background,
            );

            // 填充
            let fill_fraction = fraction.min(1.0);
            let fill_rect = Rect::from_min_size(
                bar_rect.min,
                Vec2::new(bar_rect.width() * fill_fraction, bar_height),
            );
            painter.rect_filled(fill_rect, Rounding::same(bar_height / 2.0), status_color);

            // 超出部分（如果超过目标）
            if fraction > 1.0 {
                let overflow_width = bar_rect.width() * (fraction - 1.0).min(0.5);
                let overflow_rect = Rect::from_min_size(
                    Pos2::new(bar_rect.max.x - overflow_width, bar_rect.min.y),
                    Vec2::new(overflow_width, bar_height),
                );
                painter.rect_filled(
                    overflow_rect,
                    Rounding::same(bar_height / 2.0),
                    self.theme.danger_color.linear_multiply(0.5),
                );
            }
        }

        response
    }
}
