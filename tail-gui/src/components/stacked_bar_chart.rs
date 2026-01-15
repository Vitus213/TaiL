//! 堆叠柱状图组件 - 仿 iPhone 屏幕使用时间风格
//!
//! X 轴：时间刻度（00:00 - 23:59）
//! Y 轴：时长（堆叠）
//! 每个色块代表应用或分类

use egui::{Color32, Pos2, Rect, Sense, Ui, Vec2};
use std::collections::HashMap;

use crate::theme::TaiLTheme;

/// 时间段数据
#[derive(Debug, Clone)]
pub struct TimeSlotData {
    /// 小时 (0-23)
    pub hour: u32,
    /// 该小时内各应用的时长（应用名 -> 秒数）
    pub app_durations: HashMap<String, i64>,
    /// 总时长
    pub total_seconds: i64,
}

impl TimeSlotData {
    pub fn new(hour: u32) -> Self {
        Self {
            hour,
            app_durations: HashMap::new(),
            total_seconds: 0,
        }
    }

    pub fn add_app(&mut self, app_name: String, seconds: i64) {
        self.total_seconds += seconds;
        *self.app_durations.entry(app_name).or_insert(0) += seconds;
    }
}

/// 堆叠柱状图配置
pub struct StackedBarChartConfig {
    /// 每个分类对应的颜色
    pub category_colors: HashMap<String, Color32>,
    /// "其他"分类的颜色
    pub other_color: Color32,
    /// 是否显示图例
    pub show_legend: bool,
    /// 最大条高度（像素）
    pub max_bar_height: f32,
}

impl Default for StackedBarChartConfig {
    fn default() -> Self {
        let mut category_colors = HashMap::new();
        category_colors.insert("工作".to_string(), Color32::from_rgb(74, 144, 226)); // 蓝色
        category_colors.insert("开发".to_string(), Color32::from_rgb(52, 168, 83)); // 青色
        category_colors.insert("娱乐".to_string(), Color32::from_rgb(255, 99, 71)); // 橙红色
        category_colors.insert("社交".to_string(), Color32::from_rgb(155, 89, 182)); // 紫色
        category_colors.insert("学习".to_string(), Color32::from_rgb(255, 205, 86)); // 黄色

        Self {
            category_colors,
            other_color: Color32::from_gray(150),
            show_legend: true,
            max_bar_height: 200.0,
        }
    }
}

/// 堆叠柱状图组件
pub struct StackedBarChart<'a> {
    /// 时间段数据（24小时）
    pub time_slots: &'a [TimeSlotData],
    /// 主题
    pub theme: &'a TaiLTheme,
    /// 配置
    pub config: StackedBarChartConfig,
}

impl<'a> StackedBarChart<'a> {
    pub fn new(time_slots: &'a [TimeSlotData], theme: &'a TaiLTheme) -> Self {
        Self {
            time_slots,
            theme,
            config: StackedBarChartConfig::default(),
        }
    }

    pub fn with_config(mut self, config: StackedBarChartConfig) -> Self {
        self.config = config;
        self
    }

    /// 显示堆叠柱状图，返回悬停的柱子索引（如果有）
    pub fn show(&self, ui: &mut Ui) -> Option<usize> {
        let mut hovered_slot = None;

        // 计算最大时长（用于归一化柱子高度）
        let max_seconds = self
            .time_slots
            .iter()
            .map(|slot| slot.total_seconds)
            .max()
            .unwrap_or(3600)
            .max(1800); // 最少30分钟作为最大值，避免全是小数据时柱子太满

        // Y 轴刻度配置
        let y_axis_width = 45.0;
        let y_tick_count = 5;

        // Y 轴时间格式化函数
        let format_y_tick = |seconds: i64| -> String {
            if seconds < 60 {
                format!("{}m", seconds)
            } else if seconds < 3600 {
                let mins = seconds / 60;
                format!("{}m", mins)
            } else {
                let hours = seconds as f32 / 3600.0;
                if hours.fract() < 0.1 || hours.fract() > 0.9 {
                    format!("{}h", hours.round() as i32)
                } else if hours.fract() < 0.6 {
                    format!("{}h", hours as i32)
                } else {
                    format!("{:.1}h", hours)
                }
            }
        };

        // 计算 Y 轴刻度值
        let y_ticks: Vec<i64> = (0..y_tick_count)
            .map(|i| max_seconds * i / (y_tick_count - 1))
            .collect();

        // 获取所有出现过的应用（用于分配颜色）
        let mut all_apps: Vec<String> = self
            .time_slots
            .iter()
            .flat_map(|slot| slot.app_durations.keys())
            .cloned()
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        // 排序以确保颜色分配的一致性（避免闪烁）
        all_apps.sort();

        // 为每个应用分配颜色
        let mut app_colors: HashMap<String, Color32> = HashMap::new();
        let default_colors = [
            Color32::from_rgb(74, 144, 226), // 蓝
            Color32::from_rgb(52, 168, 83),  // 青
            Color32::from_rgb(255, 205, 86), // 黄
            Color32::from_rgb(255, 99, 71),  // 橙
            Color32::from_rgb(155, 89, 182), // 紫
            Color32::from_rgb(255, 152, 0),  // 橙黄
            Color32::from_rgb(220, 57, 218), // 粉紫
            Color32::from_rgb(0, 200, 150),  // 青绿
        ];

        for (idx, app) in all_apps.iter().enumerate() {
            let color = self
                .config
                .category_colors
                .get(app)
                .copied()
                .unwrap_or_else(|| {
                    default_colors
                        .get(idx % default_colors.len())
                        .copied()
                        .unwrap_or(self.config.other_color)
                });
            app_colors.insert(app.clone(), color);
        }

        // 主容器
        let available_width = ui.available_width();
        let chart_height = self.config.max_bar_height;
        let bar_width = 18.0;
        let bar_gap = 6.0;
        let total_chart_width = y_axis_width + bar_width * 24.0 + bar_gap * 23.0;

        ui.vertical(|ui| {
            // 图例区域
            if self.config.show_legend && !all_apps.is_empty() {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing.x = 12.0;
                    let legend_apps = all_apps.iter().take(8); // 最多显示8个图例

                    for app in legend_apps {
                        let color = app_colors
                            .get(app)
                            .copied()
                            .unwrap_or(self.config.other_color);
                        ui.horizontal(|ui| {
                            // 颜色方块 - 使用 allocate_ui 确保正确布局
                            let size = Vec2::new(12.0, 12.0);
                            let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
                            ui.painter().rect_filled(
                                rect,
                                egui::Rounding::same(3.0),
                                color,
                            );
                            ui.add_space(6.0);

                            // 应用名
                            ui.label(
                                egui::RichText::new(app)
                                    .size(self.theme.small_size)
                                    .color(self.theme.text_color),
                            );
                        });
                    }
                });
                ui.add_space(8.0);
            }

            // 柱状图区域 - 水平居中
            ui.horizontal(|ui| {
                // 计算居中偏移
                let offset_x = (available_width - total_chart_width) / 2.0;
                if offset_x > 0.0 {
                    ui.add_space(offset_x);
                }

                // Y 轴区域
                ui.vertical(|ui| {
                    ui.set_width(y_axis_width);

                    let _chart_base_y = ui.cursor().min.y + chart_height;

                    // 绘制 Y 轴刻度标签和网格线
                    for &tick_seconds in y_ticks.iter() {
                        let ratio = y_ticks.iter().position(|&x| x == tick_seconds).unwrap() as f32 / (y_tick_count - 1) as f32;
                        let _y_pos = _chart_base_y - ratio * chart_height;

                        // 刻度标签
                        ui.label(
                            egui::RichText::new(format_y_tick(tick_seconds))
                                .size(self.theme.small_size)
                                .color(self.theme.secondary_text_color),
                        );

                        // 网格线（使用 painter 在柱状图区域绘制）
                    }
                });

                let start_x = ui.cursor().min.x;
                let start_y = ui.cursor().min.y;

                // 绘制水平网格线
                for (i, _tick_seconds) in y_ticks.iter().enumerate().skip(1) {
                    let ratio = i as f32 / (y_tick_count - 1) as f32;
                    let y_pos = start_y + chart_height - ratio * chart_height;
                    let line_start = Pos2::new(start_x, y_pos);
                    let line_end = Pos2::new(start_x + bar_width * 24.0 + bar_gap * 23.0, y_pos);

                    ui.painter().line_segment(
                        [line_start, line_end],
                        egui::Stroke::new(1.0, Color32::from_gray(40).gamma_multiply(0.3)),
                    );
                }

                for (idx, slot) in self.time_slots.iter().enumerate() {
                    let bar_height = if slot.total_seconds > 0 {
                        (slot.total_seconds as f32 / max_seconds as f32) * chart_height
                    } else {
                        2.0 // 最小高度，显示有活动
                    };

                    let x = start_x + idx as f32 * (bar_width + bar_gap);
                    let y = start_y + chart_height - bar_height;
                    let rect =
                        Rect::from_min_size(Pos2::new(x, y), Vec2::new(bar_width, bar_height));

                    let response = ui.allocate_rect(rect, Sense::hover());

                    // 绘制堆叠柱子
                    if ui.is_rect_visible(rect) {
                        let painter = ui.painter();
                        let mut current_y = rect.max.y;

                        // 按时长从大到小排序应用（确保大的在底部）
                        let mut apps: Vec<_> = slot.app_durations.iter().collect();
                        apps.sort_by(|a, b| b.1.cmp(a.1));

                        for (app, &seconds) in apps {
                            if seconds <= 0 {
                                continue;
                            }

                            let segment_height =
                                (seconds as f32 / slot.total_seconds as f32) * bar_height;
                            let segment_y = current_y - segment_height;

                            let color = app_colors
                                .get(&**app)
                                .copied()
                                .unwrap_or(self.config.other_color);

                            let segment_rect = Rect::from_min_size(
                                Pos2::new(rect.min.x, segment_y),
                                Vec2::new(bar_width, segment_height.max(1.0)),
                            );
                            painter.rect_filled(segment_rect, egui::Rounding::same(2.0), color);

                            current_y = segment_y;
                        }

                        // 悬停效果
                        if response.hovered() {
                            painter.rect_stroke(
                                rect,
                                egui::Rounding::same(4.0),
                                egui::Stroke::new(2.0, self.theme.primary_color),
                            );
                            hovered_slot = Some(idx);
                        }
                    }
                }
            });

            // X 轴标签
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                // 计算居中偏移（包含 Y 轴宽度）
                let offset_x = (available_width - total_chart_width) / 2.0;
                if offset_x > 0.0 {
                    ui.add_space(offset_x + y_axis_width);
                }

                let label_spacing = total_chart_width / 24.0;
                for hour in [0, 6, 12, 18] {
                    ui.label(
                        egui::RichText::new(format!("{}h", hour))
                            .size(self.theme.small_size)
                            .color(self.theme.secondary_text_color),
                    );
                    ui.add_space(label_spacing);
                }
            });
        });

        hovered_slot
    }
}

/// 堆叠柱状图的 Hover 提示内容
pub struct StackedBarTooltip {
    pub hour: u32,
    pub total_seconds: i64,
    pub top_apps: Vec<(String, i64)>,
}

impl StackedBarTooltip {
    pub fn show(&self, ui: &mut Ui, theme: &TaiLTheme) {
        let mouse_pos = ui
            .input(|i| i.pointer.hover_pos())
            .unwrap_or(Pos2::new(0.0, 0.0));

        let tooltip_width = 180.0;
        let tooltip_height = 80.0 + self.top_apps.len() as f32 * 20.0;

        let mut rect = Rect::from_center_size(
            mouse_pos + Vec2::new(tooltip_width / 2.0 + 10.0, 0.0),
            Vec2::new(tooltip_width, tooltip_height),
        );

        // 确保不超出屏幕
        let screen_rect = ui.ctx().screen_rect();
        if rect.max.x > screen_rect.max.x {
            rect = rect.translate(Vec2::new(screen_rect.max.x - rect.max.x, 0.0));
        }
        if rect.max.y > screen_rect.max.y {
            rect = rect.translate(Vec2::new(0.0, screen_rect.max.y - rect.max.y));
        }

        // 使用 Area 绘制 tooltip
        egui::Area::new(egui::Id::new("stacked_bar_tooltip"))
            .movable(false)
            .pivot(egui::Align2::LEFT_TOP)
            .fixed_pos(rect.min)
            .show(ui.ctx(), |ui| {
                ui.allocate_painter(rect.size(), egui::Sense::hover());

                // 绘制半透明背景
                let painter = ui.painter();
                let bg_rect = Rect::from_min_size(Pos2::ZERO, rect.size());
                painter.rect_filled(
                    bg_rect,
                    egui::Rounding::same(8.0),
                    Color32::from_black_alpha(230),
                );
                painter.rect_stroke(
                    bg_rect,
                    egui::Rounding::same(8.0),
                    egui::Stroke::new(1.0, theme.divider_color),
                );

                // 标题
                painter.text(
                    Pos2::new(12.0, 12.0),
                    egui::Align2::LEFT_TOP,
                    format!("{}时 - 总计", self.hour),
                    egui::FontId::proportional(theme.body_size),
                    Color32::WHITE,
                );

                // 总时长
                painter.text(
                    Pos2::new(12.0, 36.0),
                    egui::Align2::LEFT_TOP,
                    crate::utils::duration::format_duration(self.total_seconds),
                    egui::FontId::proportional(theme.heading_size),
                    Color32::WHITE,
                );

                // Top 应用列表
                let mut y_offset = 62.0;
                for (app, secs) in self.top_apps.iter().take(4) {
                    painter.text(
                        Pos2::new(12.0, y_offset),
                        egui::Align2::LEFT_TOP,
                        format!(
                            "• {} - {}",
                            app,
                            crate::utils::duration::format_duration(*secs)
                        ),
                        egui::FontId::proportional(theme.small_size),
                        Color32::from_gray(200),
                    );
                    y_offset += 18.0;
                }
            });
    }
}
