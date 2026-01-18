//! 领域错误类型
//!
//! 使用 thiserror 定义精确的错误类型，便于错误处理和调试。

use chrono::{DateTime, Utc};
use std::fmt;

/// 领域层错误
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    /// 验证错误
    Validation(ValidationError),

    /// 时间范围无效
    InvalidTimeRange {
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    },

    /// 持续时间无效
    InvalidDuration(i64),

    /// 导航错误
    Navigation(NavigationError),

    /// 实体未找到
    NotFound(String),

    /// 业务规则违反
    BusinessRuleViolation(String),
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DomainError::Validation(e) => write!(f, "验证错误: {}", e),
            DomainError::InvalidTimeRange { start, end } => {
                write!(f, "无效的时间范围: {} 必须早于 {}", start, end)
            }
            DomainError::InvalidDuration(secs) => {
                write!(f, "无效的持续时间: {} 秒", secs)
            }
            DomainError::Navigation(e) => write!(f, "导航错误: {}", e),
            DomainError::NotFound(entity) => write!(f, "未找到: {}", entity),
            DomainError::BusinessRuleViolation(msg) => {
                write!(f, "业务规则违反: {}", msg)
            }
        }
    }
}

impl std::error::Error for DomainError {}

/// 验证错误
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// 应用名称为空
    EmptyAppName,

    /// 应用名称无效
    InvalidAppName(String),

    /// 分类名称为空
    EmptyCategoryName,

    /// 时间戳无效
    InvalidTimestamp(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::EmptyAppName => write!(f, "应用名称不能为空"),
            ValidationError::InvalidAppName(name) => {
                write!(f, "应用名称无效: '{}'", name)
            }
            ValidationError::EmptyCategoryName => {
                write!(f, "分类名称不能为空")
            }
            ValidationError::InvalidTimestamp(ts) => {
                write!(f, "时间戳无效: {}", ts)
            }
        }
    }
}

/// 导航错误
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationError {
    /// 无效的状态转换
    InvalidTransition,

    /// 已经在顶层
    AlreadyAtTop,

    /// 已经在底层
    AlreadyAtBottom,

    /// 无效的选择器
    InvalidSelector,
}

impl fmt::Display for NavigationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NavigationError::InvalidTransition => write!(f, "无效的导航转换"),
            NavigationError::AlreadyAtTop => write!(f, "已经在最顶层"),
            NavigationError::AlreadyAtBottom => write!(f, "已经在最底层"),
            NavigationError::InvalidSelector => write!(f, "无效的选择器"),
        }
    }
}

impl From<ValidationError> for DomainError {
    fn from(e: ValidationError) -> Self {
        DomainError::Validation(e)
    }
}

impl From<NavigationError> for DomainError {
    fn from(e: NavigationError) -> Self {
        DomainError::Navigation(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = DomainError::Validation(ValidationError::EmptyAppName);
        assert_eq!(format!("{}", err), "验证错误: 应用名称不能为空");

        let err = DomainError::InvalidDuration(-100);
        assert_eq!(format!("{}", err), "无效的持续时间: -100 秒");
    }
}
