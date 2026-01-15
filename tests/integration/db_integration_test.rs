//! 数据库集成测试

use chrono::{Duration, Utc};
use tail_core::{AfkEvent, DailyGoal, DbConfig, Repository, WindowEvent};
use tempfile::TempDir;

/// 测试上下文
struct TestContext {
    _temp_dir: TempDir,
    repo: Repository,
}

impl TestContext {
    fn new() -> Self {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let config = DbConfig {
            path: db_path.to_str().unwrap().to_string(),
        };
        let repo = Repository::new(&config).unwrap();

        Self {
            _temp_dir: temp_dir,
            repo,
        }
    }
}

fn create_window_event(app: &str, duration: i64) -> WindowEvent {
    WindowEvent {
        id: None,
        timestamp: Utc::now(),
        app_name: app.to_string(),
        window_title: format!("{} Window", app),
        workspace: "1".to_string(),
        duration_secs: duration,
        is_afk: false,
    }
}

fn create_afk_event(duration: i64) -> AfkEvent {
    AfkEvent {
        id: None,
        start_time: Utc::now() - Duration::seconds(duration),
        end_time: Some(Utc::now()),
        duration_secs: duration,
    }
}

fn create_daily_goal(app: &str, max_minutes: i32) -> DailyGoal {
    DailyGoal {
        id: None,
        app_name: app.to_string(),
        max_minutes,
        notify_enabled: true,
    }
}

#[test]
fn test_window_event_lifecycle() {
    let ctx = TestContext::new();

    // 插入事件
    let event = create_window_event("firefox", 120);
    let id = ctx.repo.insert_window_event(&event).unwrap();
    assert!(id > 0);

    // 查询验证
    let start = Utc::now() - Duration::hours(1);
    let end = Utc::now() + Duration::hours(1);
    let events = ctx.repo.get_window_events(start, end).unwrap();

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].app_name, "firefox");
    assert_eq!(events[0].duration_secs, 120);
}

#[test]
fn test_app_usage_aggregation() {
    let ctx = TestContext::new();

    // 插入多个事件
    ctx.repo
        .insert_window_event(&create_window_event("firefox", 120))
        .unwrap();
    ctx.repo
        .insert_window_event(&create_window_event("firefox", 180))
        .unwrap();
    ctx.repo
        .insert_window_event(&create_window_event("kitty", 60))
        .unwrap();

    // 查询应用使用统计
    let start = Utc::now() - Duration::hours(1);
    let end = Utc::now() + Duration::hours(1);
    let usage = ctx.repo.get_app_usage(start, end).unwrap();

    assert_eq!(usage.len(), 2);
    assert_eq!(usage[0].app_name, "firefox");
    assert_eq!(usage[0].total_seconds, 300); // 120 + 180
    assert_eq!(usage[1].app_name, "kitty");
    assert_eq!(usage[1].total_seconds, 60);
}

#[test]
fn test_afk_event_tracking() {
    let ctx = TestContext::new();

    // 插入 AFK 事件
    let afk_event = create_afk_event(300);
    let id = ctx.repo.insert_afk_event(&afk_event).unwrap();
    assert!(id > 0);

    // 查询验证
    let start = Utc::now() - Duration::hours(1);
    let end = Utc::now() + Duration::hours(1);
    let events = ctx.repo.get_afk_events(start, end).unwrap();

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].duration_secs, 300);
}

#[test]
fn test_daily_goal_management() {
    let ctx = TestContext::new();

    // 创建目标
    let goal = create_daily_goal("firefox", 120);
    ctx.repo.upsert_daily_goal(&goal).unwrap();

    // 查询验证
    let goals = ctx.repo.get_daily_goals().unwrap();
    assert_eq!(goals.len(), 1);
    assert_eq!(goals[0].app_name, "firefox");
    assert_eq!(goals[0].max_minutes, 120);

    // 更新目标
    let updated_goal = create_daily_goal("firefox", 180);
    ctx.repo.upsert_daily_goal(&updated_goal).unwrap();

    let goals = ctx.repo.get_daily_goals().unwrap();
    assert_eq!(goals.len(), 1);
    assert_eq!(goals[0].max_minutes, 180);

    // 删除目标
    ctx.repo.delete_daily_goal("firefox").unwrap();
    let goals = ctx.repo.get_daily_goals().unwrap();
    assert_eq!(goals.len(), 0);
}

#[test]
fn test_concurrent_database_access() {
    use std::sync::Arc;
    use std::thread;

    let ctx = TestContext::new();
    let repo = Arc::new(ctx.repo);

    let mut handles = vec![];

    // 启动多个线程同时写入
    for i in 0..10 {
        let repo_clone = Arc::clone(&repo);
        let handle = thread::spawn(move || {
            let event = create_window_event(&format!("app{}", i), i * 10);
            repo_clone.insert_window_event(&event).unwrap();
        });
        handles.push(handle);
    }

    // 等待所有线程完成
    for handle in handles {
        handle.join().unwrap();
    }

    // 验证所有事件都被插入
    let start = Utc::now() - Duration::hours(1);
    let end = Utc::now() + Duration::hours(1);
    let events = repo.get_window_events(start, end).unwrap();
    assert_eq!(events.len(), 10);
}

#[test]
fn test_time_range_filtering() {
    let ctx = TestContext::new();

    // 插入不同时间的事件
    let now = Utc::now();

    let mut event1 = create_window_event("firefox", 60);
    event1.timestamp = now - Duration::hours(2);
    ctx.repo.insert_window_event(&event1).unwrap();

    let mut event2 = create_window_event("kitty", 120);
    event2.timestamp = now - Duration::minutes(30);
    ctx.repo.insert_window_event(&event2).unwrap();

    let mut event3 = create_window_event("vscode", 180);
    event3.timestamp = now - Duration::hours(5);
    ctx.repo.insert_window_event(&event3).unwrap();

    // 查询最近1小时的事件
    let start = now - Duration::hours(1);
    let end = now;
    let events = ctx.repo.get_window_events(start, end).unwrap();

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].app_name, "kitty");
}
