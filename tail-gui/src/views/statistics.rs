//! TaiL GUI - 统计视图

use chrono::{Datelike, Local};
use egui::{Color32, Rect, Rounding, Ui, Vec2};
use egui_extras::{Column, TableBuilder};
use tail_core::AppUsage;
use tail_core::models::TimeRange;
use tail_core::TimeNavigationState;

use crate::components::{
    EmptyState, HierarchicalBarChart, PageHeader, SectionDivider, QuickTimeRange,
    TimeNavigationController,
};
use crate::icons::IconCache;
use crate::theme::TaiLTheme;
use crate::utils::duration;
use crate::views::aggregation::DataAggregator;

/// 统计视图
pub struct StatisticsView<'a> {
    /// 应用使用数据
    app_usage: &'a [AppUsage],
    /// 时间导航状态
    navigation_state: &'a mut TimeNavigationState,
    /// 主题
    theme: &'a TaiLTheme,
    /// 图标缓存（可变引用以支持加载图标）
    icon_cache: &'a mut IconCache,
}

impl<'a> StatisticsView<'a> {
    pub fn new(
        app_usage: &'a [AppUsage],
        navigation_state: &'a mut TimeNavigationState,
        theme: &'a TaiLTheme,
        icon_cache: &'a mut IconCache,
    ) -> Self {
        Self {
            app_usage,
            navigation_state,
            theme,
            icon_cache,
        }
    }

    /// 渲染统计视图，返回新选择的时间范围（如果有变化）
    pub fn show(&mut self, ui: &mut Ui) -> Option<TimeRange> {
        let mut new_time_range = None;

        // 页面标题
        ui.add(PageHeader::new("详细统计", "📈", self.theme).subtitle("查看应用使用详情"));

        ui.add_space(self.theme.spacing);

        // 时间导航控制器
        let controller = TimeNavigationController::new(self.navigation_state, self.theme);
        let (go_back, quick_range, selected_level) = controller.show(ui);

        // 处理导航事件
        if go_back {
            self.navigation_state.go_back();
            new_time_range = Some(self.navigation_state.to_time_range());
        } else if let Some(quick) = quick_range {
            // 处理快捷时间范围选择
            let now = Local::now();
            eprintln!("[DEBUG] 统计视图 - 快捷时间范围被选择: {:?}", quick);
            match quick {
                QuickTimeRange::Today => {
                    self.navigation_state.go_to_today(now.year(), now.month(), now.day());
                    new_time_range = Some(self.navigation_state.to_time_range());
                }
                QuickTimeRange::ThisWeek => {
                    self.navigation_state.switch_to_this_week(now.year(), now.month());
                    new_time_range = Some(self.navigation_state.to_time_range());
                }
                QuickTimeRange::ThisMonth => {
                    eprintln!("[DEBUG] 统计视图 - 切换到本月: year={}, month={}", now.year(), now.month());
                    self.navigation_state.switch_to_this_month(now.year(), now.month());
                    eprintln!("[DEBUG] 统计视图 - 导航状态更新后: level={:?}, year={}, month={:?}, week={:?}",
                        self.navigation_state.level,
                        self.navigation_state.selected_year,
                        self.navigation_state.selected_month,
                        self.navigation_state.selected_week);
                    new_time_range = Some(self.navigation_state.to_time_range());
                }
                QuickTimeRange::ThisYear => {
                    self.navigation_state.switch_to_this_year(now.year());
                    new_time_range = Some(self.navigation_state.to_time_range());
                }
            }
        } else if let Some(level) = selected_level {
            // 切换视图级别
            self.navigation_state.switch_level(level);
            new_time_range = Some(self.navigation_state.to_time_range());
        }

        ui.add_space(self.theme.spacing);

        // 层级柱形图
        ui.add(SectionDivider::new(self.theme).with_title("时间分布 (点击柱子下钻)"));
        ui.add_space(self.theme.spacing / 2.0);

        let aggregator = DataAggregator::new(self.app_usage);
        let periods = aggregator.aggregate(self.navigation_state);
        
        eprintln!("[DEBUG] 统计视图 - 聚合数据: level={:?}, periods.len()={}",
            self.navigation_state.level, periods.len());
        for (i, period) in periods.iter().enumerate().take(5) {
            eprintln!("[DEBUG] 统计视图 - Period[{}]: label={}, total_seconds={}",
                i, period.label, period.total_seconds);
        }

        let chart =
            HierarchicalBarChart::new(&periods, self.navigation_state.level, "", self.theme);

        if let Some(clicked_index) = chart.show(ui) {
            // 根据当前层级处理点击事件
            match self.navigation_state.level {
                tail_core::models::TimeNavigationLevel::Year => {
                    // 年视图不显示，直接进入月视图
                }
                tail_core::models::TimeNavigationLevel::Month => {
                    self.navigation_state.drill_into_month(clicked_index as u32);
                    new_time_range = Some(self.navigation_state.to_time_range());
                }
                tail_core::models::TimeNavigationLevel::Week => {
                    self.navigation_state.drill_into_week(clicked_index as u32);
                    new_time_range = Some(self.navigation_state.to_time_range());
                }
                tail_core::models::TimeNavigationLevel::Day => {
                    self.navigation_state.drill_into_day(clicked_index as u32);
                    new_time_range = Some(self.navigation_state.to_time_range());
                }
                tail_core::models::TimeNavigationLevel::Hour => {
                    // 小时视图是最底层，不再下钻
                }
            }
        }

        ui.add_space(self.theme.spacing);

        // 应用详情表格
        ui.add(SectionDivider::new(self.theme).with_title("应用详情"));
        ui.add_space(self.theme.spacing / 2.0);
        self.show_app_table(ui);

        new_time_range
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

        let total_seconds: i64 = self.app_usage.iter().map(|u| u.total_seconds).sum();

        let available_height = ui.available_height().max(200.0);

        // 收集应用数据以避免借用冲突
        let app_data: Vec<_> = self
            .app_usage
            .iter()
            .enumerate()
            .map(|(idx, usage)| {
                let percentage = if total_seconds > 0 {
                    (usage.total_seconds as f32 / total_seconds as f32) * 100.0
                } else {
                    0.0
                };
                (idx, usage.app_name.clone(), usage.total_seconds, percentage)
            })
            .collect();

        TableBuilder::new(ui)
            .striped(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::exact(50.0)) // 排名
            .column(Column::exact(40.0)) // 图标
            .column(Column::remainder().at_least(150.0)) // 应用名称
            .column(Column::exact(100.0)) // 使用时长
            .column(Column::exact(80.0)) // 占比
            .column(Column::exact(100.0)) // 进度条
            .min_scrolled_height(0.0)
            .max_scroll_height(available_height)
            .header(32.0, |mut header| {
                header.col(|ui| {
                    ui.label(
                        egui::RichText::new("排名")
                            .size(self.theme.small_size)
                            .color(self.theme.secondary_text_color),
                    );
                });
                header.col(|ui| {
                    ui.label(egui::RichText::new("").size(self.theme.small_size));
                });
                header.col(|ui| {
                    ui.label(
                        egui::RichText::new("应用")
                            .size(self.theme.small_size)
                            .color(self.theme.secondary_text_color),
                    );
                });
                header.col(|ui| {
                    ui.label(
                        egui::RichText::new("时长")
                            .size(self.theme.small_size)
                            .color(self.theme.secondary_text_color),
                    );
                });
                header.col(|ui| {
                    ui.label(
                        egui::RichText::new("占比")
                            .size(self.theme.small_size)
                            .color(self.theme.secondary_text_color),
                    );
                });
                header.col(|ui| {
                    ui.label(egui::RichText::new("").size(self.theme.small_size));
                });
            })
            .body(|mut body| {
                for (idx, app_name, total_secs, percentage) in app_data {
                    body.row(36.0, |mut row| {
                        // 排名
                        row.col(|ui| {
                            let rank_color = match idx {
                                0 => Color32::from_rgb(255, 215, 0),   // 金色
                                1 => Color32::from_rgb(192, 192, 192), // 银色
                                2 => Color32::from_rgb(205, 127, 50),  // 铜色
                                _ => self.theme.secondary_text_color,
                            };
                            ui.label(
                                egui::RichText::new(format!("#{}", idx + 1))
                                    .size(self.theme.body_size)
                                    .color(rank_color),
                            );
                        });

                        // 图标（使用真正的图标）
                        row.col(|ui| {
                            AppIcon::new(&app_name).size(24.0).show(ui, self.icon_cache);
                        });

                        // 应用名称
                        row.col(|ui| {
                            ui.label(
                                egui::RichText::new(&app_name)
                                    .size(self.theme.body_size)
                                    .color(self.theme.text_color),
                            );
                        });

                        // 使用时长
                        row.col(|ui| {
                            ui.label(
                                egui::RichText::new(duration::format_duration(total_secs))
                                    .size(self.theme.body_size)
                                    .color(self.theme.text_color),
                            );
                        });

                        // 占比
                        row.col(|ui| {
                            ui.label(
                                egui::RichText::new(format!("{:.1}%", percentage))
                                    .size(self.theme.small_size)
                                    .color(self.theme.secondary_text_color),
                            );
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
            ui.label(
                egui::RichText::new(format!("{:.0}%", self.change_percent.abs()))
                    .size(theme.small_size)
                    .color(color),
            );
        });
    }
}
