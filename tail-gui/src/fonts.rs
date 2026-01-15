//! 字体配置模块
//!
//! 提供 TaiL GUI 的字体加载和配置功能。
//! 优先从系统路径加载字体（通过 TAIL_FONT_PATH 环境变量），
//! 回退到嵌入字体确保跨平台一致的显示效果。

use egui::{Context, FontData, FontDefinitions, FontFamily};
use std::path::Path;

/// 嵌入的 LXGW WenKai 字体（中文，霞鹜文楷）
const LXGW: &[u8] = include_bytes!("../assets/fonts/LXGWWenKai-Regular.ttf");

/// 嵌入的 JetBrains Mono 字体（等宽英文）
const JETBRAINS_MONO: &[u8] = include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf");

/// 嵌入的 Noto Sans SC 字体（中文后备，包含更多字符）
const NOTO_SANS_SC: &[u8] = include_bytes!("../assets/fonts/NotoSansSC-Regular.ttf");

/// 从文件加载字体数据，失败时使用嵌入的字体作为后备
fn load_font_or_fallback(
    font_path: Option<&Path>,
    fallback_data: &'static [u8],
    font_name: &str,
) -> FontData {
    if let Some(path) = font_path {
        if let Ok(bytes) = std::fs::read(path) {
            tracing::info!("从文件加载字体: {} -> {}", font_name, path.display());
            return FontData::from_owned(bytes);
        }
        tracing::warn!(
            "无法从文件加载字体: {} -> {}，使用嵌入字体",
            font_name,
            path.display()
        );
    }
    FontData::from_static(fallback_data)
}

/// 查找字体文件
///
/// 按以下顺序查找：
/// 1. TAIL_FONT_PATH 环境变量指定的目录（冒号分隔的多个路径，递归搜索）
/// 2. 系统字体目录（递归搜索）
/// 3. 返回 None（使用嵌入字体）
fn find_font_file(filename: &str) -> Option<std::path::PathBuf> {
    // 1. 首先检查 TAIL_FONT_PATH 环境变量（Nix 构建时设置）
    // TAIL_FONT_PATH 是冒号分隔的路径列表
    if let Ok(font_path) = std::env::var("TAIL_FONT_PATH") {
        for dir in font_path.split(':') {
            if dir.is_empty() {
                continue;
            }
            let base_path = std::path::PathBuf::from(dir);

            // 直接检查
            let direct_path = base_path.join(filename);
            if direct_path.exists() {
                return Some(direct_path);
            }

            // 递归搜索（限制深度避免性能问题）
            if let Ok(path) = recursive_find(&base_path, filename, 3) {
                return Some(path);
            }
        }
    }

    // 2. 系统字体目录
    let system_paths = [
        "/run/current-system/sw/share/fonts",
        "/nix/var/nix/profiles/system/sw/share/fonts",
        "~/.local/share/fonts",
        "~/.fonts",
        "/usr/share/fonts",
        "/usr/local/share/fonts",
    ];

    for base in &system_paths {
        let expanded = shellexpand::tilde(base);
        let base_path = std::path::PathBuf::from(expanded.as_ref());

        // 直接检查
        let direct_path = base_path.join(filename);
        if direct_path.exists() {
            return Some(direct_path);
        }

        // 递归搜索
        if let Ok(path) = recursive_find(&base_path, filename, 3) {
            return Some(path);
        }
    }

    None
}

/// 递归查找字体文件
fn recursive_find(dir: &std::path::Path, filename: &str, max_depth: usize) -> std::io::Result<std::path::PathBuf> {
    let entries = std::fs::read_dir(dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() && max_depth > 0 {
            // 递归搜索子目录
            if let Ok(found) = recursive_find(&path, filename, max_depth - 1) {
                return Ok(found);
            }
        } else if path.is_file() {
            if let Some(name) = path.file_name() {
                if name == filename {
                    return Ok(path);
                }
            }
        }
    }

    Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Font not found"))
}

/// 设置自定义字体
///
/// 加载字体文件并配置 egui 的字体系统。
/// 注意：egui 不支持彩色 emoji 字体，因此使用文字图标代替 emoji。
///
/// # 字体配置
/// - **Proportional（比例字体）**: LXGW WenKai (中文) → JetBrains Mono (英文) → Noto Sans SC (后备)
/// - **Monospace（等宽字体）**: JetBrains Mono → LXGW WenKai → Noto Sans SC
///
/// # 字体查找顺序
/// 1. `TAIL_FONT_PATH` 环境变量指定的目录
/// 2. 系统字体目录
/// 3. 嵌入的字体（后备）
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
    // 创建新的字体定义
    let mut fonts = FontDefinitions::default();

    // 字体文件名（对应 nixpkgs 中的字体包）
    // - lxgw-wenkai: LXGWWenKai-Regular.ttf
    // - jetbrains-mono: JetBrainsMono-Regular.ttf
    // - noto-fonts-cjk-sans: NotoSansCJK-Regular.ttc (包含多种变体)

    // 加载 LXGW WenKai 字体（用于中文显示）
    let lxgw_path = find_font_file("LXGWWenKai-Regular.ttf");
    fonts
        .font_data
        .insert("lxgw".to_owned(), load_font_or_fallback(
            lxgw_path.as_deref(),
            LXGW,
            "LXGW WenKai",
        ));

    // 加载 JetBrains Mono 字体（用于英文和等宽显示）
    let jetbrains_path = find_font_file("JetBrainsMono-Regular.ttf");
    fonts.font_data.insert(
        "jetbrains_mono".to_owned(),
        load_font_or_fallback(jetbrains_path.as_deref(), JETBRAINS_MONO, "JetBrains Mono"),
    );

    // 加载 Noto Sans CJK 字体（用于中文后备，包含更多字符）
    // 注意：noto-fonts-cjk-sans 使用 .ttc 格式，包含多种字体
    let noto_path = find_font_file("NotoSansCJK-Regular.ttc")
        .or_else(|| find_font_file("NotoSansSC-Regular.ttf"));
    fonts.font_data.insert(
        "noto_sans_sc".to_owned(),
        load_font_or_fallback(noto_path.as_deref(), NOTO_SANS_SC, "Noto Sans CJK"),
    );

    // 设置字体家族优先级
    // Proportional: 用于普通文本
    // 顺序：LXGW (中文) → JetBrains Mono (英文) → Noto Sans SC (后备)
    if let Some(proportional) = fonts.families.get_mut(&FontFamily::Proportional) {
        // 在最前面插入自定义字体（倒序插入，最后插入的在最前面）
        proportional.insert(0, "noto_sans_sc".to_owned());
        proportional.insert(0, "jetbrains_mono".to_owned());
        proportional.insert(0, "lxgw".to_owned());
    }

    // Monospace: 用于等宽文本（数字、代码等）
    if let Some(monospace) = fonts.families.get_mut(&FontFamily::Monospace) {
        // 在最前面插入字体
        monospace.insert(0, "noto_sans_sc".to_owned());
        monospace.insert(0, "lxgw".to_owned());
        monospace.insert(0, "jetbrains_mono".to_owned());
    }

    tracing::info!(
        "字体加载完成: LXGW WenKai (中文) + JetBrains Mono (英文) + Noto Sans CJK (后备)"
    );

    ctx.set_fonts(fonts);
}
