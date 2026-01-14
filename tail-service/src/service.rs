//! TaiL Service - 后台服务

use anyhow::Result;
use chrono::{DateTime, Utc};
use std::time::Instant;
use tail_core::{DbConfig, Repository, WindowEvent};
use tail_hyprland::{HyprlandIpc, HyprlandEvent};
use tail_afk::{AfkDetector, AfkState};
use tracing::{debug, error, info};
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};

/// 活动窗口信息
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ActiveWindow {
    app_name: String,
    window_title: String,
    workspace: String,
    start_time: DateTime<Utc>,
    start_instant: Instant,
    event_id: Option<i64>,
}

/// 后台服务
pub struct TailService {
    repo: Repository,
    afk_detector: AfkDetector,
    current_window: Option<ActiveWindow>,
}

impl TailService {
    /// 创建新的服务实例
    pub fn new() -> Result<Self> {
        let config = DbConfig::default();
        let repo = Repository::new(&config)?;
        let afk_detector = AfkDetector::default();

        Ok(Self {
            repo,
            afk_detector,
            current_window: None,
        })
    }

    /// 使用自定义配置创建服务实例
    pub fn with_config(db_config: DbConfig, afk_timeout_secs: u64) -> Result<Self> {
        let repo = Repository::new(&db_config)?;
        let afk_detector = AfkDetector::new(afk_timeout_secs);

        Ok(Self {
            repo,
            afk_detector,
            current_window: None,
        })
    }

    /// 运行服务
    pub async fn run(mut self) -> Result<()> {
        info!("TaiL Service starting...");

        // 创建事件通道
        let (tx, mut rx) = mpsc::channel(100);

        // 在后台任务中订阅 Hyprland 事件
        let ipc = HyprlandIpc::new()?;
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            if let Err(e) = ipc.subscribe_events(move |event| {
                // 使用 try_send 避免阻塞
                if let Err(e) = tx_clone.try_send(event) {
                    error!("Failed to send event: {}", e);
                }
            }).await {
                error!("Hyprland IPC error: {}", e);
            }
        });

        // 启动 AFK 检测任务
        tokio::spawn(async move {
            let mut check_interval = interval(Duration::from_secs(10));
            loop {
                check_interval.tick().await;
                // 这里可以添加实际的输入设备检测逻辑
                // 目前只是定期检查超时
            }
        });

        // 启动定期更新当前窗口时长的任务
        let mut update_interval = interval(Duration::from_secs(5));
        tokio::spawn(async move {
            loop {
                update_interval.tick().await;
                // 通过通道发送更新信号
                if let Err(e) = tx.try_send(HyprlandEvent::WindowTitleChanged {
                    address: String::new(),
                    title: String::from("__UPDATE_CURRENT_WINDOW__")
                }) {
                    debug!("Failed to send update signal: {}", e);
                }
            }
        });

        // 处理事件
        while let Some(event) = rx.recv().await {
            // 处理定期更新信号
            if matches!(&event, HyprlandEvent::WindowTitleChanged { title, .. } if title == "__UPDATE_CURRENT_WINDOW__") {
                if let Err(e) = self.update_current_window_duration().await {
                    error!("Error updating current window duration: {}", e);
                }
                continue;
            }
            
            if let Err(e) = self.handle_event(event).await {
                error!("Error handling event: {}", e);
            }
        }

        Ok(())
    }

    /// 处理 Hyprland 事件
    async fn handle_event(&mut self, event: HyprlandEvent) -> Result<()> {
        match event {
            HyprlandEvent::ActiveWindowChanged { class, title } => {
                info!("Active window changed: {} - {}", class, title);
                self.record_window_change(class, title, "unknown".to_string()).await?;
            }
            HyprlandEvent::WindowOpened { workspace, class, title, .. } => {
                debug!("Window opened: {} - {} on workspace {}", class, title, workspace);
                // 窗口打开时可能会自动切换，等待 ActiveWindowChanged 事件
            }
            HyprlandEvent::WindowClosed { .. } => {
                debug!("Window closed");
                // 窗口关闭时可能会切换到其他窗口，等待 ActiveWindowChanged 事件
            }
            HyprlandEvent::WorkspaceChanged { name } => {
                debug!("Workspace changed to: {}", name);
                // 工作区切换时会触发 ActiveWindowChanged 事件
            }
            HyprlandEvent::WindowTitleChanged { title, .. } => {
                // 更新当前窗口标题
                if let Some(ref mut window) = self.current_window {
                    window.window_title = title;
                }
            }
        }
        Ok(())
    }

    /// 记录窗口切换
    async fn record_window_change(
        &mut self,
        app_name: String,
        window_title: String,
        workspace: String,
    ) -> Result<()> {
        let now = Utc::now();
        let now_instant = Instant::now();

        // 检查 AFK 状态
        let afk_state = self.afk_detector.check_state();
        let is_afk = matches!(afk_state, AfkState::Afk { .. });

        // 如果有当前窗口，计算其使用时长并保存
        if let Some(prev_window) = self.current_window.take() {
            let duration_secs = now_instant
                .duration_since(prev_window.start_instant)
                .as_secs() as i64;

            // 只有时长大于0才记录
            if duration_secs > 0 {
                if let Some(event_id) = prev_window.event_id {
                    // 更新已存在的事件
                    if let Err(e) = self.repo.update_window_event_duration(event_id, duration_secs) {
                        error!("Failed to update window event duration: {}", e);
                    } else {
                        info!(
                            "Updated window event: {} used for {} seconds",
                            prev_window.app_name, duration_secs
                        );
                    }
                }
            }
        }

        // 创建新的窗口事件（初始时长为0）
        let event = WindowEvent {
            id: None,
            timestamp: now,
            app_name: app_name.clone(),
            window_title: window_title.clone(),
            workspace: workspace.clone(),
            duration_secs: 0,
            is_afk,
        };

        // 插入新事件到数据库
        match self.repo.insert_window_event(&event) {
            Ok(event_id) => {
                info!("Inserted new window event: {} (id: {})", app_name, event_id);
                
                // 更新当前窗口
                self.current_window = Some(ActiveWindow {
                    app_name,
                    window_title,
                    workspace,
                    start_time: now,
                    start_instant: now_instant,
                    event_id: Some(event_id),
                });
            }
            Err(e) => {
                error!("Failed to insert window event: {}", e);
            }
        }

        Ok(())
    }

    /// 记录用户活动（用于 AFK 检测）
    pub fn record_activity(&mut self) {
        self.afk_detector.record_activity();
    }

    /// 获取当前 AFK 状态
    pub fn is_afk(&self) -> bool {
        self.afk_detector.is_afk()
    }

    /// 更新当前窗口的使用时长（不切换窗口）
    async fn update_current_window_duration(&mut self) -> Result<()> {
        if let Some(ref window) = self.current_window {
            let duration_secs = Instant::now()
                .duration_since(window.start_instant)
                .as_secs() as i64;

            if duration_secs > 0 {
                if let Some(event_id) = window.event_id {
                    self.repo.update_window_event_duration(event_id, duration_secs)?;
                    debug!("Updated current window duration: {} ({} seconds)", window.app_name, duration_secs);
                }
            }
        }
        Ok(())
    }

    /// 强制保存当前窗口的使用时长
    pub async fn flush_current_window(&mut self) -> Result<()> {
        if let Some(ref window) = self.current_window {
            let duration_secs = Instant::now()
                .duration_since(window.start_instant)
                .as_secs() as i64;

            if let Some(event_id) = window.event_id {
                self.repo.update_window_event_duration(event_id, duration_secs)?;
                info!("Flushed current window: {} ({} seconds)", window.app_name, duration_secs);
            }
        }
        Ok(())
    }
}

impl Default for TailService {
    fn default() -> Self {
        Self::new().expect("Failed to create TaiL Service")
    }
}

/// Service main entry point - 可以被其他 crate 调用
pub fn service_main() {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into())
        )
        .init();

    // 创建 runtime
    let rt = tokio::runtime::Runtime::new()
        .expect("Failed to create tokio runtime");

    rt.block_on(async {
        if let Err(e) = run_service().await {
            error!("Service error: {}", e);
            std::process::exit(1);
        }
    });
}

/// 运行服务
async fn run_service() -> anyhow::Result<()> {
    info!("Starting TaiL Service...");

    let service = TailService::new()?;
    
    // 直接运行服务，Ctrl+C 会自动终止进程
    service.run().await?;

    Ok(())
}
