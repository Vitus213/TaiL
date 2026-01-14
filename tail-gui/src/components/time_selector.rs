//! TaiL GUI - 时间选择器组件

use egui::{Color32, Pos2, Rect, Response, Rounding, Sense, Ui, Vec2, Widget};
use tail_core::models::TimeRange;

use crate::theme::TaiLTheme;

/// 时间范围选择器
pub struct TimeRangeSelector<'a> {
    /// 当前选中的时间范围
    current: TimeRange,
    /// 主题
    theme: &'a TaiLTheme,
}

impl<'a> TimeRangeSelector<'a> {
    pub fn new(current: TimeRange, theme: &'a TaiLTheme) -> Self {
        Self { current, theme }
    }
}

/// 时间范围选择器的返回值
pub struct TimeRangeSelectorResponse {
    pub response: Response,
    pub selected: Option<TimeRange>,
}

impl<'a> TimeRangeSelector<'a> {
    pub fn show(self, ui: &mut Ui) -> TimeRangeSelectorResponse {
        let mut selected = None;
        
        let response = ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            
            let options = [
                (TimeRange::Today, "今天"),
                (TimeRange::Yesterday, "昨天"),
                (TimeRange::Last7Days, "7天"),
                (TimeRange::Last30Days, "30天"),
            ];

            for (range, label) in options {
                let is_selected = self.is_same_range(&range);
                
                if ui.add(TimeRangeButton::new(label, is_selected, self.theme)).clicked() {
                    selected = Some(range);
                }
            }
        });

        TimeRangeSelectorResponse {
            response: response.response,
            selected,
        }
    }

    fn is_same_range(&self, other: &TimeRange) -> bool {
        matches!(
            (&self.current, other),
            (TimeRange::Today, TimeRange::Today)
                | (TimeRange::Yesterday, TimeRange::Yesterday)
                | (TimeRange::Last7Days, TimeRange::Last7Days)
                | (TimeRange::Last30Days, TimeRange::Last30Days)
        )
    }
}

/// 时间范围按钮
struct TimeRangeButton<'a> {
    label: &'a str,
    selected: bool,
    theme: &'a TaiLTheme,
}

impl<'a> TimeRangeButton<'a> {
    fn new(label: &'a str, selected: bool, theme: &'a TaiLTheme) -> Self {
        Self {
            label,
            selected,
            theme,
        }
    }
}

impl<'a> Widget for TimeRangeButton<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let padding = Vec2::new(16.0, 8.0);
        let text_size = self.theme.small_size;
        
        // 计算文本大小
        let galley = ui.painter().layout_no_wrap(
            self.label.to_string(),
            egui::FontId::proportional(text_size),
            Color32::WHITE,
        );
        let text_size_vec = galley.rect.size();
        
        let desired_size = text_size_vec + padding * 2.0;
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            
            // 背景颜色
            let bg_color = if self.selected {
                self.theme.primary_color
            } else if response.hovered() {
                self.theme.card_hover_background
            } else {
                self.theme.card_background
            };

            // 文字颜色
            let text_color = if self.selected {
                Color32::WHITE
            } else {
                self.theme.text_color
            };

            // 绘制背景
            painter.rect_filled(
                rect,
                Rounding::same(6.0),
                bg_color,
            );

            // 绘制文字
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                self.label,
                egui::FontId::proportional(self.theme.small_size),
                text_color,
            );
        }

        response
    }
}

/// 标签页选择器
pub struct TabSelector<'a> {
    /// 标签列表
    tabs: &'a [&'a str],
    /// 当前选中的索引
    selected: usize,
    /// 主题
    theme: &'a TaiLTheme,
}

impl<'a> TabSelector<'a> {
    pub fn new(tabs: &'a [&'a str], selected: usize, theme: &'a TaiLTheme) -> Self {
        Self {
            tabs,
            selected,
            theme,
        }
    }
}

pub struct TabSelectorResponse {
    pub response: Response,
    pub selected: Option<usize>,
}

impl<'a> TabSelector<'a> {
    pub fn show(self, ui: &mut Ui) -> TabSelectorResponse {
        let mut new_selected = None;
        
        let response = ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            
            for (idx, tab) in self.tabs.iter().enumerate() {
                let is_selected = idx == self.selected;
                let is_first = idx == 0;
                let is_last = idx == self.tabs.len() - 1;
                
                if ui.add(TabButton::new(tab, is_selected, is_first, is_last, self.theme)).clicked() {
                    new_selected = Some(idx);
                }
            }
        });

        TabSelectorResponse {
            response: response.response,
            selected: new_selected,
        }
    }
}

/// 标签按钮
struct TabButton<'a> {
    label: &'a str,
    selected: bool,
    is_first: bool,
    is_last: bool,
    theme: &'a TaiLTheme,
}

impl<'a> TabButton<'a> {
    fn new(label: &'a str, selected: bool, is_first: bool, is_last: bool, theme: &'a TaiLTheme) -> Self {
        Self {
            label,
            selected,
            is_first,
            is_last,
            theme,
        }
    }
}

impl<'a> Widget for TabButton<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let padding = Vec2::new(20.0, 10.0);
        let text_size = self.theme.body_size;
        
        let galley = ui.painter().layout_no_wrap(
            self.label.to_string(),
            egui::FontId::proportional(text_size),
            Color32::WHITE,
        );
        let text_size_vec = galley.rect.size();
        
        let desired_size = text_size_vec + padding * 2.0;
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            
            // 背景颜色
            let bg_color = if self.selected {
                self.theme.primary_color
            } else if response.hovered() {
                self.theme.card_hover_background
            } else {
                self.theme.card_background
            };

            // 文字颜色
            let text_color = if self.selected {
                Color32::WHITE
            } else {
                self.theme.text_color
            };

            // 计算圆角
            let rounding = if self.is_first && self.is_last {
                Rounding::same(8.0)
            } else if self.is_first {
                Rounding {
                    nw: 8.0,
                    sw: 8.0,
                    ne: 0.0,
                    se: 0.0,
                }
            } else if self.is_last {
                Rounding {
                    nw: 0.0,
                    sw: 0.0,
                    ne: 8.0,
                    se: 8.0,
                }
            } else {
                Rounding::ZERO
            };

            // 绘制背景
            painter.rect_filled(rect, rounding, bg_color);

            // 选中指示器
            if self.selected {
                let indicator_rect = Rect::from_min_size(
                    Pos2::new(rect.min.x, rect.max.y - 3.0),
                    Vec2::new(rect.width(), 3.0),
                );
                painter.rect_filled(
                    indicator_rect,
                    Rounding::ZERO,
                    self.theme.accent_color,
                );
            }

            // 绘制文字
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                self.label,
                egui::FontId::proportional(self.theme.body_size),
                text_color,
            );
        }

        response
    }
}

/// 日期显示组件
pub struct DateDisplay<'a> {
    /// 日期文本
    date_text: &'a str,
    /// 主题
    theme: &'a TaiLTheme,
}

impl<'a> DateDisplay<'a> {
    pub fn new(date_text: &'a str, theme: &'a TaiLTheme) -> Self {
        Self { date_text, theme }
    }
}

impl<'a> Widget for DateDisplay<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let desired_size = Vec2::new(ui.available_width(), 30.0);
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::hover());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // 日历图标
            painter.text(
                Pos2::new(rect.min.x, rect.center().y),
                egui::Align2::LEFT_CENTER,
                "📅",
                egui::FontId::proportional(16.0),
                self.theme.secondary_text_color,
            );

            // 日期文本
            painter.text(
                Pos2::new(rect.min.x + 24.0, rect.center().y),
                egui::Align2::LEFT_CENTER,
                self.date_text,
                egui::FontId::proportional(self.theme.small_size),
                self.theme.text_color,
            );
        }

        response
    }
}