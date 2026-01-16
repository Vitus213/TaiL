# TaiL 测试指南

本文档介绍 TaiL 的测试策略和如何运行/编写测试。

## 目录

- [测试概述](#测试概述)
- [运行测试](#运行测试)
- [编写测试](#编写测试)
- [测试覆盖率](#测试覆盖率)

---

## 测试概述

### 测试金字塔

```
    ┌─────────────────┐
    │   E2E 测试      │  (少量，手动验证)
    └─────────────────┘
          ▲
    ┌─────────────────┐
    │   集成测试      │  (中等，tests/ 目录)
    └─────────────────┘
          ▲
    ┌─────────────────┐
    │   单元测试      │  (大量，#[test])
    └─────────────────┘
```

### 测试目录结构

```
tail/
├── tail-core/
│   └── src/           # 单元测试（在模块内）
├── tail-hyprland/
│   └── src/           # 单元测试
├── tail-afk/
│   └── src/           # 单元测试
├── tail-gui/
│   └── src/           # 单元测试
├── tail-service/
│   └── src/           # 单元测试
└── tests/             # 集成测试
    └── integration/
```

---

## 运行测试

### 运行所有测试

```bash
# 使用 Cargo
cargo test --workspace

# 使用 just
just test
```

### 运行单元测试

```bash
# 只运行库测试
cargo test --lib --workspace

# 运行特定模块
cargo test -p tail-core
cargo test -p tail-hyprland
```

### 运行集成测试

```bash
# 运行所有集成测试
cargo test --test '*'

# 运行特定集成测试
cargo test --test test_repository
```

### 显示测试输出

```bash
# 显示 print! 输出
cargo test -- --nocapture

# 显示详细输出
cargo test -- --show-output
```

### 运行特定测试

```bash
# 运行特定名称的测试
cargo test test_window_event

# 运行特定模块的测试
cargo test tail_core::models::tests
```

### 并行运行

```bash
# 禁用并行（测试有冲突时）
cargo test -- --test-threads=1
```

---

## 编写测试

### 单元测试

在模块内编写单元测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_event_creation() {
        let event = WindowEvent {
            id: None,
            app_name: "test".to_string(),
            window_title: "Test Window".to_string(),
            start_time: Utc::now(),
            duration_secs: 60,
            is_afk: false,
        };

        assert_eq!(event.app_name, "test");
        assert_eq!(event.duration_secs, 60);
    }

    #[test]
    fn test_duration_calculation() {
        let start = Utc::now();
        let end = start + chrono::Duration::seconds(60);
        let duration = (end - start).num_seconds();

        assert_eq!(duration, 60);
    }
}
```

### 集成测试

在 `tests/` 目录创建集成测试：

```rust
// tests/integration/test_repository.rs

use tail_core::repositories::Repository;
use tail_core::models::WindowEvent;

#[tokio::test]
async fn test_insert_and_query() {
    // 创建临时数据库
    let repo = Repository::new_in_memory().await;

    // 插入测试数据
    let event = WindowEvent {
        id: None,
        app_name: "test".to_string(),
        window_title: "Test".to_string(),
        start_time: Utc::now(),
        duration_secs: 60,
        is_afk: false,
    };

    let id = repo.insert_window_event(&event).await.unwrap();
    assert!(id > 0);

    // 查询数据
    let events = repo.get_window_events(None, None).await.unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].app_name, "test");
}
```

### 异步测试

使用 `tokio::test` 宏：

```rust
#[tokio::test]
async fn test_async_function() {
    let result = async_function().await.unwrap();
    assert_eq!(result, expected);
}
```

### 参数化测试

使用测试参数：

```rust
#[test]
fn test_multiple_cases() {
    let cases = vec![
        ("app1", 100),
        ("app2", 200),
        ("app3", 300),
    ];

    for (app_name, expected_duration) in cases {
        let event = create_event(app_name, expected_duration);
        assert_eq!(event.duration_secs, expected_duration);
    }
}
```

---

## 测试覆盖率

### 安装 tarpaulin

```bash
cargo install cargo-tarpaulin
```

### 生成覆盖率报告

```bash
# HTML 格式
cargo tarpaulin --workspace --out Html

# 终端输出
cargo tarpaulin --workspace --out Stdout

# 覆盖率目标
cargo tarpaulin --workspace --out Html --timeout 120 --skip-clean
```

### 覆盖率目标

| 模块 | 目标覆盖率 |
|------|-----------|
| tail-core | 90%+ |
| tail-hyprland | 80%+ |
| tail-service | 75%+ |
| **总体** | **80%+** |

---

## 测试最佳实践

### 1. 测试命名

使用描述性的测试名称：

```rust
// 好的命名
fn test_insert_window_event_with_valid_data_succeeds() {}

// 避免过于简单
fn test_insert() {}
```

### 2. 测试组织

使用模块组织相关测试：

```rust
#[cfg(test)]
mod tests {
    mod creation {
        // 创建相关测试
    }

    mod validation {
        // 验证相关测试
    }

    mod persistence {
        // 持久化相关测试
    }
}
```

### 3. 使用测试辅助函数

```rust
fn create_test_event(app_name: &str) -> WindowEvent {
    WindowEvent {
        id: None,
        app_name: app_name.to_string(),
        window_title: "Test".to_string(),
        start_time: Utc::now(),
        duration_secs: 60,
        is_afk: false,
    }
}

#[test]
fn test_case_1() {
    let event = create_test_event("app1");
    // 测试逻辑
}
```

### 4. 测试数据隔离

每个测试应该独立运行，不依赖其他测试：

```rust
#[tokio::test]
async fn test_independent() {
    // 创建独立的测试数据
    let repo = Repository::new_in_memory().await;
    // 测试逻辑
}
```

---

## CI/CD 测试

项目使用 GitHub Actions 自动运行测试：

```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: cachix/install-nix-action@v20
        with:
          nix_path: nixpkgs=channel:nixos-unstable
      - run: nix flake check
      - run: nix develop --command cargo test --workspace
```

每次提交都会：
1. 运行所有单元测试
2. 运行所有集成测试
3. 检查代码格式
4. 运行 Clippy 检查
