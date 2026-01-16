//! TaiL GUI 应用入口

use tail_core::logging::LogOutput;
use tail_gui::{TaiLApp, ThemeType, setup_fonts};
use tracing::{info, warn};

/// 加载应用图标
fn load_app_icon() -> Option<egui::IconData> {
    // 尝试从嵌入的 SVG 加载图标
    let svg_data = include_bytes!("../../tail-gui/assets/icons/tail.svg");
    let svg_len = svg_data.len();

    info!(size_bytes = svg_len, "正在加载应用图标");

    // 解析 SVG
    let options = resvg::usvg::Options::default();
    let tree = match resvg::usvg::Tree::from_data(svg_data, &options) {
        Ok(t) => t,
        Err(e) => {
            tracing::error!(error = %e, "解析 SVG 图标失败");
            return None;
        }
    };

    // 渲染为 64x64 像素
    let size = 64;
    let mut pixmap = match resvg::tiny_skia::Pixmap::new(size, size) {
        Some(p) => p,
        None => {
            tracing::error!("创建 Pixmap 失败");
            return None;
        }
    };

    let tree_size = tree.size();
    let scale_x = size as f32 / tree_size.width();
    let scale_y = size as f32 / tree_size.height();
    let scale = scale_x.min(scale_y);

    let offset_x = (size as f32 - tree_size.width() * scale) / 2.0;
    let offset_y = (size as f32 - tree_size.height() * scale) / 2.0;

    let transform =
        resvg::tiny_skia::Transform::from_scale(scale, scale).post_translate(offset_x, offset_y);

    resvg::render(&tree, transform, &mut pixmap.as_mut());

    let pixels = pixmap.take();
    info!(width = size, height = size, "应用图标加载成功");

    Some(egui::IconData {
        rgba: pixels,
        width: size,
        height: size,
    })
}

fn main() -> eframe::Result<()> {
    // 初始化日志系统
    tail_core::logging::init_logging(LogOutput::Stdout, "info");

    info!("TaiL GUI 应用正在启动...");

    // 加载应用图标
    let icon = load_app_icon();
    if icon.is_none() {
        warn!("无法加载应用图标，将使用默认图标");
    }

    // 初始化 egui
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([900.0, 700.0])
        .with_min_inner_size([600.0, 400.0])
        .with_title("TaiL - 时间追踪")
        .with_app_id("tail"); // 设置 Wayland app_id，用于 Hyprand 识别窗口

    // 设置窗口图标
    if let Some(icon_data) = icon {
        viewport = viewport.with_icon(std::sync::Arc::new(icon_data));
    }

    let options = eframe::NativeOptions {
        viewport,
        // 设置为首选以支持软件渲染后备（解决 NixOS 上 glow 线程问题）
        hardware_acceleration: eframe::HardwareAcceleration::Preferred,
        follow_system_theme: false,
        vsync: true,
        ..Default::default()
    };

    info!("正在创建 egui 窗口...");

    eframe::run_native(
        "TaiL - Window Time Tracker",
        options,
        Box::new(|cc| {
            // 加载自定义字体（来自 tail-gui 库）
            setup_fonts(&cc.egui_ctx);

            // 应用默认主题
            let theme = ThemeType::default().to_theme();
            theme.apply(&cc.egui_ctx);

            info!("TaiL GUI 应用已启动");
            Ok(Box::new(TaiLApp::new(cc)))
        }),
    )
}
