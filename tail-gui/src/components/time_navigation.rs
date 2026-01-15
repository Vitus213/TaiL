//! 时间导航控制器组件

use egui::Ui;
use tail_core::models::TimeNavigationState;

use crate::theme::TaiLTheme;

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
    /// 返回：(是否返回上一级, 是否跳转今天, 是否跳转昨天)
    pub fn show(&self, ui: &mut Ui) -> (bool, bool, bool) {
        let mut go_back = false;
        let mut go_today = false;
        let mut go_yesterday = false;

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

            // 今天按钮
            if ui
                .button(egui::RichText::new("📅 今天").color(self.theme.primary_color))
                .clicked()
            {
                go_today = true;
            }

            ui.add_space(4.0);

            // 昨天按钮
            if ui
                .button(egui::RichText::new("📆 昨天").color(self.theme.text_color))
                .clicked()
            {
                go_yesterday = true;
            }
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        (go_back, go_today, go_yesterday)
    }
}
