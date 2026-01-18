//! 领域层 - 纯业务逻辑，无外部依赖
//!
//! 此模块包含：
//! - 值对象（Value Objects）
//! - 领域服务（Domain Services）
//! - 领域错误（Domain Errors）
//!
//! 设计原则：
//! - 无外部依赖（如数据库、GUI）
//! - 纯函数式实现
//! - 易于测试

pub mod time_event;
pub mod aggregation;
pub mod navigation;
pub mod errors;

// 重新导出常用类型
pub use time_event::{TimeEvent, TimeRange, AppName};
pub use aggregation::{TimeSeriesAnalyzer, TimeGranularity, AggregationResult, Bucket, TrendAnalysis, TrendDirection};
pub use navigation::{NavigationPath, TimeSelector, NavigationLevel};
pub use errors::{DomainError, ValidationError, NavigationError};
