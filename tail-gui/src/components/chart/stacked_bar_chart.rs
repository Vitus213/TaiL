//! 通用堆叠柱形图组件
//!
//! 支持不同时间粒度和分组模式的堆叠柱形图显示

use egui::{Color32, Pos2, Rect, Rounding, Sense, Stroke, Ui, Vec2};
use std::collections::HashMap;

use super::chart_data::{CategoryColorMap, ChartData, ChartTimeGranularity};
use crate::theme::TaiLTheme;

/// 堆叠柱形图配置
pub struct StackedBarChartConfig {
    /// 分组颜色映射
    pub color_map: CategoryColorMap,
    /// 是否显示图例
    pub show_legend: bool,
    /// 最大条高度（像素）
    pub max_bar_height: f32,
    /// 是否显示 Y 轴刻度
    pub show_y_axis: bool,
    /// 是否显示网格线
    pub show_grid_lines: bool,
    /// 是否显示悬停高亮
    pub show_hover_highlight: bool,
}

impl Default for StackedBarChartConfig {
    fn default() -> Self {
        Self {
            color_map: CategoryColorMap::default(),
            show_legend: true,
            max_bar_height: 200.0,
            show_y_axis: true,
            show_grid_lines: true,
            show_hover_highlight: true,
        }
    }
}

/// 堆叠柱形图组件
pub struct StackedBarChart<'a> {
    /// 图表数据
    pub data: &'a ChartData,
    /// 主题
    pub theme: &'a TaiLTheme,
    /// 配置
    pub config: StackedBarChartConfig,
}

impl<'a> StackedBarChart<'a> {
    pub fn new(data: &'a ChartData, theme: &'a TaiLTheme) -> Self {
        Self {
            data,
            theme,
            config: StackedBarChartConfig::default(),
        }
    }

    pub fn with_config(mut self, config: StackedBarChartConfig) -> Self {
        self.config = config;
        self
    }

    /// 显示堆叠柱形图，返回悬停的时间槽索引（如果有）
    pub fn show(&self, ui: &mut Ui) -> Option<usize> {
        if self.data.time_slots.is_empty() {
            ui.label("暂无数据");
            return None;
        }

        let mut hovered_slot = None;

        // 计算最大时长（用于归一化柱子高度）
        let max_seconds = self.data.max_seconds().max(60); // 最少1分钟作为最大值

        // Y 轴刻度配置
        let y_axis_width = if self.config.show_y_axis { 45.0 } else { 0.0 };
        let y_tick_count = 5;

        // Y 轴时间格式化函数 - 使用统一的时间格式化模块
        let format_y_tick = |seconds: i64| -> String {
            tail_core::time::format::TimeFormatter::format_y_axis(seconds)
        };

        // 计算 Y 轴刻度值
        let y_ticks: Vec<i64> = (0..y_tick_count)
            .map(|i| max_seconds * i / (y_tick_count - 1))
            .collect();

        // 获取所有分组并分配颜色
        let all_groups = self.data.all_groups();
        let group_colors = self.config.color_map.assign_colors(&all_groups);

        // 根据时间粒度确定柱子宽度
        let (bar_width, bar_gap) = self.calculate_bar_sizes();

        let total_chart_width = y_axis_width
            + bar_width * self.data.time_slots.len() as f32
            + bar_gap * (self.data.time_slots.len() as f32 - 1.0);

        ui.vertical(|ui| {
            // 图例区域
            if self.config.show_legend && !all_groups.is_empty() {
                self.show_legend(ui, &all_groups, &group_colors);
            }

            // 柱状图区域
            let available_width = ui.available_width();
            let chart_height = self.config.max_bar_height;

            // Y轴位置信息，用于后续绘制X轴
            let mut y_axis_start_x = 0.0;
            let mut chart_start_y = 0.0;

            ui.horizontal(|ui| {
                // 计算居中偏移
                let offset_x = (available_width - total_chart_width) / 2.0;
                if offset_x > 0.0 {
                    ui.add_space(offset_x);
                }

                // Y 轴区域
                y_axis_start_x = if self.config.show_y_axis {
                    self.show_y_axis(ui, chart_height, &y_ticks, format_y_tick);
                    ui.cursor().min.x
                } else {
                    ui.cursor().min.x
                };

                chart_start_y = ui.cursor().min.y;

                // 绘制水平网格线
                if self.config.show_grid_lines {
                    self.draw_grid_lines(
                        ui,
                        y_axis_start_x,
                        chart_start_y,
                        chart_height,
                        bar_width,
                        bar_gap,
                        &y_ticks,
                        y_tick_count as usize,
                    );
                }

                // 绘制柱子
                for (idx, slot) in self.data.time_slots.iter().enumerate() {
                    let result = self.draw_bar(
                        ui,
                        slot,
                        idx,
                        y_axis_start_x,
                        chart_start_y,
                        chart_height,
                        max_seconds,
                        bar_width,
                        bar_gap,
                        &group_colors,
                    );

                    if result.hovered {
                        hovered_slot = Some(idx);
                    }
                }
            });

            // 在水平布局外绘制 X 轴标签（确保在柱形图下方）
            self.show_x_axis(
                ui,
                y_axis_start_x,
                chart_start_y,
                chart_height,
                bar_width,
                bar_gap,
            );
        });

        hovered_slot
    }

    /// 根据时间粒度计算柱子尺寸
    fn calculate_bar_sizes(&self) -> (f32, f32) {
        match self.data.granularity {
            ChartTimeGranularity::Year => (24.0, 12.0),
            ChartTimeGranularity::Month => (30.0, 10.0),
            ChartTimeGranularity::Week => (40.0, 15.0),
            ChartTimeGranularity::Day => (18.0, 6.0),
            ChartTimeGranularity::Hour => (4.0, 2.0),
        }
    }

    /// 显示图例
    fn show_legend(
        &self,
        ui: &mut Ui,
        all_groups: &[String],
        group_colors: &HashMap<String, Color32>,
    ) {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 12.0;
            let legend_groups = all_groups.iter().take(12);

            for group in legend_groups {
                let color = group_colors
                    .get(group)
                    .copied()
                    .unwrap_or(self.config.color_map.other_color());
                ui.horizontal(|ui| {
                    let size = Vec2::new(12.0, 12.0);
                    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
                    ui.painter().rect_filled(rect, Rounding::same(3.0), color);
                    ui.add_space(6.0);

                    ui.label(
                        egui::RichText::new(group)
                            .size(self.theme.small_size)
                            .color(self.theme.text_color),
                    );
                });
            }
        });
        ui.add_space(8.0);
    }

    /// 显示 Y 轴
    fn show_y_axis(
        &self,
        ui: &mut Ui,
        chart_height: f32,
        y_ticks: &[i64],
        format_y_tick: impl Fn(i64) -> String,
    ) -> f32 {
        let y_axis_width = 45.0;
        let start_y = ui.cursor().min.y;

        // 预留空间
        ui.allocate_space(Vec2::new(y_axis_width, chart_height));

        let painter = ui.painter();

        // 从下往上绘制标签，与网格线位置对齐
        for (i, &tick_seconds) in y_ticks.iter().enumerate() {
            let ratio = i as f32 / (y_ticks.len() - 1) as f32;
            // 从底部向上计算位置
            let y_pos = start_y + chart_height - ratio * chart_height;

            painter.text(
                Pos2::new(ui.cursor().min.x, y_pos),
                egui::Align2::RIGHT_CENTER,
                format_y_tick(tick_seconds),
                egui::FontId::proportional(self.theme.small_size),
                self.theme.secondary_text_color,
            );
        }

        y_axis_width
    }

    /// 绘制网格线
    #[allow(clippy::too_many_arguments)]
    fn draw_grid_lines(
        &self,
        ui: &mut Ui,
        start_x: f32,
        start_y: f32,
        chart_height: f32,
        bar_width: f32,
        bar_gap: f32,
        y_ticks: &[i64],
        y_tick_count: usize,
    ) {
        let total_width = bar_width * self.data.time_slots.len() as f32
            + bar_gap * (self.data.time_slots.len() as f32 - 1.0);

        for (i, _tick_seconds) in y_ticks.iter().enumerate().skip(1) {
            let ratio = i as f32 / (y_tick_count - 1) as f32;
            let y_pos = start_y + chart_height - ratio * chart_height;
            let line_start = Pos2::new(start_x, y_pos);
            let line_end = Pos2::new(start_x + total_width, y_pos);

            ui.painter().line_segment(
                [line_start, line_end],
                Stroke::new(1.0, Color32::from_gray(40).gamma_multiply(0.3)),
            );
        }
    }

    /// 绘制单个柱子
    #[allow(clippy::too_many_arguments)]
    fn draw_bar(
        &self,
        ui: &mut Ui,
        slot: &super::chart_data::ChartTimeSlot,
        idx: usize,
        start_x: f32,
        start_y: f32,
        chart_height: f32,
        max_seconds: i64,
        bar_width: f32,
        bar_gap: f32,
        group_colors: &HashMap<String, Color32>,
    ) -> BarDrawResult {
        // 柱子高度 = (该小时总时长 / 最大时长) * 图表高度
        // 如果没有数据，使用最小高度2像素
        let bar_height = if slot.total_seconds > 0 {
            (slot.total_seconds as f32 / max_seconds as f32) * chart_height
        } else {
            2.0 // 空柱子，显示有这个位置
        };

        // 柱子从底部向上：y = 底部位置 - 柱子高度
        let x = start_x + idx as f32 * (bar_width + bar_gap);
        let bottom_y = start_y + chart_height; // 底部Y坐标
        let top_y = bottom_y - bar_height; // 顶部Y坐标
        let rect = Rect::from_min_size(Pos2::new(x, top_y), Vec2::new(bar_width, bar_height));

        let response = ui.allocate_rect(rect, Sense::hover());

        // 绘制堆叠柱子
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // 先绘制柱子背景（空的柱子也能看到位置）
            if slot.total_seconds > 0 {
                let mut current_y = bottom_y;

                // 按时长从大到小排序分组（大的在底部）
                let mut groups: Vec<_> = slot.group_durations.iter().collect();
                groups.sort_by(|a, b| b.1.cmp(a.1));

                for (group, &seconds) in groups {
                    if seconds <= 0 {
                        continue;
                    }

                    // 该应用段的高度 = (该应用时长 / 总时长) * 柱子高度
                    let segment_height = (seconds as f32 / slot.total_seconds as f32) * bar_height;
                    let segment_top_y = current_y - segment_height;

                    let color = group_colors
                        .get(&**group)
                        .copied()
                        .unwrap_or(self.config.color_map.other_color());

                    let segment_rect = Rect::from_min_size(
                        Pos2::new(x, segment_top_y),
                        Vec2::new(bar_width, segment_height.max(1.0)),
                    );
                    painter.rect_filled(segment_rect, Rounding::same(2.0), color);

                    current_y = segment_top_y;
                }
            } else {
                // 空柱子，绘制浅灰色背景
                painter.rect_filled(rect, Rounding::same(2.0), self.theme.progress_background);
            }

            // 悬停效果（可选）
            if self.config.show_hover_highlight && response.hovered() {
                painter.rect_stroke(
                    rect,
                    Rounding::same(4.0),
                    Stroke::new(2.0, self.theme.primary_color),
                );
            }
        }

        BarDrawResult {
            hovered: response.hovered(),
        }
    }

    /// 显示 X 轴标签
    fn show_x_axis(
        &self,
        ui: &mut Ui,
        start_x: f32,
        start_y: f32,
        chart_height: f32,
        bar_width: f32,
        bar_gap: f32,
    ) {
        // X轴标签在柱形图底部下方
        let bottom_y = start_y + chart_height;
        let label_y = bottom_y + 8.0; // 标签与柱形图底部之间的间距

        let painter = ui.painter();

        // 根据时间粒度决定显示哪些标签
        let label_indices = self.get_label_indices(self.data.time_slots.len());

        eprintln!(
            "[DEBUG] show_x_axis - label_indices.len()={}, granularity={:?}",
            label_indices.len(),
            self.data.granularity
        );

        for &idx in &label_indices {
            if let Some(slot) = self.data.time_slots.get(idx) {
                // 跳过空标签
                if slot.label.is_empty() {
                    continue;
                }

                // 计算柱子中心位置（与 draw_bar 中的计算一致）
                let bar_center_x = start_x + idx as f32 * (bar_width + bar_gap) + bar_width / 2.0;

                // 绘制标签，居中对齐到柱子中心
                painter.text(
                    Pos2::new(bar_center_x, label_y),
                    egui::Align2::CENTER_TOP,
                    &slot.label,
                    egui::FontId::proportional(self.theme.small_size),
                    self.theme.secondary_text_color,
                );
            }
        }
    }

    /// 根据时间粒度获取需要显示的标签索引
    fn get_label_indices(&self, slot_count: usize) -> Vec<usize> {
        match self.data.granularity {
            ChartTimeGranularity::Hour => {
                // 每10分钟显示一个标签
                (0..slot_count).step_by(10).collect()
            }
            ChartTimeGranularity::Day => {
                // 每6小时显示一个标签
                vec![0, 6, 12, 18]
            }
            ChartTimeGranularity::Week => {
                // 显示所有标签
                (0..slot_count).collect()
            }
            ChartTimeGranularity::Month => {
                // 显示所有标签
                (0..slot_count).collect()
            }
            ChartTimeGranularity::Year => {
                // 每月显示一个标签
                (0..slot_count).collect()
            }
        }
    }
}

struct BarDrawResult {
    hovered: bool,
}

/// 堆叠柱形图的 Hover 提示内容
pub struct StackedBarTooltip<'a> {
    pub slot: &'a super::chart_data::ChartTimeSlot,
}

impl<'a> StackedBarTooltip<'a> {
    pub fn new(slot: &'a super::chart_data::ChartTimeSlot) -> Self {
        Self { slot }
    }

    pub fn show(&self, ui: &mut Ui, theme: &TaiLTheme) {
        let mouse_pos = ui
            .input(|i| i.pointer.hover_pos())
            .unwrap_or(Pos2::new(0.0, 0.0));

        let tooltip_width = 200.0;
        let tooltip_height = 80.0 + self.slot.top_groups(6).len() as f32 * 20.0;

        let mut rect = Rect::from_center_size(
            mouse_pos + Vec2::new(tooltip_width / 2.0 + 10.0, 0.0),
            Vec2::new(tooltip_width, tooltip_height),
        );

        // 确保不超出屏幕
        let screen_rect = ui.ctx().screen_rect();
        if rect.max.x > screen_rect.max.x {
            rect = rect.translate(Vec2::new(screen_rect.max.x - rect.max.x - 10.0, 0.0));
        }
        if rect.max.y > screen_rect.max.y {
            rect = rect.translate(Vec2::new(0.0, screen_rect.max.y - rect.max.y - 10.0));
        }

        // 使用 Area 绘制 tooltip
        egui::Area::new(egui::Id::new("stacked_bar_tooltip_v2"))
            .movable(false)
            .pivot(egui::Align2::LEFT_TOP)
            .fixed_pos(rect.min)
            .show(ui.ctx(), |ui| {
                ui.allocate_painter(rect.size(), Sense::hover());

                let painter = ui.painter();
                let bg_rect = Rect::from_min_size(Pos2::ZERO, rect.size());
                painter.rect_filled(bg_rect, Rounding::same(8.0), Color32::from_black_alpha(230));
                painter.rect_stroke(
                    bg_rect,
                    Rounding::same(8.0),
                    Stroke::new(1.0, theme.divider_color),
                );

                // 标题
                painter.text(
                    Pos2::new(12.0, 12.0),
                    egui::Align2::LEFT_TOP,
                    &self.slot.label,
                    egui::FontId::proportional(theme.body_size),
                    Color32::WHITE,
                );

                // 总时长
                painter.text(
                    Pos2::new(12.0, 36.0),
                    egui::Align2::LEFT_TOP,
                    crate::utils::duration::format_duration(self.slot.total_seconds),
                    egui::FontId::proportional(theme.heading_size),
                    Color32::WHITE,
                );

                // Top 分组列表
                let mut y_offset = 62.0;
                for (group, secs) in self.slot.top_groups(6) {
                    painter.text(
                        Pos2::new(12.0, y_offset),
                        egui::Align2::LEFT_TOP,
                        format!(
                            "• {} - {}",
                            group,
                            crate::utils::duration::format_duration(secs)
                        ),
                        egui::FontId::proportional(theme.small_size),
                        Color32::from_gray(200),
                    );
                    y_offset += 18.0;
                }
            });
    }
}
