//! TaiL 日志配置模块
//!
//! 提供统一的日志初始化函数和辅助宏

use tracing_subscriber::{fmt, EnvFilter};

/// 日志输出模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogOutput {
    /// 标准输出（GUI 默认）
    Stdout,
    /// systemd journal（服务端默认）
    SystemdJournal,
}

/// 初始化日志系统
///
/// # 参数
/// - `output`: 日志输出模式
/// - `default_level`: 默认日志级别（当 RUST_LOG 未设置时使用）
///
/// # 示例
/// ```no_run
/// use tail_core::logging::{init_logging, LogOutput};
///
/// // GUI 应用：输出到标准输出
/// init_logging(LogOutput::Stdout, "info");
///
/// // 服务应用：输出到 systemd journal
/// init_logging(LogOutput::SystemdJournal, "info");
/// ```
pub fn init_logging(output: LogOutput, default_level: &str) {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        tracing::warn!("RUST_LOG 解析失败，使用默认级别: {}", default_level);
        EnvFilter::new(default_level)
    });

    let builder = fmt().with_env_filter(env_filter);

    match output {
        LogOutput::Stdout => {
            builder.init();
            tracing::info!(
                "日志系统已初始化（输出：标准输出，默认级别：{}）",
                default_level
            );
        }
        LogOutput::SystemdJournal => {
            builder.with_ansi(false).init();
            tracing::info!(
                "日志系统已初始化（输出：systemd journal，默认级别：{}）",
                default_level
            );
        }
    }
}

/// unwrap 失败时记录错误日志
///
/// 用于替换 unwrap() 调用，在 panic 前记录详细的错误信息
///
/// # 示例
/// ```no_run
/// use tail_core::unwrap_or_log;
///
/// let result: Result<i32, &str> = Err("error");
/// let value = unwrap_or_log!(result, "Failed to get value");
/// ```
#[macro_export]
macro_rules! unwrap_or_log {
    ($expr:expr, $msg:expr) => {
        match $expr {
            Ok(v) => v,
            Err(e) => {
                tracing::error!(
                    error = %e,
                    context = $msg,
                    file = file!(),
                    line = line!(),
                    "unwrap failed"
                );
                panic!("{}: {}", $msg, e);
            }
        }
    };
}

/// unwrap Option 时记录错误日志
///
/// 用于替换 expect() 调用，在 panic 前记录详细的错误信息
///
/// # 示例
/// ```no_run
/// use tail_core::unwrap_some_or_log;
///
/// let value: Option<i32> = None;
/// let v = unwrap_some_or_log!(value, "Value was None");
/// ```
#[macro_export]
macro_rules! unwrap_some_or_log {
    ($expr:expr, $msg:expr) => {
        match $expr {
            Some(v) => v,
            None => {
                tracing::error!(
                    context = $msg,
                    file = file!(),
                    line = line!(),
                    "unwrap_some failed: value was None"
                );
                panic!("{}: value was None", $msg);
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_output_display() {
        let stdout = LogOutput::Stdout;
        let journal = LogOutput::SystemdJournal;

        // 测试 PartialEq
        assert_eq!(stdout, LogOutput::Stdout);
        assert_eq!(journal, LogOutput::SystemdJournal);
        assert_ne!(stdout, journal);
    }
}
