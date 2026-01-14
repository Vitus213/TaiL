//! 字体配置模块
//!
//! 提供 TaiL GUI 的字体加载和配置功能。
//! 使用嵌入式字体确保跨平台一致的显示效果。

use egui::{Context, FontData, FontDefinitions, FontFamily};

/// 嵌入的 LXGW WenKai 字体（中文，霞鹜文楷）
const LXGW: &[u8] = include_bytes!("../assets/fonts/LXGWWenKai-Regular.ttf");

/// 嵌入的 Noto Sans SC 字体（中文后备）
const NOTO_SANS_SC: &[u8] = include_bytes!("../assets/fonts/NotoSansSC-Regular.ttf");

/// 嵌入的 JetBrains Mono 字体（等宽英文）
const JETBRAINS_MONO: &[u8] = include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf");

/// 设置自定义字体
///
/// 加载嵌入的字体文件并配置 egui 的字体系统。
/// 保留 egui 默认字体以支持 emoji 显示。
///
/// # 字体配置
/// - **Proportional（比例字体）**: LXGW WenKai → Noto Sans SC → JetBrains Mono → egui 默认字体（emoji）
/// - **Monospace（等宽字体）**: JetBrains Mono → LXGW WenKai → Noto Sans SC → egui 默认字体（emoji）
///
/// # 示例
/// ```ignore
/// use tail_gui::setup_fonts;
///
/// eframe::run_native(
///     "My App",
///     options,
///     Box::new(|cc| {
///         setup_fonts(&cc.egui_ctx);
///         Ok(Box::new(MyApp::new(cc)))
///     }),
/// )
/// ```
pub fn setup_fonts(ctx: &Context) {
    // 获取默认字体定义，保留 egui 内置的 emoji 支持
    let mut fonts = FontDefinitions::default();
    
    // 加载 LXGW WenKai 字体（用于中文显示）
    fonts.font_data.insert(
        "lxgw".to_owned(),
        FontData::from_static(LXGW),
    );
    
    // 加载 Noto Sans SC 字体（用于中文显示，作为后备）
    fonts.font_data.insert(
        "noto_sans_sc".to_owned(),
        FontData::from_static(NOTO_SANS_SC),
    );
    
    // 加载 JetBrains Mono 字体（用于等宽显示）
    fonts.font_data.insert(
        "jetbrains_mono".to_owned(),
        FontData::from_static(JETBRAINS_MONO),
    );
    
    // 设置字体家族优先级
    // Proportional: 用于普通文本
    // 在默认字体列表前面插入自定义字体，保留 emoji 支持
    if let Some(proportional) = fonts.families.get_mut(&FontFamily::Proportional) {
        // 在最前面插入中文字体（倒序插入，最后插入的在最前面）
        proportional.insert(0, "jetbrains_mono".to_owned());
        proportional.insert(0, "noto_sans_sc".to_owned());
        proportional.insert(0, "lxgw".to_owned());
    }
    
    // Monospace: 用于等宽文本（数字、代码等）
    if let Some(monospace) = fonts.families.get_mut(&FontFamily::Monospace) {
        // 在最前面插入字体
        monospace.insert(0, "noto_sans_sc".to_owned());
        monospace.insert(0, "lxgw".to_owned());
        monospace.insert(0, "jetbrains_mono".to_owned());
    }
    
    tracing::info!("字体加载完成: LXGW WenKai + Noto Sans SC (中文) + JetBrains Mono (等宽) + egui 默认字体 (emoji)");
    
    ctx.set_fonts(fonts);
}