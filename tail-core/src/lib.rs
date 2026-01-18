//! TaiL Core - 数据模型和数据库访问层
//!
//! # 模块结构
//!
//! - [`domain`]: 纯领域层，包含值对象和领域服务
//! - [`application`]: 应用层，包含用例和端口定义
//! - [`db`]: 数据库访问层
//! - [`services`]: 应用服务层
//! - [`models`]: 数据模型（兼容旧版）
//! - [`traits`]: 仓储抽象接口

pub mod domain;
pub mod application;
pub mod db;
pub mod errors;
pub mod logging;
pub mod models;
pub mod services;
pub mod time;
pub mod traits;
pub mod utils;

// 重新导出领域层
pub use domain::{
    aggregation::{AggregationResult, Bucket, TimeGranularity, TimeSeriesAnalyzer, TrendAnalysis, TrendDirection},
    navigation::{NavigationLevel, NavigationPath, TimeSelector},
    time_event::{AppName, TimeEvent, TimeRange},
    DomainError, NavigationError, ValidationError,
};

// 重新导出应用层
pub use application::{
    adapter::RepositoryAdapter,
    converters::{dashboard_view_to_app_usage, stats_view_to_app_usage},
    ports::{
        AppError, AppUsageItem, DashboardView, EventRepositoryPort, RepoError,
        StatsQueryPort, StatsView, TrendDirection as AppTrendDirection, TrendView,
        format_duration, parse_duration,
    },
    use_cases::{CategoryManagementUseCase, StatsQueryUseCase},
    CategoryManagementPort,
};

// 应用层的 Category 类型与 models::Category 冲突，使用别名导出
pub use application::ports::Category as CategoryDto;

// 重新导出数据库层
pub use db::*;
pub use errors::{DbError, DbResult};
pub use logging::*;
pub use models::*;
pub use traits::*;
pub use utils::{duration, filter, time_range};

// 重新导出服务层的数据类型
pub use services::{
    category_service::CategoryManagementData,
    goal_service::GoalProgress,
    usage_service::{DashboardData, StatsData},
};
