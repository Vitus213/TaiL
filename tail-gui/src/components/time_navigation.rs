//! 时间导航控制器组件

use chrono::{Datelike, Local};
use egui::Ui;
use tail_core::models::{TimeNavigationLevel, TimeNavigationState};

use crate::theme::TaiLTheme;

/// 快捷时间范围选择
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuickTimeRange {
    Today,
    ThisWeek,
    ThisMonth,
    ThisYear,
}

/// 时间导航控制器
pub struct TimeNavigationController<'a> {
    /// 当前导航状态
    state: &'a TimeNavigationState,
    /// 主题
    theme: &'a TaiLTheme,
}

impl<'a> TimeNavigationController<'a> {
    /// 创建新的时间导航控制器
    pub fn new(state: &'a TimeNavigationState, theme: &'a TaiLTheme) -> Self {
        Self { state, theme }
    }

    /// 显示导航控制器
    /// 返回：(是否返回上一级, 快捷时间范围选择, 选择的视图级别)
    pub fn show(&self, ui: &mut Ui) -> (bool, Option<QuickTimeRange>, Option<TimeNavigationLevel>) {
        let mut go_back = false;
        let mut quick_range = None;
        let mut selected_level = None;

        ui.horizontal(|ui| {
            // 面包屑导航
            ui.label(
                egui::RichText::new(format!("📍 {}", self.state.get_breadcrumb()))
                    .color(self.theme.text_color)
                    .size(14.0),
            );

            ui.add_space(16.0);

            // 返回按钮
            if ui
                .button(egui::RichText::new("⬅ 返回").color(self.theme.text_color))
                .clicked()
            {
                go_back = true;
            }

            ui.add_space(8.0);

            // 快捷时间范围按钮
            // 今天按钮
            let is_today = self.state.level == TimeNavigationLevel::Hour
                && self.is_current_today();
            if ui
                .selectable_label(
                    is_today,
                    egui::RichText::new("📅 今天")
                        .size(13.0)
                        .color(if is_today {
                            self.theme.primary_color
                        } else {
                            self.theme.text_color
                        }),
                )
                .clicked()
                && !is_today
            {
                quick_range = Some(QuickTimeRange::Today);
            }

            // 本周按钮
            let is_this_week = self.state.level == TimeNavigationLevel::Day
                && self.is_current_week();
            if ui
                .selectable_label(
                    is_this_week,
                    egui::RichText::new("📆 本周")
                        .size(13.0)
                        .color(if is_this_week {
                            self.theme.primary_color
                        } else {
                            self.theme.text_color
                        }),
                )
                .clicked()
                && !is_this_week
            {
                quick_range = Some(QuickTimeRange::ThisWeek);
            }

            // 本月按钮
            let is_this_month = self.state.level == TimeNavigationLevel::Week
                && self.is_current_month();
            if ui
                .selectable_label(
                    is_this_month,
                    egui::RichText::new("🗓️ 本月")
                        .size(13.0)
                        .color(if is_this_month {
                            self.theme.primary_color
                        } else {
                            self.theme.text_color
                        }),
                )
                .clicked()
                && !is_this_month
            {
                quick_range = Some(QuickTimeRange::ThisMonth);
            }

            // 本年按钮
            let is_this_year = self.state.level == TimeNavigationLevel::Month
                && self.is_current_year();
            if ui
                .selectable_label(
                    is_this_year,
                    egui::RichText::new("📅 本年")
                        .size(13.0)
                        .color(if is_this_year {
                            self.theme.primary_color
                        } else {
                            self.theme.text_color
                        }),
                )
                .clicked()
                && !is_this_year
            {
                quick_range = Some(QuickTimeRange::ThisYear);
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // 视图级别选择器
                ui.label(egui::RichText::new("视图:").size(self.theme.small_size));

                for level in [
                    TimeNavigationLevel::Month,
                    TimeNavigationLevel::Week,
                    TimeNavigationLevel::Day,
                ] {
                    let label = match level {
                        TimeNavigationLevel::Month => "月视图",
                        TimeNavigationLevel::Week => "周视图",
                        TimeNavigationLevel::Day => "日视图",
                        _ => continue,
                    };

                    let is_active = self.state.level == level;

                    if ui
                        .selectable_label(is_active, egui::RichText::new(label).size(13.0))
                        .clicked()
                        && !is_active
                    {
                        selected_level = Some(level);
                    }
                }
            });
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        (go_back, quick_range, selected_level)
    }

    /// 检查是否是当前今天
    fn is_current_today(&self) -> bool {
        let now = Local::now();
        self.state.selected_year == now.year()
            && self.state.selected_month == Some(now.month())
            && self.state.selected_day == Some(now.day())
    }

    /// 检查是否是当前周
    fn is_current_week(&self) -> bool {
        let now = Local::now();
        self.state.selected_year == now.year()
            && self.state.selected_month == Some(now.month())
    }

    /// 检查是否是当前月
    fn is_current_month(&self) -> bool {
        let now = Local::now();
        self.state.selected_year == now.year()
            && self.state.selected_month == Some(now.month())
    }

    /// 检查是否是当前年
    fn is_current_year(&self) -> bool {
        let now = Local::now();
        self.state.selected_year == now.year()
    }
}
