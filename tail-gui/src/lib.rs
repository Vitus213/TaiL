//! TaiL GUI - egui 界面
//!
//! 这是 TaiL 时间追踪工具的图形用户界面模块。
//! 使用 egui 框架构建，提供现代化、美观的用户体验。

pub mod app;
pub mod components;
pub mod fonts;
pub mod icons;
pub mod theme;
pub mod views;

pub use app::*;
pub use fonts::setup_fonts;
pub use theme::{TaiLTheme, ThemeType};
