//! TaiL Core - 数据模型和数据库访问层

pub mod db;
pub mod errors;
pub mod logging;
pub mod models;
pub mod services;
pub mod time;
pub mod traits;
pub mod utils;

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
