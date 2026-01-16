//! TaiL GUI - 统计视图

use chrono::{Datelike, Local, Utc};
use egui::{Color32, Rect, Rounding, Ui, Vec2};
use egui_extras::{Column, TableBuilder};
use tail_core::AppUsage;
use tail_core::TimeNavigationState;
use tail_core::models::TimeRange;

use crate::components::chart::{
    ChartDataBuilder, ChartGroupMode, ChartTimeGranularity, StackedBarChart, StackedBarChartConfig,
    StackedBarTooltip,
};
use crate::components::{
    EmptyState, HierarchicalBarChart, PageHeader, QuickTimeRange, SectionDivider,
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
    /// 是否使用堆叠视图
    use_stacked_view: bool,
    /// 悬停的时间槽索引
    hovered_slot: Option<usize>,
}

impl<'a> StatisticsView<'a> {
    pub fn new(
        app_usage: &'a [AppUsage],
        navigation_state: &'a mut TimeNavigationState,
        theme: &'a TaiLTheme,
        icon_cache: &'a mut IconCache,
        use_stacked_view: bool,
    ) -> Self {
        Self {
            app_usage,
            navigation_state,
            theme,
            icon_cache,
            use_stacked_view,
            hovered_slot: None,
        }
    }

    /// 渲染统计视图，返回 (新选择的时间范围, 是否使用堆叠视图)
    pub fn show(&mut self, ui: &mut Ui) -> (Option<TimeRange>, bool) {
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
                QuickTimeRange::Yesterday => {
                    // 昨天 - 显示24小时
                    let yesterday = Local::now().date_naive() - chrono::Duration::days(1);
                    self.navigation_state.go_to_yesterday(
                        yesterday.year(),
                        yesterday.month(),
                        yesterday.day(),
                    );
                    new_time_range = Some(TimeRange::Yesterday);
                }
                QuickTimeRange::Today => {
                    // 今天 - 显示24小时
                    self.navigation_state
                        .go_to_today(now.year(), now.month(), now.day());
                    new_time_range = Some(TimeRange::Today);
                }
                QuickTimeRange::ThisWeek => {
                    // 本周 - 显示7天
                    // 设置 level = Day，不设置 selected_week，这样 to_time_range() 返回整月
                    // 然后数据会聚合为7天
                    self.navigation_state.selected_year = now.year();
                    self.navigation_state.selected_month = Some(now.month());
                    self.navigation_state.selected_week = None;
                    self.navigation_state.selected_day = None;
                    self.navigation_state.level = tail_core::models::TimeNavigationLevel::Day;
                    // 使用本周的时间范围（从周一开始）
                    let weekday = now.date_naive().weekday().num_days_from_monday();
                    let week_start = now.date_naive() - chrono::Duration::days(weekday as i64);
                    let week_start_dt = week_start
                        .and_hms_opt(0, 0, 0)
                        .unwrap()
                        .and_local_timezone(Local)
                        .unwrap()
                        .with_timezone(&Utc);
                    let week_end = week_start_dt + chrono::Duration::days(7);
                    new_time_range = Some(TimeRange::Custom(week_start_dt, week_end));
                }
                QuickTimeRange::ThisMonth => {
                    // 本月 - 显示该月的周
                    self.navigation_state.selected_year = now.year();
                    self.navigation_state.selected_month = Some(now.month());
                    self.navigation_state.selected_week = None;
                    self.navigation_state.selected_day = None;
                    self.navigation_state.level = tail_core::models::TimeNavigationLevel::Week;
                    new_time_range = Some(self.navigation_state.to_time_range());
                }
                QuickTimeRange::ThisYear => {
                    // 本年 - 显示12个月
                    self.navigation_state.selected_year = now.year();
                    self.navigation_state.selected_month = None;
                    self.navigation_state.selected_week = None;
                    self.navigation_state.selected_day = None;
                    self.navigation_state.level = tail_core::models::TimeNavigationLevel::Month;
                    new_time_range = Some(self.navigation_state.to_time_range());
                }
            }
        } else if let Some(level) = selected_level {
            // 切换视图级别
            self.navigation_state.switch_level(level);
            new_time_range = Some(self.navigation_state.to_time_range());
        }

        ui.add_space(self.theme.spacing);

        // 图表类型切换按钮
        ui.horizontal(|ui| {
            ui.label("图表类型:");
            if ui
                .selectable_label(!self.use_stacked_view, "📊 简单柱形图")
                .clicked()
            {
                eprintln!("[DEBUG] 切换到简单柱形图");
                self.use_stacked_view = false;
            }
            if ui
                .selectable_label(self.use_stacked_view, "📈 堆叠柱形图")
                .clicked()
            {
                eprintln!("[DEBUG] 切换到堆叠柱形图");
                self.use_stacked_view = true;
            }
        });

        ui.add_space(self.theme.spacing / 2.0);

        eprintln!(
            "[DEBUG] 准备显示图表, use_stacked_view={}",
            self.use_stacked_view
        );

        // 层级柱形图或堆叠柱形图
        if self.use_stacked_view {
            eprintln!("[DEBUG] 进入堆叠柱形图分支");
            ui.add(SectionDivider::new(self.theme).with_title("时间分布 (按应用堆叠)"));
            ui.add_space(self.theme.spacing / 2.0);
            eprintln!("[DEBUG] 即将调用 show_stacked_chart");
            self.show_stacked_chart(ui);
            eprintln!("[DEBUG] show_stacked_chart 返回");
        } else {
            ui.add(SectionDivider::new(self.theme).with_title("时间分布 (点击柱子下钻)"));
            ui.add_space(self.theme.spacing / 2.0);
            let aggregator = DataAggregator::new(self.app_usage);
            let periods = aggregator.aggregate(self.navigation_state);

            eprintln!(
                "[DEBUG] 统计视图 - 聚合数据: level={:?}, periods.len()={}",
                self.navigation_state.level,
                periods.len()
            );
            for (i, period) in periods.iter().enumerate().take(5) {
                eprintln!(
                    "[DEBUG] 统计视图 - Period[{}]: label={}, total_seconds={}",
                    i, period.label, period.total_seconds
                );
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
        }

        ui.add_space(self.theme.spacing);

        // 应用详情表格
        ui.add(SectionDivider::new(self.theme).with_title("应用详情"));
        ui.add_space(self.theme.spacing / 2.0);
        self.show_app_table(ui);

        (new_time_range, self.use_stacked_view)
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

        // 只计算非 AFK 时间，与柱形图保持一致
        let total_seconds: i64 = self
            .app_usage
            .iter()
            .map(|u| {
                u.window_events
                    .iter()
                    .filter(|e| !e.is_afk)
                    .map(|e| e.duration_secs)
                    .sum::<i64>()
            })
            .sum();

        let available_height = ui.available_height().max(200.0);

        // 收集应用数据以避免借用冲突，并按使用时长降序排序
        // 只计算非 AFK 时间，与柱形图保持一致
        let mut app_data: Vec<_> = self
            .app_usage
            .iter()
            .filter(|usage| !usage.app_name.is_empty())
            .map(|usage| {
                let non_afk_seconds: i64 = usage
                    .window_events
                    .iter()
                    .filter(|e| !e.is_afk)
                    .map(|e| e.duration_secs)
                    .sum();

                let percentage = if total_seconds > 0 {
                    (non_afk_seconds as f32 / total_seconds as f32) * 100.0
                } else {
                    0.0
                };
                (usage.app_name.clone(), non_afk_seconds, percentage)
            })
            .collect();

        // 按使用时长降序排序
        app_data.sort_by(|a, b| b.1.cmp(&a.1));

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
                for (rank, (app_name, total_secs, percentage)) in app_data.into_iter().enumerate() {
                    body.row(36.0, |mut row| {
                        // 排名
                        row.col(|ui| {
                            let rank_color = match rank {
                                0 => Color32::from_rgb(255, 215, 0),   // 金色
                                1 => Color32::from_rgb(192, 192, 192), // 银色
                                2 => Color32::from_rgb(205, 127, 50),  // 铜色
                                _ => self.theme.secondary_text_color,
                            };
                            ui.label(
                                egui::RichText::new(format!("#{}", rank + 1))
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

    /// 显示堆叠柱状图（按应用堆叠）
    fn show_stacked_chart(&mut self, ui: &mut Ui) {
        // 根据当前导航状态确定时间粒度
        // 快捷选项的 level 设置：
        // - Today: level = Hour (显示24小时)
        // - ThisWeek: level = Day, selected_week = None (显示7天)
        // - ThisMonth: level = Week (显示该月的周)
        // - ThisYear: level = Month (显示12个月)

        let granularity = match self.navigation_state.level {
            tail_core::models::TimeNavigationLevel::Month => {
                // Month level 表示显示12个月（本年快捷选项）
                ChartTimeGranularity::Year
            }
            tail_core::models::TimeNavigationLevel::Week => {
                // Week level 表示显示该月的周（本月快捷选项）
                ChartTimeGranularity::Month
            }
            tail_core::models::TimeNavigationLevel::Day => {
                // Day level:
                // - 如果 selected_week 是 None，表示显示7天（本周快捷选项）
                // - 如果 selected_week 有值，表示显示该周7天
                ChartTimeGranularity::Week
            }
            tail_core::models::TimeNavigationLevel::Hour => {
                // Hour level 表示显示24小时（今天快捷选项）
                ChartTimeGranularity::Day
            }
            tail_core::models::TimeNavigationLevel::Year => {
                // Year level 不应该出现在快捷选项中
                ChartTimeGranularity::Year
            }
        };

        eprintln!(
            "[DEBUG] show_stacked_chart - level={:?}, granularity={:?}, app_usage.len()={}",
            self.navigation_state.level,
            granularity,
            self.app_usage.len()
        );

        // 如果数据为空，显示空状态而不是尝试构建图表
        if self.app_usage.is_empty() {
            ui.add(EmptyState::new(
                "📊",
                "暂无数据",
                "请选择其他时间范围",
                self.theme,
            ));
            return;
        }

        let chart_data = ChartDataBuilder::new(self.app_usage)
            .with_granularity(granularity)
            .with_group_mode(ChartGroupMode::ByApp)
            .build();

        eprintln!(
            "[DEBUG] show_stacked_chart - chart_data.time_slots.len()={}, max_seconds={}",
            chart_data.time_slots.len(),
            chart_data.max_seconds()
        );

        if chart_data.time_slots.is_empty() {
            ui.label("暂无数据");
            return;
        }

        let config = StackedBarChartConfig {
            max_bar_height: 200.0,
            ..Default::default()
        };

        eprintln!("[DEBUG] show_stacked_chart - 准备显示图表");

        let chart = StackedBarChart::new(&chart_data, self.theme).with_config(config);

        eprintln!("[DEBUG] show_stacked_chart - 开始调用 chart.show()");
        self.hovered_slot = chart.show(ui);
        eprintln!(
            "[DEBUG] show_stacked_chart - chart.show() 返回, hovered_slot={:?}",
            self.hovered_slot
        );

        // 显示悬停提示
        if let Some(idx) = self.hovered_slot
            && let Some(slot) = chart_data.time_slots.get(idx)
        {
            eprintln!(
                "[DEBUG] show_stacked_chart - 显示 tooltip, idx={}, label={}",
                idx, slot.label
            );
            let tooltip = StackedBarTooltip::new(slot);
            tooltip.show(ui, self.theme);
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
            ui.label(
                egui::RichText::new(format!("{:.0}%", self.change_percent.abs()))
                    .size(theme.small_size)
                    .color(color),
            );
        });
    }
}
