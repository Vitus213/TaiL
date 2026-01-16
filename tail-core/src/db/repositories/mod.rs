//! 数据库仓储模块

pub mod alias;
pub mod afk_event;
pub mod category;
pub mod daily_goal;
pub mod window_event;

pub use alias::AliasRepositoryImpl;
pub use afk_event::AfkEventRepositoryImpl;
pub use category::CategoryRepositoryImpl;
pub use daily_goal::DailyGoalRepositoryImpl;
pub use window_event::WindowEventRepositoryImpl;
