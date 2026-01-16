//! 目标服务实现

use crate::errors::{DbError, DbResult};
use crate::models::DailyGoal;
use crate::traits::DailyGoalRepository;
use crate::db::repositories::DailyGoalRepositoryImpl;
use crate::db::pool::DbPool;
use async_trait::async_trait;

/// 目标进度
#[derive(Debug, Clone)]
pub struct GoalProgress {
    /// 应用名称
    pub app_name: String,
    /// 目标分钟数
    pub goal_minutes: i32,
    /// 已使用秒数
    pub used_seconds: i64,
    /// 是否达成目标
    pub achieved: bool,
    /// 进度百分比 (0-150)
    pub progress_percent: u32,
}

/// 目标服务实现
pub struct GoalServiceImpl {
    goal_repo: DailyGoalRepositoryImpl,
}

impl GoalServiceImpl {
    pub fn new(pool: DbPool) -> Self {
        Self {
            goal_repo: DailyGoalRepositoryImpl::new(pool),
        }
    }

    /// 检查目标进度
    pub async fn check_goal_progress(&self, app_name: &str) -> DbResult<GoalProgress> {
        let goals = self.goal_repo.get_all().await?;
        let goal = goals
            .iter()
            .find(|g| g.app_name == app_name)
            .ok_or_else(|| DbError::NotFound(format!("Goal not found for app: {}", app_name)))?;

        let used_seconds = self.goal_repo.get_today_usage(app_name).await?;
        let goal_seconds = goal.max_minutes as i64 * 60;

        let progress_percent = if goal_seconds > 0 {
            let percent = (used_seconds as f64 / goal_seconds as f64 * 100.0) as u32;
            percent.min(150)
        } else {
            0
        };

        let achieved = used_seconds >= goal_seconds;

        Ok(GoalProgress {
            app_name: app_name.to_string(),
            goal_minutes: goal.max_minutes,
            used_seconds,
            achieved,
            progress_percent,
        })
    }

    /// 获取所有目标及其进度
    pub async fn get_all_goal_progress(&self) -> DbResult<Vec<GoalProgress>> {
        let goals = self.goal_repo.get_all().await?;
        let mut result = Vec::new();

        for goal in goals {
            let used_seconds = self.goal_repo.get_today_usage(&goal.app_name).await?;
            let goal_seconds = goal.max_minutes as i64 * 60;

            let progress_percent = if goal_seconds > 0 {
                let percent = (used_seconds as f64 / goal_seconds as f64 * 100.0) as u32;
                percent.min(150)
            } else {
                0
            };

            let achieved = used_seconds >= goal_seconds;

            result.push(GoalProgress {
                app_name: goal.app_name,
                goal_minutes: goal.max_minutes,
                used_seconds,
                achieved,
                progress_percent,
            });
        }

        Ok(result)
    }
}

#[async_trait]
impl DailyGoalRepository for GoalServiceImpl {
    async fn upsert(&self, goal: &DailyGoal) -> DbResult<i64> {
        self.goal_repo.upsert(goal).await
    }

    async fn get_all(&self) -> DbResult<Vec<DailyGoal>> {
        self.goal_repo.get_all().await
    }

    async fn delete(&self, app_name: &str) -> DbResult<()> {
        self.goal_repo.delete(app_name).await
    }

    async fn get_today_usage(&self, app_name: &str) -> DbResult<i64> {
        self.goal_repo.get_today_usage(app_name).await
    }
}

impl Clone for GoalServiceImpl {
    fn clone(&self) -> Self {
        Self {
            goal_repo: self.goal_repo.clone(),
        }
    }
}
