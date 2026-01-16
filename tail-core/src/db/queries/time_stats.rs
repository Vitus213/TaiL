//! 时间统计查询实现

use crate::errors::{DbError, DbResult};
use crate::models::PeriodUsage;
use crate::traits::TimeStatsQuery;
use crate::db::pool::DbPool;
use async_trait::async_trait;
use chrono::{Datelike, Local, NaiveDate, Utc};
use rusqlite::params;
use std::sync::Arc;

/// 时间统计查询实现
pub struct TimeStatsQueryImpl {
    pool: Arc<DbPool>,
}

impl TimeStatsQueryImpl {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }

    fn get_yearly_usage_sync(&self, years: i32) -> DbResult<Vec<PeriodUsage>> {
        let conn = self.pool.get()?;
        let current_year = Local::now().year();
        let mut result = Vec::new();

        for i in 0..years {
            let year = current_year - i;
            let year_start = NaiveDate::from_ymd_opt(year, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_local_timezone(Local)
                .unwrap()
                .with_timezone(&Utc);
            let year_end = NaiveDate::from_ymd_opt(year, 12, 31)
                .unwrap()
                .and_hms_opt(23, 59, 59)
                .unwrap()
                .and_local_timezone(Local)
                .unwrap()
                .with_timezone(&Utc);

            let mut stmt = conn.prepare(
                "SELECT COALESCE(SUM(duration_secs), 0)
                 FROM window_events
                 WHERE timestamp >= ?1 AND timestamp <= ?2 AND is_afk = 0",
            )?;

            let total: i64 = stmt.query_row(params![year_start, year_end], |row| row.get(0))?;

            result.push(PeriodUsage {
                label: format!("{}年", year),
                index: year,
                total_seconds: total,
            });
        }

        result.reverse();
        Ok(result)
    }

    fn get_monthly_usage_sync(&self, year: i32) -> DbResult<Vec<PeriodUsage>> {
        let conn = self.pool.get()?;
        let mut result = Vec::new();

        for month in 1..=12 {
            let month_start = NaiveDate::from_ymd_opt(year, month, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_local_timezone(Local)
                .unwrap()
                .with_timezone(&Utc);

            let next_month = if month == 12 {
                NaiveDate::from_ymd_opt(year + 1, 1, 1)
            } else {
                NaiveDate::from_ymd_opt(year, month + 1, 1)
            }
            .unwrap();
            let last_day = next_month.pred_opt().unwrap();

            let month_end = last_day
                .and_hms_opt(23, 59, 59)
                .unwrap()
                .and_local_timezone(Local)
                .unwrap()
                .with_timezone(&Utc);

            let mut stmt = conn.prepare(
                "SELECT COALESCE(SUM(duration_secs), 0)
                 FROM window_events
                 WHERE timestamp >= ?1 AND timestamp <= ?2 AND is_afk = 0",
            )?;

            let total: i64 = stmt.query_row(params![month_start, month_end], |row| row.get(0))?;

            result.push(PeriodUsage {
                label: format!("{}月", month),
                index: month as i32,
                total_seconds: total,
            });
        }

        Ok(result)
    }

    fn get_weekly_usage_sync(&self, year: i32, month: u32) -> DbResult<Vec<PeriodUsage>> {
        let conn = self.pool.get()?;
        let mut result = Vec::new();

        let first_day = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        let next_month = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1)
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1)
        }
        .unwrap();
        let last_day = next_month.pred_opt().unwrap();

        let mut week_num = 1;
        let mut current_day = first_day;

        while current_day <= last_day {
            let week_start = current_day;
            let mut week_end = current_day;

            while week_end.weekday() != chrono::Weekday::Sun && week_end < last_day {
                week_end = week_end.succ_opt().unwrap();
            }

            let start_dt = week_start
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_local_timezone(Local)
                .unwrap()
                .with_timezone(&Utc);
            let end_dt = week_end
                .and_hms_opt(23, 59, 59)
                .unwrap()
                .and_local_timezone(Local)
                .unwrap()
                .with_timezone(&Utc);

            let mut stmt = conn.prepare(
                "SELECT COALESCE(SUM(duration_secs), 0)
                 FROM window_events
                 WHERE timestamp >= ?1 AND timestamp <= ?2 AND is_afk = 0",
            )?;

            let total: i64 = stmt.query_row(params![start_dt, end_dt], |row| row.get(0))?;

            result.push(PeriodUsage {
                label: format!("第{}周", week_num),
                index: week_num,
                total_seconds: total,
            });

            current_day = week_end.succ_opt().unwrap();
            week_num += 1;
        }

        Ok(result)
    }

    fn get_daily_usage_for_week_sync(
        &self,
        year: i32,
        month: u32,
        week: u32,
    ) -> DbResult<Vec<PeriodUsage>> {
        let conn = self.pool.get()?;
        let mut result = Vec::new();

        let first_day = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        let next_month = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1)
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1)
        }
        .unwrap();
        let last_day = next_month.pred_opt().unwrap();

        // 找到指定周的开始日期
        let mut current_day = first_day;
        let mut current_week = 1u32;

        while current_week < week && current_day <= last_day {
            while current_day.weekday() != chrono::Weekday::Sun && current_day < last_day {
                current_day = current_day.succ_opt().unwrap();
            }
            current_day = current_day.succ_opt().unwrap_or(current_day);
            current_week += 1;
        }

        let weekday_names = ["周一", "周二", "周三", "周四", "周五", "周六", "周日"];

        loop {
            if current_day > last_day {
                break;
            }

            let start_dt = current_day
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_local_timezone(Local)
                .unwrap()
                .with_timezone(&Utc);
            let end_dt = current_day
                .and_hms_opt(23, 59, 59)
                .unwrap()
                .and_local_timezone(Local)
                .unwrap()
                .with_timezone(&Utc);

            let mut stmt = conn.prepare(
                "SELECT COALESCE(SUM(duration_secs), 0)
                 FROM window_events
                 WHERE timestamp >= ?1 AND timestamp <= ?2 AND is_afk = 0",
            )?;

            let total: i64 = stmt.query_row(params![start_dt, end_dt], |row| row.get(0))?;

            let weekday_idx = current_day.weekday().num_days_from_monday() as usize;
            result.push(PeriodUsage {
                label: weekday_names[weekday_idx].to_string(),
                index: current_day.day() as i32,
                total_seconds: total,
            });

            if current_day.weekday() == chrono::Weekday::Sun || current_day >= last_day {
                break;
            }

            current_day = current_day.succ_opt().unwrap();
        }

        Ok(result)
    }

    fn get_hourly_usage_sync(&self, year: i32, month: u32, day: u32) -> DbResult<Vec<PeriodUsage>> {
        let conn = self.pool.get()?;
        let mut result = Vec::new();

        let date = NaiveDate::from_ymd_opt(year, month, day).unwrap();

        for hour in 0..24 {
            let hour_start = date
                .and_hms_opt(hour, 0, 0)
                .unwrap()
                .and_local_timezone(Local)
                .unwrap()
                .with_timezone(&Utc);
            let hour_end = date
                .and_hms_opt(hour, 59, 59)
                .unwrap()
                .and_local_timezone(Local)
                .unwrap()
                .with_timezone(&Utc);

            let mut stmt = conn.prepare(
                "SELECT COALESCE(SUM(duration_secs), 0)
                 FROM window_events
                 WHERE timestamp >= ?1 AND timestamp <= ?2 AND is_afk = 0",
            )?;

            let total: i64 = stmt.query_row(params![hour_start, hour_end], |row| row.get(0))?;

            result.push(PeriodUsage {
                label: format!("{}时", hour),
                index: hour as i32,
                total_seconds: total,
            });
        }

        Ok(result)
    }
}

#[async_trait]
impl TimeStatsQuery for TimeStatsQueryImpl {
    async fn get_yearly_usage(&self, years: i32) -> DbResult<Vec<PeriodUsage>> {
        let query = self.clone();
        tokio::task::spawn_blocking(move || query.get_yearly_usage_sync(years))
            .await
            .map_err(|e| DbError::Validation(format!("Task join error: {}", e)))?
    }

    async fn get_monthly_usage(&self, year: i32) -> DbResult<Vec<PeriodUsage>> {
        let query = self.clone();
        tokio::task::spawn_blocking(move || query.get_monthly_usage_sync(year))
            .await
            .map_err(|e| DbError::Validation(format!("Task join error: {}", e)))?
    }

    async fn get_weekly_usage(&self, year: i32, month: u32) -> DbResult<Vec<PeriodUsage>> {
        let query = self.clone();
        tokio::task::spawn_blocking(move || query.get_weekly_usage_sync(year, month))
            .await
            .map_err(|e| DbError::Validation(format!("Task join error: {}", e)))?
    }

    async fn get_daily_usage_for_week(
        &self,
        year: i32,
        month: u32,
        week: u32,
    ) -> DbResult<Vec<PeriodUsage>> {
        let query = self.clone();
        tokio::task::spawn_blocking(move || query.get_daily_usage_for_week_sync(year, month, week))
            .await
            .map_err(|e| DbError::Validation(format!("Task join error: {}", e)))?
    }

    async fn get_hourly_usage(
        &self,
        year: i32,
        month: u32,
        day: u32,
    ) -> DbResult<Vec<PeriodUsage>> {
        let query = self.clone();
        tokio::task::spawn_blocking(move || query.get_hourly_usage_sync(year, month, day))
            .await
            .map_err(|e| DbError::Validation(format!("Task join error: {}", e)))?
    }
}

impl Clone for TimeStatsQueryImpl {
    fn clone(&self) -> Self {
        Self {
            pool: Arc::clone(&self.pool),
        }
    }
}
