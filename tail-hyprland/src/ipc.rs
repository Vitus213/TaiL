//! TaiL Hyprland IPC 客户端

use std::path::PathBuf;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::UnixStream;
use tracing::{debug, error, info, warn};

/// Hyprland IPC 错误
#[derive(Debug, Error)]
pub enum IpcError {
    #[error("Socket path not found. Is HYPRLAND_INSTANCE_SIGNATURE set?")]
    SocketNotFound,

    #[error("Failed to connect to socket: {0}")]
    ConnectionError(#[from] std::io::Error),

    #[error("Invalid event format: {0}")]
    InvalidEvent(String),
}

/// Hyprland 事件类型
#[derive(Debug, Clone)]
pub enum HyprlandEvent {
    /// 活动窗口变化
    ActiveWindowChanged { class: String, title: String },

    /// 窗口打开
    WindowOpened {
        address: String,
        workspace: String,
        class: String,
        title: String,
    },

    /// 窗口关闭
    WindowClosed { address: String },

    /// 工作区变化
    WorkspaceChanged { name: String },

    /// 窗口标题变化
    WindowTitleChanged { address: String, title: String },
}

/// Hyprland IPC 客户端
pub struct HyprlandIpc {
    socket_path: PathBuf,
}

impl HyprlandIpc {
    /// 创建新的 IPC 客户端
    pub fn new() -> Result<Self, IpcError> {
        let instance_signature =
            std::env::var("HYPRLAND_INSTANCE_SIGNATURE").map_err(|_| IpcError::SocketNotFound)?;

        debug!(
            instance_signature = %instance_signature,
            "HYPRLAND_INSTANCE_SIGNATURE 环境变量已获取"
        );

        let xdg_runtime = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());

        let socket_path = PathBuf::from(format!(
            "{}/hypr/{}/.socket2.sock",
            xdg_runtime, instance_signature
        ));

        info!(socket_path = %socket_path.display(), "Hyprland socket 路径");

        if !socket_path.exists() {
            error!(
                socket_path = %socket_path.display(),
                "Socket 文件不存在，请确认 Hyprland 正在运行"
            );
            return Err(IpcError::SocketNotFound);
        }

        debug!("Hyprland IPC 客户端创建成功");

        Ok(Self { socket_path })
    }

    /// 订阅事件流
    pub async fn subscribe_events<F>(&self, mut callback: F) -> Result<(), IpcError>
    where
        F: FnMut(HyprlandEvent),
    {
        info!("正在连接到 Hyprand IPC socket...");

        let stream = match UnixStream::connect(&self.socket_path).await {
            Ok(s) => {
                info!("Hyprand IPC socket 连接成功");
                s
            }
            Err(e) => {
                error!(
                    error = %e,
                    socket_path = %self.socket_path.display(),
                    "连接 Hyprand IPC socket 失败"
                );
                return Err(e.into());
            }
        };

        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        let mut event_count = 0;

        info!("Hyprand 事件订阅已启动");

        while reader.read_line(&mut line).await? > 0 {
            let line_str = line.trim();
            event_count += 1;

            debug!(
                event_count = event_count,
                raw_event = %line_str,
                "收到 Hyprand 事件"
            );

            if let Some(event) = Self::parse_event(line_str) {
                let event_name = match &event {
                    HyprlandEvent::ActiveWindowChanged { .. } => "ActiveWindowChanged",
                    HyprlandEvent::WindowOpened { .. } => "WindowOpened",
                    HyprlandEvent::WindowClosed { .. } => "WindowClosed",
                    HyprlandEvent::WorkspaceChanged { .. } => "WorkspaceChanged",
                    HyprlandEvent::WindowTitleChanged { .. } => "WindowTitleChanged",
                };
                debug!(event_type = event_name, "事件解析成功");
                callback(event);
            } else {
                warn!(
                    raw_event = %line_str,
                    event_count = event_count,
                    "事件解析失败，无法识别的事件格式"
                );
            }

            line.clear();
        }

        warn!("Hyprand IPC 连接已关闭（EOF）");
        Ok(())
    }

    /// 解析 Hyprland 事件
    fn parse_event(event: &str) -> Option<HyprlandEvent> {
        // 事件格式: EVENT>>DATA
        let parts: Vec<&str> = event.split(">>").collect();
        if parts.len() != 2 {
            return None;
        }

        let event_type = parts[0];
        let data = parts[1];

        match event_type {
            "activewindow" => {
                // 格式: WINDOWCLASS,WINDOWTITLE
                let window_parts: Vec<&str> = data.splitn(2, ',').collect();
                if window_parts.len() == 2 {
                    Some(HyprlandEvent::ActiveWindowChanged {
                        class: window_parts[0].to_string(),
                        title: window_parts[1].to_string(),
                    })
                } else {
                    None
                }
            }
            "openwindow" => {
                // 格式: WINDOWADDRESS,WORKSPACENAME,WINDOWCLASS,WINDOWTITLE
                let parts: Vec<&str> = data.splitn(4, ',').collect();
                if parts.len() == 4 {
                    Some(HyprlandEvent::WindowOpened {
                        address: parts[0].to_string(),
                        workspace: parts[1].to_string(),
                        class: parts[2].to_string(),
                        title: parts[3].to_string(),
                    })
                } else {
                    None
                }
            }
            "closewindow" => Some(HyprlandEvent::WindowClosed {
                address: data.to_string(),
            }),
            "workspace" | "workspacev2" => Some(HyprlandEvent::WorkspaceChanged {
                name: data.to_string(),
            }),
            "windowtitle" => {
                // windowtitle 事件只有 window address
                // 实际标题需要通过 hyprctl 查询，这里暂时返回空标题
                Some(HyprlandEvent::WindowTitleChanged {
                    address: data.to_string(),
                    title: String::new(),
                })
            }
            "windowtitlev2" => {
                // 格式: WINDOWADDRESS,WINDOWTITLE
                let parts: Vec<&str> = data.splitn(2, ',').collect();
                if parts.len() == 2 {
                    Some(HyprlandEvent::WindowTitleChanged {
                        address: parts[0].to_string(),
                        title: parts[1].to_string(),
                    })
                } else {
                    None
                }
            }
            _ => {
                debug!("Unhandled event type: {}", event_type);
                None
            }
        }
    }
}

impl Default for HyprlandIpc {
    fn default() -> Self {
        Self::new().expect("Failed to create Hyprland IPC")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_active_window_event() {
        let event_str = "activewindow>>firefox,GitHub - Mozilla Firefox";
        let event = HyprlandIpc::parse_event(event_str);

        assert!(event.is_some());
        match event.unwrap() {
            HyprlandEvent::ActiveWindowChanged { class, title } => {
                assert_eq!(class, "firefox");
                assert_eq!(title, "GitHub - Mozilla Firefox");
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_parse_open_window_event() {
        let event_str = "openwindow>>0x12345,1,kitty,Terminal";
        let event = HyprlandIpc::parse_event(event_str);

        assert!(event.is_some());
        match event.unwrap() {
            HyprlandEvent::WindowOpened {
                address,
                workspace,
                class,
                title,
            } => {
                assert_eq!(address, "0x12345");
                assert_eq!(workspace, "1");
                assert_eq!(class, "kitty");
                assert_eq!(title, "Terminal");
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_parse_close_window_event() {
        let event_str = "closewindow>>0x12345";
        let event = HyprlandIpc::parse_event(event_str);

        assert!(event.is_some());
        match event.unwrap() {
            HyprlandEvent::WindowClosed { address } => {
                assert_eq!(address, "0x12345");
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_parse_workspace_event() {
        let event_str = "workspace>>2";
        let event = HyprlandIpc::parse_event(event_str);

        assert!(event.is_some());
        match event.unwrap() {
            HyprlandEvent::WorkspaceChanged { name } => {
                assert_eq!(name, "2");
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_parse_window_title_v2_event() {
        let event_str = "windowtitlev2>>0x12345,New Title";
        let event = HyprlandIpc::parse_event(event_str);

        assert!(event.is_some());
        match event.unwrap() {
            HyprlandEvent::WindowTitleChanged { address, title } => {
                assert_eq!(address, "0x12345");
                assert_eq!(title, "New Title");
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_parse_invalid_event() {
        let event_str = "invalid>>data";
        let event = HyprlandIpc::parse_event(event_str);
        assert!(event.is_none());
    }

    #[test]
    fn test_parse_malformed_event() {
        let event_str = "activewindow";
        let event = HyprlandIpc::parse_event(event_str);
        assert!(event.is_none());
    }

    #[test]
    fn test_parse_active_window_with_comma_in_title() {
        let event_str = "activewindow>>firefox,GitHub - Mozilla Firefox, Pull Request #123";
        let event = HyprlandIpc::parse_event(event_str);

        assert!(event.is_some());
        match event.unwrap() {
            HyprlandEvent::ActiveWindowChanged { class, title } => {
                assert_eq!(class, "firefox");
                assert_eq!(title, "GitHub - Mozilla Firefox, Pull Request #123");
            }
            _ => panic!("Wrong event type"),
        }
    }
}
