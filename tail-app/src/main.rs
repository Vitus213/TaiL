//! TaiL GUI 应用入口

use tail_gui::TaiLApp;
use tail_gui::theme::Theme;

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
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([400.0, 300.0]),
        ..Default::default()
    };

    eframe::run_native(
        "TaiL - Window Time Tracker",
        options,
        Box::new(|cc| {
            // 手动加载系统字体
            setup_custom_fonts(&cc.egui_ctx);
            
            // 应用主题
            Theme::Auto.apply(&cc.egui_ctx);

            Ok(Box::new(TaiLApp::new(cc)))
        }),
    )
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    
    // 加载 Noto Sans CJK 字体（用于中文显示）
    let noto_font_data = include_bytes!("../assets/fonts/NotoSansCJK-VF.otf.ttc");
    fonts.font_data.insert(
        "noto_sans_cjk".to_owned(),
        egui::FontData::from_owned(noto_font_data.to_vec()),
    );
    
    // 加载 JetBrains Mono 字体（用于等宽显示）
    let jetbrains_font_data = include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf");
    fonts.font_data.insert(
        "jetbrains_mono".to_owned(),
        egui::FontData::from_owned(jetbrains_font_data.to_vec()),
    );
    
    // 设置字体家族优先级
    // Proportional: 用于普通文本，优先使用 JetBrains Mono，回退到 Noto Sans CJK
    fonts.families.insert(
        egui::FontFamily::Proportional,
        vec![
            "jetbrains_mono".to_owned(),
            "noto_sans_cjk".to_owned(),
        ],
    );
    
    // Monospace: 用于等宽文本（数字、代码等）
    fonts.families.insert(
        egui::FontFamily::Monospace,
        vec![
            "jetbrains_mono".to_owned(),
            "noto_sans_cjk".to_owned(),
        ],
    );
    
    tracing::info!("成功加载嵌入字体: JetBrains Mono + Noto Sans CJK");
    
    ctx.set_fonts(fonts);
}
