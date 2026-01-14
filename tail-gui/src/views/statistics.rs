//! TaiL GUI - 统计视图

use egui::{Ui, Color32, Pos2, Rect, Vec2, Rounding};
use egui_extras::{TableBuilder, Column};
use tail_core::AppUsage;
use tail_core::models::TimeRange;
use chrono::{Timelike, Datelike, Local};

use crate::components::{PageHeader, TimeRangeSelector, SectionDivider, EmptyState};
use crate::icons::IconCache;
use crate::theme::TaiLTheme;

/// 统计视图
pub struct StatisticsView<'a> {
    /// 应用使用数据
    app_usage: &'a [AppUsage],
    /// 当前时间范围
    time_range: TimeRange,
    /// 主题
    theme: &'a TaiLTheme,
    /// 图标缓存（可变引用以支持加载图标）
    icon_cache: &'a mut IconCache,
}

impl<'a> StatisticsView<'a> {
    pub fn new(
        app_usage: &'a [AppUsage],
        time_range: TimeRange,
        theme: &'a TaiLTheme,
        icon_cache: &'a mut IconCache,
    ) -> Self {
        Self {
            app_usage,
            time_range,
            theme,
            icon_cache,
        }
    }

    /// 渲染统计视图，返回新选择的时间范围（如果有变化）
    pub fn show(&mut self, ui: &mut Ui) -> Option<TimeRange> {
        let mut new_time_range = None;

        // 页面标题
        ui.add(PageHeader::new("详细统计", "📈", self.theme)
            .subtitle("查看应用使用详情"));
        
        ui.add_space(self.theme.spacing);

        // 时间范围选择器
        let selector_response = TimeRangeSelector::new(self.time_range, self.theme).show(ui);
        if let Some(selected) = selector_response.selected {
            new_time_range = Some(selected);
        }

        ui.add_space(self.theme.spacing);

        // 时间分布图（可点击）
        ui.add(SectionDivider::new(self.theme).with_title("时间分布 (点击柱子查看详情)"));
        ui.add_space(self.theme.spacing / 2.0);
        if let Some(clicked_range) = self.show_time_distribution(ui) {
            new_time_range = Some(clicked_range);
        }

        ui.add_space(self.theme.spacing);

        // 应用详情表格
        ui.add(SectionDivider::new(self.theme).with_title("应用详情"));
        ui.add_space(self.theme.spacing / 2.0);
        self.show_app_table(ui);

        new_time_range
    }

    /// 显示时间分布图（柱状图）
    /// 根据时间范围选择不同的显示方式：
    /// - 今天/昨天：显示24小时分布
    /// - 7天/30天：显示按天分布，点击柱子可以切换到该天
    fn show_time_distribution(&self, ui: &mut Ui) -> Option<TimeRange> {
        let desired_size = Vec2::new(ui.available_width(), 150.0);
        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

        let mut clicked_range = None;

        if ui.is_rect_visible(rect) {
            // 根据时间范围选择显示方式
            match self.time_range {
                TimeRange::Today | TimeRange::Yesterday => {
                    self.draw_hourly_chart(ui, rect);
                    // 今天/昨天的小时图不支持点击切换
                }
                TimeRange::Last7Days => {
                    clicked_range = self.draw_daily_chart_interactive(ui, rect, 7, &response);
                }
                TimeRange::Last30Days => {
                    clicked_range = self.draw_daily_chart_interactive(ui, rect, 30, &response);
                }
                TimeRange::Custom(_, _) => {
                    // 自定义范围默认使用按天显示
                    self.draw_daily_chart(ui, rect, 30);
                }
            }
        }

        clicked_range
    }

    /// 绘制24小时分布图
    fn draw_hourly_chart(&self, ui: &mut Ui, rect: Rect) {
        let painter = ui.painter();
        
        // 背景
        painter.rect_filled(
            rect,
            Rounding::same(self.theme.card_rounding),
            self.theme.card_background,
        );

        let padding = self.theme.card_padding;
        let content_rect = rect.shrink(padding);

        // 计算每小时的使用时间
        let mut hourly_usage = [0i64; 24];
        let mut max_usage = 0i64;

        for usage in self.app_usage {
            for event in &usage.window_events {
                let hour = event.timestamp.hour() as usize;
                if hour < 24 {
                    hourly_usage[hour] += event.duration_secs;
                    max_usage = max_usage.max(hourly_usage[hour]);
                }
            }
        }

        // 绘制柱状图
        let bar_gap = 4.0;
        let bar_width = (content_rect.width() - 23.0 * bar_gap) / 24.0;
        let chart_height = content_rect.height() - 30.0;
        let chart_bottom = content_rect.max.y - 20.0;

        for (hour, &usage) in hourly_usage.iter().enumerate() {
            let bar_height = if max_usage > 0 {
                (usage as f32 / max_usage as f32) * chart_height
            } else {
                0.0
            };

            let bar_x = content_rect.min.x + hour as f32 * (bar_width + bar_gap);
            let bar_rect = Rect::from_min_size(
                Pos2::new(bar_x, chart_bottom - bar_height),
                Vec2::new(bar_width, bar_height.max(2.0)),
            );

            // 根据使用量选择颜色
            let color = if usage > max_usage * 3 / 4 {
                self.theme.primary_color
            } else if usage > max_usage / 2 {
                self.theme.primary_color.linear_multiply(0.7)
            } else if usage > 0 {
                self.theme.primary_color.linear_multiply(0.4)
            } else {
                self.theme.divider_color
            };

            painter.rect_filled(bar_rect, Rounding::same(2.0), color);

            // 小时标签（每隔3小时显示）
            if hour % 3 == 0 {
                painter.text(
                    Pos2::new(bar_x + bar_width / 2.0, chart_bottom + 10.0),
                    egui::Align2::CENTER_CENTER,
                    format!("{}时", hour),
                    egui::FontId::proportional(self.theme.small_size - 2.0),
                    self.theme.secondary_text_color,
                );
            }
        }

        // Y轴标签
        if max_usage > 0 {
            let max_label = Self::format_duration_short(max_usage);
            painter.text(
                Pos2::new(content_rect.min.x, content_rect.min.y + 5.0),
                egui::Align2::LEFT_TOP,
                max_label,
                egui::FontId::proportional(self.theme.small_size - 2.0),
                self.theme.secondary_text_color,
            );
        }
    }

    /// 绘制按天分布图（非交互式，用于自定义范围）
    fn draw_daily_chart(&self, ui: &mut Ui, rect: Rect, days: usize) {
        let painter = ui.painter();
        
        // 背景
        painter.rect_filled(
            rect,
            Rounding::same(self.theme.card_rounding),
            self.theme.card_background,
        );

        let padding = self.theme.card_padding;
        let content_rect = rect.shrink(padding);

        use std::collections::HashMap;
        
        // 计算每天的使用时间
        let mut daily_usage: HashMap<u32, i64> = HashMap::new();
        let mut max_usage = 0i64;

        for usage in self.app_usage {
            for event in &usage.window_events {
                let day = event.timestamp.ordinal(); // 一年中的第几天
                let entry = daily_usage.entry(day).or_insert(0);
                *entry += event.duration_secs;
                max_usage = max_usage.max(*entry);
            }
        }

        // 获取最近 N 天的日期（使用本地时间）
        let today = Local::now();
        let mut day_labels: Vec<(u32, String)> = Vec::new();
        
        for i in 0..days {
            let date = today - chrono::Duration::days(i as i64);
            let ordinal = date.ordinal();
            let label = if days <= 7 {
                // 7天内显示星期几
                let weekday = date.weekday();
                match weekday {
                    chrono::Weekday::Mon => "周一",
                    chrono::Weekday::Tue => "周二",
                    chrono::Weekday::Wed => "周三",
                    chrono::Weekday::Thu => "周四",
                    chrono::Weekday::Fri => "周五",
                    chrono::Weekday::Sat => "周六",
                    chrono::Weekday::Sun => "周日",
                }.to_string()
            } else {
                // 30天显示日期
                format!("{}/{}", date.month(), date.day())
            };
            day_labels.push((ordinal, label));
        }
        
        // 反转使其从旧到新排列
        day_labels.reverse();

        // 绘制柱状图
        let bar_gap = if days <= 7 { 8.0 } else { 2.0 };
        let bar_width = (content_rect.width() - (days - 1) as f32 * bar_gap) / days as f32;
        let chart_height = content_rect.height() - 30.0;
        let chart_bottom = content_rect.max.y - 20.0;

        for (idx, (ordinal, label)) in day_labels.iter().enumerate() {
            let usage = daily_usage.get(ordinal).copied().unwrap_or(0);
            
            let bar_height = if max_usage > 0 {
                (usage as f32 / max_usage as f32) * chart_height
            } else {
                0.0
            };

            let bar_x = content_rect.min.x + idx as f32 * (bar_width + bar_gap);
            let bar_rect = Rect::from_min_size(
                Pos2::new(bar_x, chart_bottom - bar_height),
                Vec2::new(bar_width, bar_height.max(2.0)),
            );

            // 根据使用量选择颜色
            let color = if usage > max_usage * 3 / 4 {
                self.theme.primary_color
            } else if usage > max_usage / 2 {
                self.theme.primary_color.linear_multiply(0.7)
            } else if usage > 0 {
                self.theme.primary_color.linear_multiply(0.4)
            } else {
                self.theme.divider_color
            };

            painter.rect_filled(bar_rect, Rounding::same(2.0), color);

            // 日期标签
            let show_label = if days <= 7 {
                true // 7天内全部显示
            } else {
                idx % 5 == 0 || idx == days - 1 // 30天每5天显示一次
            };
            
            if show_label {
                painter.text(
                    Pos2::new(bar_x + bar_width / 2.0, chart_bottom + 10.0),
                    egui::Align2::CENTER_CENTER,
                    label,
                    egui::FontId::proportional(self.theme.small_size - 2.0),
                    self.theme.secondary_text_color,
                );
            }
        }

        // Y轴标签
        if max_usage > 0 {
            let max_label = Self::format_duration_short(max_usage);
            painter.text(
                Pos2::new(content_rect.min.x, content_rect.min.y + 5.0),
                egui::Align2::LEFT_TOP,
                max_label,
                egui::FontId::proportional(self.theme.small_size - 2.0),
                self.theme.secondary_text_color,
            );
        }
    }

    /// 绘制按天分布图（交互式，支持点击切换日期）
    fn draw_daily_chart_interactive(
        &self,
        ui: &mut Ui,
        rect: Rect,
        days: usize,
        response: &egui::Response,
    ) -> Option<TimeRange> {
        use std::collections::HashMap;
        
        let painter = ui.painter();
        
        // 背景
        painter.rect_filled(
            rect,
            Rounding::same(self.theme.card_rounding),
            self.theme.card_background,
        );

        let padding = self.theme.card_padding;
        let content_rect = rect.shrink(padding);

        let mut clicked_range = None;
        let mut hovered_info: Option<(String, i64)> = None;
        
        // 计算每天的使用时间
        let mut daily_usage: HashMap<u32, i64> = HashMap::new();
        let mut max_usage = 0i64;

        for usage in self.app_usage {
            for event in &usage.window_events {
                let day = event.timestamp.ordinal(); // 一年中的第几天
                let entry = daily_usage.entry(day).or_insert(0);
                *entry += event.duration_secs;
                max_usage = max_usage.max(*entry);
            }
        }

        // 获取最近 N 天的日期（使用本地时间）
        let today = Local::now();
        let mut day_data: Vec<(u32, String, chrono::DateTime<Local>)> = Vec::new();
        
        for i in 0..days {
            let date = today - chrono::Duration::days(i as i64);
            let ordinal = date.ordinal();
            let label = if days <= 7 {
                // 7天内显示星期几
                let weekday = date.weekday();
                match weekday {
                    chrono::Weekday::Mon => "周一",
                    chrono::Weekday::Tue => "周二",
                    chrono::Weekday::Wed => "周三",
                    chrono::Weekday::Thu => "周四",
                    chrono::Weekday::Fri => "周五",
                    chrono::Weekday::Sat => "周六",
                    chrono::Weekday::Sun => "周日",
                }.to_string()
            } else {
                // 30天显示日期
                format!("{}/{}", date.month(), date.day())
            };
            day_data.push((ordinal, label, date));
        }
        
        // 反转使其从旧到新排列
        day_data.reverse();

        // 绘制柱状图
        let bar_gap = if days <= 7 { 8.0 } else { 2.0 };
        let bar_width = (content_rect.width() - (days - 1) as f32 * bar_gap) / days as f32;
        let chart_height = content_rect.height() - 30.0;
        let chart_bottom = content_rect.max.y - 20.0;

        // 检测鼠标位置
        let hover_pos = response.hover_pos();
        let click_pos = if response.clicked() { hover_pos } else { None };

        for (idx, (ordinal, label, date)) in day_data.iter().enumerate() {
            let usage = daily_usage.get(ordinal).copied().unwrap_or(0);
            
            let bar_height = if max_usage > 0 {
                (usage as f32 / max_usage as f32) * chart_height
            } else {
                0.0
            };

            let bar_x = content_rect.min.x + idx as f32 * (bar_width + bar_gap);
            // 扩展点击区域到整个柱子高度
            let clickable_rect = Rect::from_min_size(
                Pos2::new(bar_x, content_rect.min.y),
                Vec2::new(bar_width, chart_height + 10.0),
            );
            let bar_rect = Rect::from_min_size(
                Pos2::new(bar_x, chart_bottom - bar_height),
                Vec2::new(bar_width, bar_height.max(2.0)),
            );

            // 检查是否悬停或点击
            let is_hovered = hover_pos.map(|p| clickable_rect.contains(p)).unwrap_or(false);
            let is_clicked = click_pos.map(|p| clickable_rect.contains(p)).unwrap_or(false);

            // 根据使用量和悬停状态选择颜色
            let base_color = if usage > max_usage * 3 / 4 {
                self.theme.primary_color
            } else if usage > max_usage / 2 {
                self.theme.primary_color.linear_multiply(0.7)
            } else if usage > 0 {
                self.theme.primary_color.linear_multiply(0.4)
            } else {
                self.theme.divider_color
            };

            let color = if is_hovered {
                // 悬停时高亮
                self.theme.accent_color
            } else {
                base_color
            };

            painter.rect_filled(bar_rect, Rounding::same(2.0), color);

            // 如果点击了这个柱子，切换到该天
            if is_clicked {
                // 使用本地时间计算日期范围，然后转换为 UTC
                let day_start = date.date_naive()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_local_timezone(Local)
                    .unwrap()
                    .with_timezone(&chrono::Utc);
                let day_end = date.date_naive()
                    .and_hms_opt(23, 59, 59)
                    .unwrap()
                    .and_local_timezone(Local)
                    .unwrap()
                    .with_timezone(&chrono::Utc);
                clicked_range = Some(TimeRange::Custom(day_start, day_end));
            }

            // 记录悬停信息用于后续显示工具提示
            if is_hovered {
                hovered_info = Some((date.format("%Y-%m-%d").to_string(), usage));
            }

            // 日期标签
            let show_label = if days <= 7 {
                true // 7天内全部显示
            } else {
                idx % 5 == 0 || idx == days - 1 // 30天每5天显示一次
            };
            
            if show_label {
                let label_color = if is_hovered {
                    self.theme.text_color
                } else {
                    self.theme.secondary_text_color
                };
                painter.text(
                    Pos2::new(bar_x + bar_width / 2.0, chart_bottom + 10.0),
                    egui::Align2::CENTER_CENTER,
                    label,
                    egui::FontId::proportional(self.theme.small_size - 2.0),
                    label_color,
                );
            }
        }

        // Y轴标签
        if max_usage > 0 {
            let max_label = Self::format_duration_short(max_usage);
            painter.text(
                Pos2::new(content_rect.min.x, content_rect.min.y + 5.0),
                egui::Align2::LEFT_TOP,
                max_label,
                egui::FontId::proportional(self.theme.small_size - 2.0),
                self.theme.secondary_text_color,
            );
        }

        // 显示工具提示（在绘制完成后）
        if let Some((date_str, usage)) = hovered_info {
            response.clone().on_hover_text(format!("{}: {}", date_str, Self::format_duration(usage)));
        }

        clicked_range
    }

    /// 显示应用详情表格
    fn show_app_table(&mut self, ui: &mut Ui) {
        use crate::icons::AppIcon;
        
        if self.app_usage.is_empty() {
            ui.add(EmptyState::new(
                "📭",
                "所选时间范围内暂无数据",
                "尝试选择其他时间范围",
                self.theme,
            ));
            return;
        }

        let total_seconds: i64 = self.app_usage.iter()
            .map(|u| u.total_seconds)
            .sum();

        let available_height = ui.available_height().max(200.0);
        
        // 收集应用数据以避免借用冲突
        let app_data: Vec<_> = self.app_usage.iter().enumerate().map(|(idx, usage)| {
            let percentage = if total_seconds > 0 {
                (usage.total_seconds as f32 / total_seconds as f32) * 100.0
            } else {
                0.0
            };
            (idx, usage.app_name.clone(), usage.total_seconds, percentage)
        }).collect();

        TableBuilder::new(ui)
            .striped(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::exact(50.0))   // 排名
            .column(Column::exact(40.0))   // 图标
            .column(Column::remainder().at_least(150.0))  // 应用名称
            .column(Column::exact(100.0))  // 使用时长
            .column(Column::exact(80.0))   // 占比
            .column(Column::exact(100.0))  // 进度条
            .min_scrolled_height(0.0)
            .max_scroll_height(available_height)
            .header(32.0, |mut header| {
                header.col(|ui| {
                    ui.label(egui::RichText::new("排名")
                        .size(self.theme.small_size)
                        .color(self.theme.secondary_text_color));
                });
                header.col(|ui| {
                    ui.label(egui::RichText::new("")
                        .size(self.theme.small_size));
                });
                header.col(|ui| {
                    ui.label(egui::RichText::new("应用")
                        .size(self.theme.small_size)
                        .color(self.theme.secondary_text_color));
                });
                header.col(|ui| {
                    ui.label(egui::RichText::new("时长")
                        .size(self.theme.small_size)
                        .color(self.theme.secondary_text_color));
                });
                header.col(|ui| {
                    ui.label(egui::RichText::new("占比")
                        .size(self.theme.small_size)
                        .color(self.theme.secondary_text_color));
                });
                header.col(|ui| {
                    ui.label(egui::RichText::new("")
                        .size(self.theme.small_size));
                });
            })
            .body(|mut body| {
                for (idx, app_name, total_secs, percentage) in app_data {
                    body.row(36.0, |mut row| {
                        // 排名
                        row.col(|ui| {
                            let rank_color = match idx {
                                0 => Color32::from_rgb(255, 215, 0),  // 金色
                                1 => Color32::from_rgb(192, 192, 192), // 银色
                                2 => Color32::from_rgb(205, 127, 50),  // 铜色
                                _ => self.theme.secondary_text_color,
                            };
                            ui.label(egui::RichText::new(format!("#{}", idx + 1))
                                .size(self.theme.body_size)
                                .color(rank_color));
                        });

                        // 图标（使用真正的图标）
                        row.col(|ui| {
                            AppIcon::new(&app_name).size(24.0).show(ui, self.icon_cache);
                        });

                        // 应用名称
                        row.col(|ui| {
                            ui.label(egui::RichText::new(&app_name)
                                .size(self.theme.body_size)
                                .color(self.theme.text_color));
                        });

                        // 使用时长
                        row.col(|ui| {
                            ui.label(egui::RichText::new(Self::format_duration(total_secs))
                                .size(self.theme.body_size)
                                .color(self.theme.text_color));
                        });

                        // 占比
                        row.col(|ui| {
                            ui.label(egui::RichText::new(format!("{:.1}%", percentage))
                                .size(self.theme.small_size)
                                .color(self.theme.secondary_text_color));
                        });

                        // 进度条
                        row.col(|ui| {
                            let bar_width = 80.0;
                            let bar_height = 6.0;
                            let (rect, _) = ui.allocate_exact_size(
                                Vec2::new(bar_width, bar_height),
                                egui::Sense::hover(),
                            );

                            if ui.is_rect_visible(rect) {
                                let painter = ui.painter();
                                
                                // 背景
                                painter.rect_filled(
                                    rect,
                                    Rounding::same(3.0),
                                    self.theme.progress_background,
                                );

                                // 填充
                                let fill_width = rect.width() * (percentage / 100.0).min(1.0);
                                let fill_rect = Rect::from_min_size(
                                    rect.min,
                                    Vec2::new(fill_width, bar_height),
                                );
                                painter.rect_filled(
                                    fill_rect,
                                    Rounding::same(3.0),
                                    self.theme.primary_color,
                                );
                            }
                        });
                    });
                }
            });
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

    /// 格式化时长（短格式）
    fn format_duration_short(seconds: i64) -> String {
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;

        if hours > 0 {
            format!("{}h", hours)
        } else {
            format!("{}m", minutes)
        }
    }
}

/// 趋势指示器
pub struct TrendIndicator {
    /// 变化百分比
    change_percent: f32,
}

impl TrendIndicator {
    pub fn new(change_percent: f32) -> Self {
        Self { change_percent }
    }

    pub fn show(&self, ui: &mut Ui, theme: &TaiLTheme) {
        let (icon, color) = if self.change_percent > 5.0 {
            ("↑", theme.danger_color)
        } else if self.change_percent < -5.0 {
            ("↓", theme.success_color)
        } else {
            ("→", theme.secondary_text_color)
        };

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(icon).color(color));
            ui.label(egui::RichText::new(format!("{:.0}%", self.change_percent.abs()))
                .size(theme.small_size)
                .color(color));
        });
    }
}