//! TaiL GUI - 图标缓存模块
//!
//! 提供应用图标的加载、缓存和显示功能。
//! 支持从系统图标目录和 .desktop 文件中查找图标。

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use egui::{ColorImage, TextureHandle, TextureOptions, Context};

/// 图标大小（像素）
const ICON_SIZE: u32 = 32;

/// 图标缓存
pub struct IconCache {
    /// 缓存的纹理句柄
    textures: HashMap<String, Option<Arc<TextureHandle>>>,
    /// 图标路径缓存
    icon_paths: HashMap<String, Option<PathBuf>>,
    /// 默认图标文本（当找不到图标时使用）
    default_labels: HashMap<String, &'static str>,
}

impl Default for IconCache {
    fn default() -> Self {
        Self::new()
    }
}

impl IconCache {
    pub fn new() -> Self {
        let mut default_labels = HashMap::new();
        
        // 常见应用的默认文本标签
        default_labels.insert("code".to_string(), "VS");
        default_labels.insert("visual studio code".to_string(), "VS");
        default_labels.insert("vscode".to_string(), "VS");
        default_labels.insert("firefox".to_string(), "FF");
        default_labels.insert("chrome".to_string(), "CH");
        default_labels.insert("chromium".to_string(), "CR");
        default_labels.insert("brave".to_string(), "BR");
        default_labels.insert("microsoft-edge".to_string(), "ED");
        default_labels.insert("edge".to_string(), "ED");
        default_labels.insert("terminal".to_string(), ">_");
        default_labels.insert("konsole".to_string(), ">_");
        default_labels.insert("alacritty".to_string(), ">_");
        default_labels.insert("kitty".to_string(), ">_");
        default_labels.insert("wezterm".to_string(), ">_");
        default_labels.insert("discord".to_string(), "DC");
        default_labels.insert("slack".to_string(), "SL");
        default_labels.insert("telegram".to_string(), "TG");
        default_labels.insert("wechat".to_string(), "WX");
        default_labels.insert("feishu".to_string(), "FS");
        default_labels.insert("bytedance-feishu".to_string(), "FS");
        default_labels.insert("spotify".to_string(), "SP");
        default_labels.insert("nautilus".to_string(), "FM");
        default_labels.insert("dolphin".to_string(), "FM");
        default_labels.insert("thunar".to_string(), "FM");
        default_labels.insert("steam".to_string(), "ST");
        default_labels.insert("obs".to_string(), "OB");
        default_labels.insert("obs studio".to_string(), "OB");
        default_labels.insert("gimp".to_string(), "GP");
        default_labels.insert("inkscape".to_string(), "IK");
        default_labels.insert("krita".to_string(), "KR");
        default_labels.insert("blender".to_string(), "BL");
        default_labels.insert("libreoffice".to_string(), "LO");
        default_labels.insert("thunderbird".to_string(), "TB");
        default_labels.insert("evolution".to_string(), "EV");
        default_labels.insert("vlc".to_string(), "VL");
        default_labels.insert("mpv".to_string(), "MP");
        default_labels.insert("zathura".to_string(), "ZA");
        default_labels.insert("evince".to_string(), "EV");
        default_labels.insert("okular".to_string(), "OK");
        default_labels.insert("neovim".to_string(), "NV");
        default_labels.insert("nvim".to_string(), "NV");
        default_labels.insert("vim".to_string(), "VI");
        default_labels.insert("emacs".to_string(), "EM");
        default_labels.insert("jetbrains".to_string(), "JB");
        default_labels.insert("idea".to_string(), "IJ");
        default_labels.insert("pycharm".to_string(), "PC");
        default_labels.insert("webstorm".to_string(), "WS");
        default_labels.insert("clion".to_string(), "CL");
        default_labels.insert("goland".to_string(), "GO");
        default_labels.insert("zed".to_string(), "ZD");
        default_labels.insert("dev.zed.zed".to_string(), "ZD");
        
        Self {
            textures: HashMap::new(),
            icon_paths: HashMap::new(),
            default_labels,
        }
    }

    /// 获取应用的文本标签（当没有图标时使用）
    pub fn get_emoji(&self, app_name: &str) -> &'static str {
        let name_lower = app_name.to_lowercase();
        
        // 首先尝试精确匹配
        if let Some(label) = self.default_labels.get(&name_lower) {
            return label;
        }
        
        // 然后尝试部分匹配
        for (key, label) in &self.default_labels {
            if name_lower.contains(key) || key.contains(&name_lower) {
                return label;
            }
        }
        
        // 默认标签
        "AP"
    }

    /// 获取应用图标的纹理句柄
    pub fn get_texture(&mut self, ctx: &Context, app_name: &str) -> Option<Arc<TextureHandle>> {
        let name_lower = app_name.to_lowercase();
        
        // 检查纹理缓存
        if let Some(cached) = self.textures.get(&name_lower) {
            return cached.clone();
        }
        
        // 尝试加载图标
        let texture = self.load_icon_texture(ctx, &name_lower);
        self.textures.insert(name_lower, texture.clone());
        texture
    }

    /// 加载图标并创建纹理
    fn load_icon_texture(&mut self, ctx: &Context, app_name: &str) -> Option<Arc<TextureHandle>> {
        // 获取图标路径
        let icon_path = self.get_icon_path(app_name)?;
        
        // 加载图片
        let image = self.load_image(&icon_path)?;
        
        // 创建纹理
        let texture = ctx.load_texture(
            format!("icon_{}", app_name),
            image,
            TextureOptions::LINEAR,
        );
        
        Some(Arc::new(texture))
    }

    /// 加载图片文件
    fn load_image(&self, path: &PathBuf) -> Option<ColorImage> {
        let extension = path.extension()?.to_str()?.to_lowercase();
        
        match extension.as_str() {
            "png" | "jpg" | "jpeg" | "ico" => {
                let img = image::open(path).ok()?;
                let img = img.resize_exact(ICON_SIZE, ICON_SIZE, image::imageops::FilterType::Lanczos3);
                let rgba = img.to_rgba8();
                let size = [rgba.width() as usize, rgba.height() as usize];
                let pixels = rgba.into_raw();
                Some(ColorImage::from_rgba_unmultiplied(size, &pixels))
            }
            "svg" => {
                // SVG 需要额外的库来渲染，暂时跳过
                None
            }
            _ => None,
        }
    }

    /// 获取图标路径
    fn get_icon_path(&mut self, app_name: &str) -> Option<PathBuf> {
        // 检查缓存
        if let Some(cached) = self.icon_paths.get(app_name) {
            return cached.clone();
        }

        // 尝试查找图标
        let icon_path = self.find_icon(app_name);
        self.icon_paths.insert(app_name.to_string(), icon_path.clone());
        icon_path
    }

    /// 在系统中查找图标
    fn find_icon(&self, app_name: &str) -> Option<PathBuf> {
        let name_lower = app_name.to_lowercase();
        
        // 图标搜索路径（按优先级排序）
        let icon_dirs = [
            "/usr/share/icons/hicolor/48x48/apps",
            "/usr/share/icons/hicolor/64x64/apps",
            "/usr/share/icons/hicolor/32x32/apps",
            "/usr/share/icons/hicolor/128x128/apps",
            "/usr/share/icons/hicolor/256x256/apps",
            "/usr/share/pixmaps",
            "/usr/share/icons/Adwaita/48x48/apps",
            "/usr/share/icons/breeze/apps/48",
        ];

        // 图标扩展名（按优先级排序）
        let extensions = ["png", "svg", "xpm", "ico"];

        // 尝试直接匹配
        for dir in &icon_dirs {
            let dir_path = PathBuf::from(dir);
            if !dir_path.exists() {
                continue;
            }

            for ext in &extensions {
                let icon_path = dir_path.join(format!("{}.{}", name_lower, ext));
                if icon_path.exists() {
                    return Some(icon_path);
                }
            }
        }

        // 尝试从 .desktop 文件获取图标
        if let Some(icon) = self.find_icon_from_desktop(&name_lower) {
            return Some(icon);
        }

        // 尝试模糊匹配
        for dir in &icon_dirs {
            let dir_path = PathBuf::from(dir);
            if !dir_path.exists() {
                continue;
            }

            if let Ok(entries) = std::fs::read_dir(&dir_path) {
                for entry in entries.flatten() {
                    let file_name = entry.file_name();
                    let file_name_str = file_name.to_string_lossy().to_lowercase();
                    if file_name_str.contains(&name_lower) {
                        let path = entry.path();
                        if let Some(ext) = path.extension() {
                            if extensions.contains(&ext.to_str().unwrap_or("")) {
                                return Some(path);
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// 从 .desktop 文件获取图标
    fn find_icon_from_desktop(&self, app_name: &str) -> Option<PathBuf> {
        let home = std::env::var("HOME").unwrap_or_default();
        let desktop_dirs = [
            "/usr/share/applications".to_string(),
            format!("{}/.local/share/applications", home),
            "/var/lib/flatpak/exports/share/applications".to_string(),
            format!("{}/.local/share/flatpak/exports/share/applications", home),
        ];

        for dir in &desktop_dirs {
            let dir_path = PathBuf::from(dir);
            if !dir_path.exists() {
                continue;
            }

            // 查找匹配的 .desktop 文件
            if let Ok(entries) = std::fs::read_dir(&dir_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map_or(false, |e| e == "desktop") {
                        let file_name = path.file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("")
                            .to_lowercase();
                        
                        // 检查文件名是否匹配
                        if file_name.contains(app_name) || app_name.contains(&file_name) {
                            if let Some(icon) = self.parse_desktop_file(&path) {
                                return Some(icon);
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// 解析 .desktop 文件获取图标
    fn parse_desktop_file(&self, path: &PathBuf) -> Option<PathBuf> {
        let content = std::fs::read_to_string(path).ok()?;
        
        let mut icon_name = None;

        for line in content.lines() {
            let line = line.trim();
            
            if line.starts_with("Icon=") {
                icon_name = Some(line[5..].to_string());
                break;
            }
        }

        let icon_name = icon_name?;
        
        // 如果是绝对路径
        if icon_name.starts_with('/') {
            let path = PathBuf::from(&icon_name);
            if path.exists() {
                return Some(path);
            }
        }

        // 在图标目录中查找
        let icon_dirs = [
            "/usr/share/icons/hicolor/48x48/apps",
            "/usr/share/icons/hicolor/64x64/apps",
            "/usr/share/icons/hicolor/128x128/apps",
            "/usr/share/icons/hicolor/256x256/apps",
            "/usr/share/pixmaps",
        ];

        let extensions = ["png", "svg", "xpm", "ico", ""];

        for dir in &icon_dirs {
            let dir_path = PathBuf::from(dir);
            if !dir_path.exists() {
                continue;
            }

            for ext in &extensions {
                let icon_path = if ext.is_empty() {
                    dir_path.join(&icon_name)
                } else {
                    dir_path.join(format!("{}.{}", icon_name, ext))
                };
                
                if icon_path.exists() {
                    return Some(icon_path);
                }
            }
        }

        None
    }

    /// 清除缓存
    pub fn clear(&mut self) {
        self.textures.clear();
        self.icon_paths.clear();
    }
}

/// 图标显示组件
pub struct AppIcon<'a> {
    app_name: &'a str,
    size: f32,
}

impl<'a> AppIcon<'a> {
    pub fn new(app_name: &'a str) -> Self {
        Self {
            app_name,
            size: 24.0,
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// 显示图标
    pub fn show(self, ui: &mut egui::Ui, icon_cache: &mut IconCache) -> egui::Response {
        // 尝试获取纹理
        if let Some(texture) = icon_cache.get_texture(ui.ctx(), self.app_name) {
            let image = egui::Image::new(&*texture)
                .fit_to_exact_size(egui::Vec2::splat(self.size));
            ui.add(image)
        } else {
            // 显示文本标签作为后备
            let label = icon_cache.get_emoji(self.app_name);
            let (rect, response) = ui.allocate_exact_size(
                egui::Vec2::splat(self.size),
                egui::Sense::hover(),
            );
            
            if ui.is_rect_visible(rect) {
                let painter = ui.painter();
                
                // 绘制圆形背景
                painter.circle_filled(
                    rect.center(),
                    self.size / 2.0,
                    egui::Color32::from_rgb(100, 100, 100),
                );
                
                // 绘制文本
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    label,
                    egui::FontId::proportional(self.size * 0.4),
                    egui::Color32::WHITE,
                );
            }
            
            response
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_emoji() {
        let cache = IconCache::new();
        
        assert_eq!(cache.get_emoji("code"), "VS");
        assert_eq!(cache.get_emoji("Firefox"), "FF");
        assert_eq!(cache.get_emoji("unknown_app"), "AP");
    }

    #[test]
    fn test_partial_match() {
        let cache = IconCache::new();
        
        // 部分匹配测试
        assert_eq!(cache.get_emoji("Visual Studio Code"), "VS");
        assert_eq!(cache.get_emoji("Mozilla Firefox"), "FF");
    }
}