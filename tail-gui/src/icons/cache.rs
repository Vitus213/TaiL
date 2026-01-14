//! TaiL GUI - 图标缓存模块

use std::collections::HashMap;
use std::path::PathBuf;

/// 图标缓存
pub struct IconCache {
    /// 缓存的图标路径
    icon_paths: HashMap<String, Option<PathBuf>>,
    /// 默认图标映射（应用名 -> 文本标签）
    default_icons: HashMap<String, &'static str>,
}

impl Default for IconCache {
    fn default() -> Self {
        Self::new()
    }
}

impl IconCache {
    pub fn new() -> Self {
        let mut default_icons = HashMap::new();
        
        // 常见应用的默认图标（使用文本缩写代替 emoji）
        default_icons.insert("code".to_string(), "VS");
        default_icons.insert("visual studio code".to_string(), "VS");
        default_icons.insert("vscode".to_string(), "VS");
        default_icons.insert("firefox".to_string(), "FF");
        default_icons.insert("chrome".to_string(), "CH");
        default_icons.insert("chromium".to_string(), "CR");
        default_icons.insert("brave".to_string(), "BR");
        default_icons.insert("microsoft-edge".to_string(), "ED");
        default_icons.insert("edge".to_string(), "ED");
        default_icons.insert("terminal".to_string(), ">_");
        default_icons.insert("konsole".to_string(), ">_");
        default_icons.insert("alacritty".to_string(), ">_");
        default_icons.insert("kitty".to_string(), ">_");
        default_icons.insert("wezterm".to_string(), ">_");
        default_icons.insert("discord".to_string(), "DC");
        default_icons.insert("slack".to_string(), "SL");
        default_icons.insert("telegram".to_string(), "TG");
        default_icons.insert("wechat".to_string(), "WX");
        default_icons.insert("feishu".to_string(), "FS");
        default_icons.insert("bytedance-feishu".to_string(), "FS");
        default_icons.insert("spotify".to_string(), "SP");
        default_icons.insert("nautilus".to_string(), "FM");
        default_icons.insert("dolphin".to_string(), "FM");
        default_icons.insert("thunar".to_string(), "FM");
        default_icons.insert("steam".to_string(), "ST");
        default_icons.insert("obs".to_string(), "OB");
        default_icons.insert("obs studio".to_string(), "OB");
        default_icons.insert("gimp".to_string(), "GP");
        default_icons.insert("inkscape".to_string(), "IK");
        default_icons.insert("krita".to_string(), "KR");
        default_icons.insert("blender".to_string(), "BL");
        default_icons.insert("libreoffice".to_string(), "LO");
        default_icons.insert("thunderbird".to_string(), "TB");
        default_icons.insert("evolution".to_string(), "EV");
        default_icons.insert("vlc".to_string(), "VL");
        default_icons.insert("mpv".to_string(), "MP");
        default_icons.insert("zathura".to_string(), "ZA");
        default_icons.insert("evince".to_string(), "EV");
        default_icons.insert("okular".to_string(), "OK");
        default_icons.insert("neovim".to_string(), "NV");
        default_icons.insert("nvim".to_string(), "NV");
        default_icons.insert("vim".to_string(), "VI");
        default_icons.insert("emacs".to_string(), "EM");
        default_icons.insert("jetbrains".to_string(), "JB");
        default_icons.insert("idea".to_string(), "IJ");
        default_icons.insert("pycharm".to_string(), "PC");
        default_icons.insert("webstorm".to_string(), "WS");
        default_icons.insert("clion".to_string(), "CL");
        default_icons.insert("goland".to_string(), "GO");
        default_icons.insert("zed".to_string(), "ZD");
        default_icons.insert("dev.zed.zed".to_string(), "ZD");
        
        Self {
            icon_paths: HashMap::new(),
            default_icons,
        }
    }

    /// 获取应用的文本图标
    pub fn get_emoji(&self, app_name: &str) -> &'static str {
        let name_lower = app_name.to_lowercase();
        
        // 首先尝试精确匹配
        if let Some(icon) = self.default_icons.get(&name_lower) {
            return icon;
        }
        
        // 然后尝试部分匹配
        for (key, icon) in &self.default_icons {
            if name_lower.contains(key) || key.contains(&name_lower) {
                return icon;
            }
        }
        
        // 默认图标 - 取应用名前两个字符
        "AP"
    }

    /// 尝试从系统获取图标路径
    pub fn get_icon_path(&mut self, app_name: &str) -> Option<PathBuf> {
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
        
        // 图标搜索路径
        let icon_dirs = [
            "/usr/share/icons/hicolor/48x48/apps",
            "/usr/share/icons/hicolor/64x64/apps",
            "/usr/share/icons/hicolor/128x128/apps",
            "/usr/share/icons/hicolor/scalable/apps",
            "/usr/share/pixmaps",
        ];

        // 图标扩展名
        let extensions = ["png", "svg", "xpm"];

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
        self.find_icon_from_desktop(&name_lower)
    }

    /// 从 .desktop 文件获取图标
    fn find_icon_from_desktop(&self, app_name: &str) -> Option<PathBuf> {
        let desktop_dirs = [
            "/usr/share/applications",
            &format!("{}/.local/share/applications", std::env::var("HOME").unwrap_or_default()),
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
                        if let Some(icon) = self.parse_desktop_file(&path, app_name) {
                            return Some(icon);
                        }
                    }
                }
            }
        }

        None
    }

    /// 解析 .desktop 文件获取图标
    fn parse_desktop_file(&self, path: &PathBuf, app_name: &str) -> Option<PathBuf> {
        let content = std::fs::read_to_string(path).ok()?;
        
        // 检查是否是目标应用
        let mut is_target = false;
        let mut icon_name = None;

        for line in content.lines() {
            let line = line.trim();
            
            if line.starts_with("Name=") {
                let name = &line[5..];
                if name.to_lowercase().contains(app_name) {
                    is_target = true;
                }
            }
            
            if line.starts_with("Icon=") {
                icon_name = Some(line[5..].to_string());
            }
        }

        if !is_target {
            return None;
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
            "/usr/share/icons/hicolor/scalable/apps",
            "/usr/share/pixmaps",
        ];

        let extensions = ["png", "svg", "xpm", ""];

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
        self.icon_paths.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_emoji() {
        let cache = IconCache::new();
        
        assert_eq!(cache.get_emoji("code"), "💻");
        assert_eq!(cache.get_emoji("Firefox"), "🦊");
        assert_eq!(cache.get_emoji("unknown_app"), "📱");
    }

    #[test]
    fn test_partial_match() {
        let cache = IconCache::new();
        
        // 部分匹配测试
        assert_eq!(cache.get_emoji("Visual Studio Code"), "💻");
        assert_eq!(cache.get_emoji("Mozilla Firefox"), "🦊");
    }
}