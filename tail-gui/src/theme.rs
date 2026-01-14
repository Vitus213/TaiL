//! TaiL GUI - 主题模块

use eframe::egui;

/// 应用主题
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Light,
    Dark,
    Auto,
}

impl Theme {
    /// 应用主题到 egui 上下文 (egui 0.28 API)
    pub fn apply(self, ctx: &egui::Context) {
        match self {
            Theme::Light => {
                ctx.set_visuals(egui::Visuals::light());
            }
            Theme::Dark => {
                ctx.set_visuals(egui::Visuals::dark());
            }
            Theme::Auto => {
                // 跟随系统主题
                #[cfg(target_os = "linux")]
                {
                    if dark_light::detect() == dark_light::Mode::Dark {
                        ctx.set_visuals(egui::Visuals::dark());
                    } else {
                        ctx.set_visuals(egui::Visuals::light());
                    }
                }
                #[cfg(not(target_os = "linux"))]
                {
                    ctx.set_visuals(egui::Visuals::dark());
                }
            }
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Theme::Auto
    }
}
