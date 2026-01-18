//! 应用层 - 用例编排
//!
//! 此模块包含：
//! - 端口定义（Ports）- 输入端口定义用例接口
//! - 用例实现（Use Cases）- 编排领域对象完成业务逻辑
//! - 适配器（Adapters）- 连接新旧代码的适配器

pub mod adapter;
pub mod converters;
pub mod ports;
pub mod use_cases;

// 重新导出常用类型
pub use adapter::RepositoryAdapter;
pub use converters::*;
pub use ports::*;
pub use use_cases::{CategoryManagementUseCase, StatsQueryUseCase};
