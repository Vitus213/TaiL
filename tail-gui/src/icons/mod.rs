//! TaiL GUI - 图标管理模块
//!
//! 提供应用图标的加载、缓存和显示功能。

pub mod cache;
pub mod ui_icons;

pub use cache::{IconCache, AppIcon};
pub use ui_icons::*;