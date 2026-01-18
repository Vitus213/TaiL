//! 异步桥接模块
//!
//! 实现 GUI 与后端服务之间的非阻塞通信
//!
//! 重构说明：现在使用新的用例层（StatsQueryUseCase）而非直接调用旧的 traits

use chrono::{DateTime, Utc};
use std::sync::Arc;
use tail_core::models::{Category, DailyGoal};
use tail_core::{AppUsage, CategoryUsage, RepositoryAdapter, StatsQueryUseCase, StatsQueryPort, dashboard_view_to_app_usage, stats_view_to_app_usage, AppError};
use tokio::sync::mpsc;

/// GUI 命令枚举
#[derive(Debug)]
pub enum GuiCommand {
    /// 刷新仪表板数据
    RefreshDashboard { start: DateTime<Utc>, end: DateTime<Utc> },

    /// 刷新统计数据
    RefreshStats { start: DateTime<Utc>, end: DateTime<Utc> },

    /// 刷新详细记录数据
    RefreshDetails { start: DateTime<Utc>, end: DateTime<Utc> },

    /// 加载分类页面数据
    LoadCategoriesData { start: DateTime<Utc>, end: DateTime<Utc> },

    /// 添加每日目标
    AddDailyGoal(DailyGoal),

    /// 删除每日目标
    DeleteDailyGoal(String),

    /// 设置应用别名
    SetAppAlias { app_name: String, alias: String },

    /// 删除应用别名
    DeleteAppAlias(String),

    /// 获取所有别名
    GetAllAliases,

    /// 添加分类
    AddCategory(Category),

    /// 更新分类
    UpdateCategory(Category),

    /// 删除分类
    DeleteCategory(i64),

    /// 设置应用分类
    SetAppCategories { app_name: String, category_ids: Vec<i64> },

    /// 从分类中移除应用
    RemoveAppFromCategory { app_name: String, category_id: i64 },

    /// 获取应用分类
    GetAppCategories(String),

    /// 获取所有分类
    GetAllCategories,

    /// 获取所有应用名称
    GetAllAppNames,

    /// 退出
    Shutdown,
}

/// GUI 响应枚举
#[derive(Debug)]
pub enum GuiResponse {
    /// 仪表板数据
    DashboardData(Result<Vec<AppUsage>, String>),

    /// 统计数据
    StatsData(Result<Vec<AppUsage>, String>),

    /// 详细记录数据
    DetailsData(Result<Vec<AppUsage>, String>),

    /// 分类页面数据
    CategoriesData(Result<CategoriesDataPayload, String>),

    /// 每日目标列表
    DailyGoals(Result<Vec<DailyGoal>, String>),

    /// 别名列表
    Aliases(Result<Vec<(String, String)>, String>),

    /// 分类列表
    Categories(Result<Vec<Category>, String>),

    /// 应用名称列表
    AppNames(Result<Vec<String>, String>),

    /// 应用分类列表
    AppCategories(Result<Vec<Category>, String>),

    /// 操作完成（无返回值）
    Done(Result<(), String>),

    /// 关闭确认
    ShutdownAck,
}

/// 分类页面数据载荷
#[derive(Debug, Clone)]
pub struct CategoriesDataPayload {
    pub category_usage: Vec<CategoryUsage>,
    pub categories: Vec<Category>,
    pub all_apps: Vec<String>,
    pub app_usage: Vec<AppUsage>,
}

/// 异步桥接器
///
/// 管理 GUI 与后台 tokio runtime 之间的通信
pub struct AsyncBridge {
    /// 命令发送端
    command_tx: mpsc::UnboundedSender<GuiCommand>,

    /// 响应接收端
    response_rx: mpsc::UnboundedReceiver<GuiResponse>,
}

impl AsyncBridge {
    /// 创建新的异步桥接器
    ///
    /// 启动一个后台线程运行 tokio runtime，处理来自 GUI 的命令
    ///
    /// 重构说明：使用新的用例层而非直接调用旧的 traits
    pub fn new(repo: Arc<tail_core::Repository>) -> Self {
        let (command_tx, mut command_rx) = mpsc::unbounded_channel::<GuiCommand>();
        let (response_tx, response_rx) = mpsc::unbounded_channel::<GuiResponse>();

        // 启动后台线程处理命令
        std::thread::spawn(move || {
            // 在后台线程创建 tokio runtime
            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .thread_name("tail-gui-worker")
                .enable_all()
                .build()
                .expect("Failed to create tokio runtime");

            // 在 runtime 中运行命令处理循环
            rt.block_on(async {
                tracing::info!("异步桥接器后台线程已启动");

                // 创建新的用例层实例（使用 RepositoryAdapter 连接旧仓储）
                let event_repo = RepositoryAdapter::from_repository(repo.clone());
                let stats_use_case = Arc::new(StatsQueryUseCase::new(event_repo));

                while let Some(cmd) = command_rx.recv().await {
                    match cmd {
                        GuiCommand::RefreshDashboard { start: _start, end: _end } => {
                            // 使用新的用例层获取仪表板数据
                            let result: Result<Vec<AppUsage>, String> = stats_use_case.get_dashboard().await
                                .map(|view| dashboard_view_to_app_usage(view))
                                .map_err(|e: AppError| e.to_string());

                            let _ = response_tx.send(GuiResponse::DashboardData(result));
                        }

                        GuiCommand::RefreshStats { start: _start, end: _end } => {
                            // 使用新的用例层获取统计数据
                            let nav = tail_core::domain::NavigationPath::new();
                            let result: Result<Vec<AppUsage>, String> = stats_use_case.get_stats(&nav).await
                                .map(|view| stats_view_to_app_usage(view))
                                .map_err(|e: AppError| e.to_string());

                            let _ = response_tx.send(GuiResponse::StatsData(result));
                        }

                        GuiCommand::RefreshDetails { start: _start, end: _end } => {
                            // 使用新的用例层获取详细统计数据
                            let nav = tail_core::domain::NavigationPath::new();
                            let result: Result<Vec<AppUsage>, String> = stats_use_case.get_stats(&nav).await
                                .map(|view| stats_view_to_app_usage(view))
                                .map_err(|e: AppError| e.to_string());

                            let _ = response_tx.send(GuiResponse::DetailsData(result));
                        }

                        GuiCommand::LoadCategoriesData { start, end } => {
                            // category_usage 暂时仍使用旧实现（后续迁移到 CategoryManagementUseCase）
                            let category_usage_result = tail_core::traits::CategoryUsageQuery::get_category_usage(
                                &repo.usage_service(),
                                start,
                                end,
                            )
                            .await
                            .map_err(|e| e.to_string());

                            if let Ok(category_usage) = category_usage_result {
                                let categories_result = tail_core::traits::CategoryRepository::get_all(
                                    &repo.category_service(),
                                )
                                .await
                                .map_err(|e| e.to_string());

                                let all_apps_result = tail_core::traits::CategoryRepository::get_all_app_names(
                                    &repo.category_service(),
                                )
                                .await
                                .map_err(|e| e.to_string());

                                // 使用新的用例层获取应用使用数据
                                let app_usage_result: Result<Vec<AppUsage>, String> = stats_use_case.get_stats(&tail_core::domain::NavigationPath::new()).await
                                    .map(|view| stats_view_to_app_usage(view))
                                    .map_err(|e: AppError| e.to_string());

                                match (categories_result, all_apps_result, app_usage_result) {
                                    (Ok(categories), Ok(all_apps), Ok(app_usage)) => {
                                        let payload = CategoriesDataPayload {
                                            category_usage,
                                            categories,
                                            all_apps,
                                            app_usage,
                                        };
                                        let _ = response_tx.send(GuiResponse::CategoriesData(Ok(payload)));
                                    }
                                    (Err(e), _, _) | (_, Err(e), _) | (_, _, Err(e)) => {
                                        let _ = response_tx.send(GuiResponse::CategoriesData(Err(e)));
                                    }
                                }
                            } else {
                                let _ = response_tx.send(GuiResponse::CategoriesData(
                                    category_usage_result.map(|_| CategoriesDataPayload {
                                        category_usage: vec![],
                                        categories: vec![],
                                        all_apps: vec![],
                                        app_usage: vec![],
                                    })
                                ));
                            }
                        }

                        GuiCommand::AddDailyGoal(goal) => {
                            let result = tail_core::traits::DailyGoalRepository::upsert(
                                &repo.goal_service(),
                                &goal,
                            )
                            .await
                            .map(|_| ())
                            .map_err(|e| e.to_string());

                            let _ = response_tx.send(GuiResponse::Done(result));
                        }

                        GuiCommand::DeleteDailyGoal(app_name) => {
                            let result = tail_core::traits::DailyGoalRepository::delete(
                                &repo.goal_service(),
                                &app_name,
                            )
                            .await
                            .map(|_| ())
                            .map_err(|e| e.to_string());

                            let _ = response_tx.send(GuiResponse::Done(result));
                        }

                        GuiCommand::SetAppAlias { app_name, alias } => {
                            let result = tail_core::traits::AliasRepository::set(
                                &repo.aliases(),
                                &app_name,
                                &alias,
                            )
                            .await
                            .map_err(|e| e.to_string());

                            let _ = response_tx.send(GuiResponse::Done(result));
                        }

                        GuiCommand::DeleteAppAlias(app_name) => {
                            let result = tail_core::traits::AliasRepository::delete(
                                &repo.aliases(),
                                &app_name,
                            )
                            .await
                            .map_err(|e| e.to_string());

                            let _ = response_tx.send(GuiResponse::Done(result));
                        }

                        GuiCommand::GetAllAliases => {
                            let result = tail_core::traits::AliasRepository::get_all(&repo.aliases())
                                .await
                                .map_err(|e| e.to_string());

                            let _ = response_tx.send(GuiResponse::Aliases(result));
                        }

                        GuiCommand::AddCategory(category) => {
                            let result = tail_core::traits::CategoryRepository::insert(
                                &repo.category_service(),
                                &category,
                            )
                            .await
                            .map(|_| ())
                            .map_err(|e| e.to_string());

                            let _ = response_tx.send(GuiResponse::Done(result));
                        }

                        GuiCommand::UpdateCategory(category) => {
                            let result = tail_core::traits::CategoryRepository::update(
                                &repo.category_service(),
                                &category,
                            )
                            .await
                            .map_err(|e| e.to_string());

                            let _ = response_tx.send(GuiResponse::Done(result));
                        }

                        GuiCommand::DeleteCategory(id) => {
                            let result = tail_core::traits::CategoryRepository::delete(
                                &repo.category_service(),
                                id,
                            )
                            .await
                            .map_err(|e| e.to_string());

                            let _ = response_tx.send(GuiResponse::Done(result));
                        }

                        GuiCommand::SetAppCategories { app_name, category_ids } => {
                            let result = tail_core::traits::CategoryRepository::set_app_categories(
                                &repo.category_service(),
                                &app_name,
                                &category_ids,
                            )
                            .await
                            .map_err(|e| e.to_string());

                            let _ = response_tx.send(GuiResponse::Done(result));
                        }

                        GuiCommand::RemoveAppFromCategory { app_name, category_id } => {
                            let result = tail_core::traits::CategoryRepository::remove_app_from_category(
                                &repo.category_service(),
                                &app_name,
                                category_id,
                            )
                            .await
                            .map_err(|e| e.to_string());

                            let _ = response_tx.send(GuiResponse::Done(result));
                        }

                        GuiCommand::GetAppCategories(app_name) => {
                            let result = tail_core::traits::CategoryRepository::get_app_categories(
                                &repo.category_service(),
                                &app_name,
                            )
                            .await
                            .map_err(|e| e.to_string());

                            let _ = response_tx.send(GuiResponse::AppCategories(result));
                        }

                        GuiCommand::GetAllCategories => {
                            let result = tail_core::traits::CategoryRepository::get_all(
                                &repo.category_service(),
                            )
                            .await
                            .map_err(|e| e.to_string());

                            let _ = response_tx.send(GuiResponse::Categories(result));
                        }

                        GuiCommand::GetAllAppNames => {
                            let result = tail_core::traits::CategoryRepository::get_all_app_names(
                                &repo.category_service(),
                            )
                            .await
                            .map_err(|e| e.to_string());

                            let _ = response_tx.send(GuiResponse::AppNames(result));
                        }

                        GuiCommand::Shutdown => {
                            tracing::info!("异步桥接器收到关闭命令");
                            let _ = response_tx.send(GuiResponse::ShutdownAck);
                            break;
                        }
                    }
                }

                tracing::info!("异步桥接器后台线程已退出");
            });
        });

        Self {
            command_tx,
            response_rx,
        }
    }

    /// 发送命令（非阻塞）
    pub fn send_command(&self, cmd: GuiCommand) -> Result<(), String> {
        self.command_tx
            .send(cmd)
            .map_err(|e| format!("Failed to send command: {}", e))
    }

    /// 尝试接收响应（非阻塞）
    pub fn try_recv_response(&mut self) -> Option<GuiResponse> {
        self.response_rx.try_recv().ok()
    }

    /// 检查是否有待处理的响应
    pub fn has_response(&self) -> bool {
        !self.response_rx.is_empty()
    }
}

impl Drop for AsyncBridge {
    fn drop(&mut self) {
        // 发送关闭命令
        let _ = self.send_command(GuiCommand::Shutdown);
    }
}
