//! TaiL AFK 检测器

use std::time::{Duration, Instant};
use thiserror::Error;

/// AFK 检测错误
#[derive(Debug, Error)]
pub enum AfkError {
    #[error("Failed to detect activity: {0}")]
    DetectionError(String),
}

/// AFK 状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AfkState {
    /// 活跃状态
    Active,
    /// 空闲状态 (包含开始时间)
    Afk { since: Instant },
}

/// AFK 检测器
pub struct AfkDetector {
    timeout: Duration,
    last_activity: Instant,
    state: AfkState,
}

impl AfkDetector {
    /// 创建新的 AFK 检测器
    pub fn new(timeout_secs: u64) -> Self {
        Self {
            timeout: Duration::from_secs(timeout_secs),
            last_activity: Instant::now(),
            state: AfkState::Active,
        }
    }

    /// 更新活动时间
    pub fn record_activity(&mut self) {
        self.last_activity = Instant::now();
        if self.state != AfkState::Active {
            self.state = AfkState::Active;
        }
    }

    /// 检查当前 AFK 状态
    pub fn check_state(&mut self) -> AfkState {
        let elapsed = self.last_activity.elapsed();

        if elapsed >= self.timeout && self.state == AfkState::Active {
            self.state = AfkState::Afk {
                since: self.last_activity,
            };
        } else if elapsed < self.timeout {
            self.state = AfkState::Active;
        }

        self.state
    }

    /// 获取当前状态 (不更新)
    pub fn current_state(&self) -> AfkState {
        self.state
    }

    /// 检查是否处于 AFK 状态
    pub fn is_afk(&self) -> bool {
        !matches!(self.state, AfkState::Active)
    }
}

impl Default for AfkDetector {
    fn default() -> Self {
        Self::new(300) // 默认 5 分钟
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_new_detector_is_active() {
        let detector = AfkDetector::new(5);
        assert_eq!(detector.current_state(), AfkState::Active);
        assert!(!detector.is_afk());
    }

    #[test]
    fn test_record_activity() {
        let mut detector = AfkDetector::new(5);
        thread::sleep(Duration::from_secs(1));

        detector.record_activity();
        assert_eq!(detector.current_state(), AfkState::Active);
    }

    #[test]
    fn test_afk_timeout() {
        let mut detector = AfkDetector::new(1); // 1秒超时

        // 等待超时
        thread::sleep(Duration::from_secs(2));

        let state = detector.check_state();
        assert!(matches!(state, AfkState::Afk { .. }));
        assert!(detector.is_afk());
    }

    #[test]
    fn test_return_from_afk() {
        let mut detector = AfkDetector::new(1);

        // 进入 AFK 状态
        thread::sleep(Duration::from_secs(2));
        detector.check_state();
        assert!(detector.is_afk());

        // 记录活动，应该回到活跃状态
        detector.record_activity();
        assert_eq!(detector.current_state(), AfkState::Active);
        assert!(!detector.is_afk());
    }

    #[test]
    fn test_check_state_updates_status() {
        let mut detector = AfkDetector::new(1);

        // 初始是活跃的
        assert!(!detector.is_afk());

        // 等待超时
        thread::sleep(Duration::from_secs(2));
        detector.check_state();

        // 现在应该是 AFK
        assert!(detector.is_afk());
    }

    #[test]
    fn test_default_timeout() {
        let detector = AfkDetector::default();
        // 默认超时应该是 300 秒
        assert_eq!(detector.timeout, Duration::from_secs(300));
    }
}
