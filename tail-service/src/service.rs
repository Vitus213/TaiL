//! TaiL Service - 后台服务

use anyhow::Result;
use chrono::{DateTime, Utc};
use std::time::Instant;
use tail_afk::{AfkDetector, AfkState};
use tail_core::traits::WindowEventRepository;
use tail_core::{db::Config as DbConfig, Repository, WindowEvent};
use tail_hyprland::{HyprlandEvent, HyprlandIpc};
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, warn};

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
        info!("正在创建 TaiL Service 实例");
        let config = DbConfig::default();
        debug!(db_path = %config.path, "数据库配置");

        let repo = Repository::new(&config)?;
        let afk_detector = AfkDetector::default();
        debug!("AFK 检测器已创建，默认超时: 300 秒");

        info!("TaiL Service 实例创建成功");

        Ok(Self {
            repo,
            afk_detector,
            current_window: None,
        })
    }

    /// 使用自定义配置创建服务实例
    pub fn with_config(db_config: DbConfig, afk_timeout_secs: u64) -> Result<Self> {
        info!(
            db_path = %db_config.path,
            afk_timeout_secs = afk_timeout_secs,
            "正在创建 TaiL Service 实例（自定义配置）"
        );

        let repo = Repository::new(&db_config)?;
        let afk_detector = AfkDetector::new(afk_timeout_secs);

        info!("TaiL Service 实例创建成功");

        Ok(Self {
            repo,
            afk_detector,
            current_window: None,
        })
    }

    /// 运行服务
    pub async fn run(mut self) -> Result<()> {
        info!("TaiL Service 正在启动...");

        // 创建事件通道
        let (tx, mut rx) = mpsc::channel(100);
        debug!("事件通道已创建，容量: 100");

        // 在后台任务中订阅 Hyprland 事件
        info!("正在连接 Hyprand IPC...");
        let ipc = HyprlandIpc::new()?;
        info!("Hyprland IPC 客户端创建成功");

        let tx_hyprland = tx.clone();
        tokio::spawn(async move {
            debug!("Hyprland 事件订阅任务已启动");
            if let Err(e) = ipc
                .subscribe_events(move |event| {
                    debug!("收到 Hyprland 事件: {:?}", std::mem::discriminant(&event));
                    // 使用 try_send 避免阻塞
                    if let Err(e) = tx_hyprland.try_send(event) {
                        error!(error = %e, "发送事件到通道失败");
                    }
                })
                .await
            {
                error!(error = %e, "Hyprand IPC 错误，连接断开");
            }
            warn!("Hyprand 事件订阅任务已退出");
        });

        // 启动 AFK 检测任务
        debug!("AFK 检测任务已启动");
        tokio::spawn(async move {
            let mut check_interval = interval(Duration::from_secs(10));
            loop {
                check_interval.tick().await;
                // 这里可以添加实际的输入设备检测逻辑
                // 目前只是定期检查超时
            }
        });

        // 启动定期更新当前窗口时长的任务
        let tx_update = tx.clone();
        tokio::spawn(async move {
            debug!("定期更新任务已启动");
            let mut update_interval = interval(Duration::from_secs(5));
            loop {
                update_interval.tick().await;
                // 通过通道发送更新信号
                if let Err(e) = tx_update.try_send(HyprlandEvent::WindowTitleChanged {
                    address: String::new(),
                    title: String::from("__UPDATE_CURRENT_WINDOW__"),
                }) {
                    debug!("发送更新信号失败: {}", e);
                }
            }
        });

        // 丢弃原始 tx，这样当所有发送者都关闭时，rx.recv() 会返回 None
        drop(tx);

        info!("TaiL Service 主循环已启动");
        // 处理事件
        while let Some(event) = rx.recv().await {
            // 处理定期更新信号
            if matches!(&event, HyprlandEvent::WindowTitleChanged { title, .. } if title == "__UPDATE_CURRENT_WINDOW__")
            {
                if let Err(e) = self.update_current_window_duration().await {
                    error!(error = %e, "更新当前窗口时长失败");
                }
                continue;
            }

            if let Err(e) = self.handle_event(event).await {
                error!(error = %e, "处理事件失败");
            }
        }

        info!("TaiL Service 主循环已退出");
        Ok(())
    }

    /// 处理 Hyprland 事件
    async fn handle_event(&mut self, event: HyprlandEvent) -> Result<()> {
        match event {
            HyprlandEvent::ActiveWindowChanged { class, title } => {
                info!("Active window changed: {} - {}", class, title);
                self.record_window_change(class, title, "unknown".to_string())
                    .await?;
            }
            HyprlandEvent::WindowOpened {
                workspace,
                class,
                title,
                ..
            } => {
                debug!(
                    "Window opened: {} - {} on workspace {}",
                    class, title, workspace
                );
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
        debug!(
            app_name = %app_name,
            window_title = %window_title,
            workspace = %workspace,
            "记录窗口切换"
        );

        let now = Utc::now();
        let now_instant = Instant::now();

        // 窗口切换表示用户活动，更新 AFK 检测器
        self.afk_detector.record_activity();
        debug!("用户活动已记录（窗口切换）");

        // 检查 AFK 状态（现在应该是活跃状态，因为刚刚记录了活动）
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
                    match WindowEventRepository::update_duration(
                        &self.repo,
                        event_id,
                        duration_secs,
                    )
                    .await
                    {
                        Ok(_) => {
                            info!(
                                app_name = %prev_window.app_name,
                                duration_secs = duration_secs,
                                event_id = event_id,
                                "窗口事件已更新"
                            );
                        }
                        Err(e) => {
                            error!(
                                error = %e,
                                event_id = event_id,
                                app_name = %prev_window.app_name,
                                "更新窗口事件时长失败"
                            );
                        }
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
        match WindowEventRepository::insert(&self.repo, &event).await {
            Ok(event_id) => {
                info!(
                    app_name = %app_name,
                    event_id = event_id,
                    is_afk = is_afk,
                    "新窗口事件已插入"
                );

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
                error!(
                    error = %e,
                    app_name = %app_name,
                    "插入窗口事件失败"
                );
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
                    WindowEventRepository::update_duration(&self.repo, event_id, duration_secs)
                        .await?;
                    debug!(
                        "Updated current window duration: {} ({} seconds)",
                        window.app_name, duration_secs
                    );
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
                WindowEventRepository::update_duration(&self.repo, event_id, duration_secs).await?;
                info!(
                    "Flushed current window: {} ({} seconds)",
                    window.app_name, duration_secs
                );
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
    // 检测是否在 systemd 环境下运行，禁用 ANSI 颜色代码
    let is_running_under_systemd = std::env::var("INVOCATION_ID").is_ok();

    use tail_core::logging::LogOutput;
    let output = if is_running_under_systemd {
        LogOutput::SystemdJournal
    } else {
        LogOutput::Stdout
    };

    tail_core::logging::init_logging(output, "info");

    // 创建 runtime
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

    rt.block_on(async {
        if let Err(e) = run_service().await {
            error!(error = %e, "Service error");
            std::process::exit(1);
        }
    });
}

/// 运行服务
async fn run_service() -> anyhow::Result<()> {
    info!("正在启动 TaiL Service...");

    let service = TailService::new()?;

    // 直接运行服务，Ctrl+C 会自动终止进程
    service.run().await?;

    Ok(())
}
