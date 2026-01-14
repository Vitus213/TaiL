//! TaiL GUI 应用入口

use tail_gui::{setup_fonts, TaiLApp, ThemeType};

fn main() -> eframe::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or(tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // 初始化 egui
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 700.0])
            .with_min_inner_size([600.0, 400.0])
            .with_title("TaiL - 时间追踪"),
        ..Default::default()
    };

    eframe::run_native(
        "TaiL - Window Time Tracker",
        options,
        Box::new(|cc| {
            // 加载自定义字体（来自 tail-gui 库）
            setup_fonts(&cc.egui_ctx);
            
            // 应用默认主题
            let theme = ThemeType::default().to_theme();
            theme.apply(&cc.egui_ctx);

            Ok(Box::new(TaiLApp::new(cc)))
        }),
    )
}
