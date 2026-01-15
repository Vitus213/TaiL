//! 层级柱形图组件 - 支持可下钻的时间导航

use egui::{Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};
use tail_core::models::{PeriodUsage, TimeNavigationLevel};

use crate::theme::TaiLTheme;
use crate::utils::duration;

/// 层级柱形图组件
pub struct HierarchicalBarChart<'a> {
    /// 时间段数据
    periods: &'a [PeriodUsage],
    /// 当前导航层级
    level: TimeNavigationLevel,
    /// 图表标题
    title: &'a str,
    /// 主题
    theme: &'a TaiLTheme,
}

impl<'a> HierarchicalBarChart<'a> {
    /// 创建新的层级柱形图
    pub fn new(
        periods: &'a [PeriodUsage],
        level: TimeNavigationLevel,
        title: &'a str,
        theme: &'a TaiLTheme,
    ) -> Self {
        Self {
            periods,
            level,
            title,
            theme,
        }
    }

    /// 显示图表，返回被点击的时间段索引
    pub fn show(&self, ui: &mut Ui) -> Option<i32> {
        let mut clicked_index = None;

        ui.vertical(|ui| {
            // 标题
            ui.heading(self.title);
            ui.add_space(8.0);

            if self.periods.is_empty() {
                ui.label("暂无数据");
                return;
            }

            // 计算最大值用于归一化
            let max_seconds = self
                .periods
                .iter()
                .map(|p| p.total_seconds)
                .max()
                .unwrap_or(1);

            // 图表区域
            let available_width = ui.available_width();
            let chart_height = 200.0;
            let bar_spacing = 8.0;
            let bar_count = self.periods.len() as f32;
            let bar_width = (available_width - bar_spacing * (bar_count + 1.0)) / bar_count;

            let (response, painter) =
                ui.allocate_painter(Vec2::new(available_width, chart_height), Sense::hover());

            let chart_rect = response.rect;

            // 绘制每个柱子
            for (i, period) in self.periods.iter().enumerate() {
                let x = chart_rect.min.x + bar_spacing + i as f32 * (bar_width + bar_spacing);
                let normalized_height = if max_seconds > 0 {
                    (period.total_seconds as f32 / max_seconds as f32) * (chart_height - 40.0)
                } else {
                    0.0
                };
                let y = chart_rect.max.y - 20.0 - normalized_height;

                let bar_rect =
                    Rect::from_min_size(Pos2::new(x, y), Vec2::new(bar_width, normalized_height));

                // 检测鼠标交互
                let bar_response = ui.interact(bar_rect, ui.id().with(i), Sense::click());

                // 确定柱子颜色
                let bar_color = if bar_response.hovered() {
                    self.theme.accent_color
                } else {
                    self.get_bar_color(period.total_seconds, max_seconds)
                };

                // 绘制柱子
                painter.rect_filled(bar_rect, 2.0, bar_color);
                painter.rect_stroke(bar_rect, 2.0, Stroke::new(1.0, self.theme.divider_color));

                // 绘制标签
                let label_pos = Pos2::new(x + bar_width / 2.0, chart_rect.max.y - 10.0);
                painter.text(
                    label_pos,
                    egui::Align2::CENTER_CENTER,
                    self.format_label(&period.label),
                    egui::FontId::proportional(10.0),
                    self.theme.text_color,
                );

                // 悬停提示
                let bar_response = if bar_response.hovered() {
                    bar_response.on_hover_text(format!(
                        "{}: {}",
                        period.label,
                        duration::format_duration_chinese(period.total_seconds)
                    ))
                } else {
                    bar_response
                };

                // 点击事件
                if bar_response.clicked() {
                    clicked_index = Some(period.index);
                }
            }
        });

        clicked_index
    }

    /// 根据使用量获取柱子颜色
    fn get_bar_color(&self, seconds: i64, max_seconds: i64) -> Color32 {
        if seconds == 0 {
            return self.theme.divider_color;
        }

        let ratio = seconds as f32 / max_seconds as f32;

        if ratio > 0.75 {
            self.theme.primary_color
        } else if ratio > 0.5 {
            self.blend_color(self.theme.primary_color, 0.7)
        } else {
            self.blend_color(self.theme.primary_color, 0.4)
        }
    }

    /// 混合颜色
    fn blend_color(&self, color: Color32, factor: f32) -> Color32 {
        Color32::from_rgba_premultiplied(
            (color.r() as f32 * factor) as u8,
            (color.g() as f32 * factor) as u8,
            (color.b() as f32 * factor) as u8,
            color.a(),
        )
    }

    /// 格式化标签（根据层级简化显示）
    fn format_label(&self, label: &str) -> String {
        match self.level {
            TimeNavigationLevel::Year => label.to_string(),
            TimeNavigationLevel::Month => {
                // 简化月份标签：1月 -> 1
                label.replace("月", "")
            }
            TimeNavigationLevel::Week => {
                // 简化周标签：第1周 -> W1
                label.replace("第", "W").replace("周", "")
            }
            TimeNavigationLevel::Day => {
                // 简化日期标签：周一 -> 一
                label.replace("周", "")
            }
            TimeNavigationLevel::Hour => {
                // 简化小时标签：0时 -> 0
                label.replace("时", "")
            }
        }
    }
}
