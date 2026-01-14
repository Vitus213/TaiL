# TaiL 测试策略与 CI/CD 架构

## 测试策略概览

TaiL 项目采用**多层次测试策略**，确保代码质量和功能正确性。

### 测试金字塔

```
        ┌─────────────────┐
        │   E2E 测试      │  (少量，关键流程)
        │   Docker 验证   │
        └─────────────────┘
              ▲
        ┌─────────────────┐
        │   集成测试      │  (中等数量，模块交互)
        │   tests/        │
        └─────────────────┘
              ▲
        ┌─────────────────┐
        │   单元测试      │  (大量，函数级别)
        │   #[test]       │
        └─────────────────┘
```

---

## 测试目录结构

```
TaiL/
├── tests/                          # 集成测试目录
│   ├── common/                     # 测试工具和辅助函数
│   │   ├── mod.rs
│   │   ├── fixtures.rs             # 测试数据
│   │   └── helpers.rs              # 测试辅助函数
│   ├── integration/                # 集成测试
│   │   ├── db_integration_test.rs
│   │   ├── ipc_integration_test.rs
│   │   └── service_integration_test.rs
│   └── e2e/                        # 端到端测试
│       ├── window_tracking_test.rs
│       └── gui_workflow_test.rs
├── tail-core/
│   └── src/
│       ├── db.rs
│       └── db_test.rs              # 单元测试 (可选独立文件)
├── tail-hyprland/
│   └── src/
│       └── ipc.rs                  # 包含 #[cfg(test)] mod tests
├── tail-service/
│   └── src/
│       └── service.rs              # 包含 #[cfg(test)] mod tests
└── Dockerfile.test                 # 测试环境 Docker 镜像
```

---

## 测试层次详解

### 1. 单元测试 (Unit Tests)

**位置**: 各模块源码文件中的 `#[cfg(test)] mod tests`

**目标**: 测试单个函数或方法的正确性

**示例**:

```rust
// tail-core/src/db.rs
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_insert_window_event() {
        let config = DbConfig {
            path: ":memory:".to_string(), // 内存数据库
        };
        let repo = Repository::new(&config).unwrap();

        let event = WindowEvent {
            id: None,
            timestamp: Utc::now(),
            app_name: "firefox".to_string(),
            window_title: "Test".to_string(),
            workspace: "1".to_string(),
            duration_secs: 60,
            is_afk: false,
        };

        let id = repo.insert_window_event(&event).unwrap();
        assert!(id > 0);
    }

    #[test]
    fn test_get_app_usage() {
        // 测试应用使用统计查询
    }

    #[test]
    fn test_invalid_duration() {
        // 测试负数时长的错误处理
    }
}
```

**覆盖范围**:
- ✅ 数据库 CRUD 操作
- ✅ 事件解析逻辑
- ✅ 时间计算函数
- ✅ 错误处理路径

### 2. 集成测试 (Integration Tests)

**位置**: `tests/integration/` 目录

**目标**: 测试多个模块协同工作

**示例**:

```rust
// tests/integration/db_integration_test.rs
use tail_core::{Repository, DbConfig, WindowEvent};
use chrono::{Utc, Duration};

#[test]
fn test_window_event_lifecycle() {
    // 创建临时数据库
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    let config = DbConfig {
        path: db_path.to_str().unwrap().to_string(),
    };
    let repo = Repository::new(&config).unwrap();

    // 插入事件
    let event = create_test_event("firefox", 120);
    let id = repo.insert_window_event(&event).unwrap();

    // 查询验证
    let start = Utc::now() - Duration::hours(1);
    let end = Utc::now() + Duration::hours(1);
    let events = repo.get_window_events(start, end).unwrap();
    
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].app_name, "firefox");
}

#[test]
fn test_app_usage_aggregation() {
    // 测试应用使用时长聚合
}

fn create_test_event(app: &str, duration: i64) -> WindowEvent {
    WindowEvent {
        id: None,
        timestamp: Utc::now(),
        app_name: app.to_string(),
        window_title: "Test Window".to_string(),
        workspace: "1".to_string(),
        duration_secs: duration,
        is_afk: false,
    }
}
```

```rust
// tests/integration/service_integration_test.rs
use tail_service::TailService;
use tail_hyprland::HyprlandEvent;

#[tokio::test]
async fn test_event_processing() {
    // 模拟 Hyprland 事件
    // 验证服务正确处理并写入数据库
}

#[tokio::test]
async fn test_afk_detection_integration() {
    // 测试 AFK 检测与窗口事件的集成
}
```

**覆盖范围**:
- ✅ 数据库与业务逻辑集成
- ✅ IPC 客户端与事件处理集成
- ✅ AFK 检测与时间计算集成
- ✅ 多模块协同工作

### 3. 端到端测试 (E2E Tests)

**位置**: `tests/e2e/` 目录

**目标**: 测试完整的用户场景

**示例**:

```rust
// tests/e2e/window_tracking_test.rs
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
#[ignore] // 需要真实 Hyprland 环境
async fn test_complete_window_tracking_flow() {
    // 1. 启动服务
    let service = start_test_service().await;

    // 2. 模拟窗口切换
    simulate_window_switch("firefox", "GitHub").await;
    sleep(Duration::from_secs(5)).await;
    
    simulate_window_switch("kitty", "Terminal").await;
    sleep(Duration::from_secs(3)).await;

    // 3. 验证数据库记录
    let repo = get_test_repository();
    let events = repo.get_window_events(
        Utc::now() - Duration::from_secs(60),
        Utc::now()
    ).unwrap();

    assert_eq!(events.len(), 2);
    assert_eq!(events[0].app_name, "firefox");
    assert!(events[0].duration_secs >= 5);
}
```

**覆盖范围**:
- ✅ 完整的窗口追踪流程
- ✅ GUI 工作流程
- ✅ 数据持久化和查询
- ✅ 用户交互场景

---

## 测试工具和辅助函数

### tests/common/fixtures.rs

```rust
// 测试数据生成器
use tail_core::WindowEvent;
use chrono::{Utc, Duration};

pub fn create_window_event(app: &str, duration: i64) -> WindowEvent {
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

pub fn create_test_events(count: usize) -> Vec<WindowEvent> {
    let apps = vec!["firefox", "kitty", "vscode", "chrome"];
    (0..count)
        .map(|i| {
            let app = apps[i % apps.len()];
            create_window_event(app, (i as i64 + 1) * 60)
        })
        .collect()
}

pub fn create_afk_event(duration: i64) -> AfkEvent {
    AfkEvent {
        id: None,
        start_time: Utc::now() - Duration::seconds(duration),
        end_time: Some(Utc::now()),
        duration_secs: duration,
    }
}
```

### tests/common/helpers.rs

```rust
// 测试辅助函数
use tail_core::{Repository, DbConfig};
use tempfile::TempDir;

pub struct TestContext {
    pub temp_dir: TempDir,
    pub repo: Repository,
}

impl TestContext {
    pub fn new() -> Self {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        let config = DbConfig {
            path: db_path.to_str().unwrap().to_string(),
        };
        let repo = Repository::new(&config).unwrap();

        Self { temp_dir, repo }
    }
}

pub fn setup_test_db() -> Repository {
    let config = DbConfig {
        path: ":memory:".to_string(),
    };
    Repository::new(&config).unwrap()
}

pub async fn wait_for_condition<F>(mut condition: F, timeout_secs: u64) -> bool
where
    F: FnMut() -> bool,
{
    let start = std::time::Instant::now();
    while start.elapsed().as_secs() < timeout_secs {
        if condition() {
            return true;
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    false
}
```

---

## Docker 测试环境

### Dockerfile.test

```dockerfile
# 测试环境 Docker 镜像
FROM nixos/nix:latest

# 启用 Flakes
RUN echo "experimental-features = nix-command flakes" >> /etc/nix/nix.conf

# 设置工作目录
WORKDIR /workspace

# 复制项目文件
COPY . .

# 构建项目
RUN nix build .#tail-app
RUN nix build .#tail-service

# 运行测试
CMD ["nix", "develop", "--command", "cargo", "test", "--workspace"]
```

### docker-compose.test.yml

```yaml
version: '3.8'

services:
  test:
    build:
      context: .
      dockerfile: Dockerfile.test
    volumes:
      - .:/workspace
      - cargo-cache:/root/.cargo
      - nix-store:/nix
    environment:
      - RUST_LOG=debug
      - RUST_BACKTRACE=1
    command: >
      sh -c "
        cargo test --workspace --verbose &&
        cargo clippy --workspace -- -D warnings &&
        cargo fmt --check
      "

volumes:
  cargo-cache:
  nix-store:
```

---

## CI/CD 流程架构

### GitHub Actions 工作流

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Nix
        uses: cachix/install-nix-action@v22
        with:
          extra_nix_config: |
            experimental-features = nix-command flakes
      
      - name: Setup Nix Cache
        uses: cachix/cachix-action@v12
        with:
          name: tail-cache
          authToken: '${{ secrets.CACHIX_AUTH_TOKEN }}'
      
      - name: Build
        run: nix build .#tail-app .#tail-service
      
      - name: Run Unit Tests
        run: nix develop --command cargo test --lib --workspace
      
      - name: Run Integration Tests
        run: nix develop --command cargo test --test '*' --workspace
      
      - name: Run Clippy
        run: nix develop --command cargo clippy --workspace -- -D warnings
      
      - name: Check Formatting
        run: nix develop --command cargo fmt --check
      
      - name: Docker Test
        run: |
          docker-compose -f docker-compose.test.yml up --abort-on-container-exit
          docker-compose -f docker-compose.test.yml down

  coverage:
    runs-on: ubuntu-latest
    needs: test
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Nix
        uses: cachix/install-nix-action@v22
      
      - name: Run Coverage
        run: |
          nix develop --command cargo install cargo-tarpaulin
          nix develop --command cargo tarpaulin --workspace --out Xml
      
      - name: Upload Coverage
        uses: codecov/codecov-action@v3
        with:
          files: ./cobertura.xml

  build-release:
    runs-on: ubuntu-latest
    needs: test
    if: github.ref == 'refs/heads/main'
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Nix
        uses: cachix/install-nix-action@v22
      
      - name: Build Release
        run: nix build .#tail-app .#tail-service --print-build-logs
      
      - name: Create Docker Image
        run: |
          nix build .#dockerImage
          docker load < result
      
      - name: Push to Registry
        run: |
          echo "${{ secrets.DOCKER_PASSWORD }}" | docker login -u "${{ secrets.DOCKER_USERNAME }}" --password-stdin
          docker push tail:latest
```

---

## CI/CD 流程图

```
┌─────────────────────────────────────────────────────────────┐
│                    Git Push / Pull Request                   │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                   GitHub Actions Trigger                     │
└────────────────────────┬────────────────────────────────────┘
                         │
        ┌────────────────┼────────────────┐
        │                │                │
        ▼                ▼                ▼
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│  Nix Build   │  │  Unit Tests  │  │   Clippy     │
│  (并行)      │  │  (并行)      │  │   Lint       │
└──────┬───────┘  └──────┬───────┘  └──────┬───────┘
       │                 │                 │
       └────────┬────────┴────────┬────────┘
                │                 │
                ▼                 ▼
        ┌──────────────┐  ┌──────────────┐
        │ Integration  │  │   Format     │
        │    Tests     │  │    Check     │
        └──────┬───────┘  └──────┬───────┘
               │                 │
               └────────┬────────┘
                        │
                        ▼
                ┌──────────────┐
                │ Docker Test  │
                │  (E2E)       │
                └──────┬───────┘
                       │
                       ▼
                ┌──────────────┐
                │  Coverage    │
                │   Report     │
                └──────┬───────┘
                       │
                       ▼ (仅 main 分支)
                ┌──────────────┐
                │Build Release │
                │Create Docker │
                │Push Registry │
                └──────────────┘
```

---

## 测试命令速查

### 本地开发

```bash
# 运行所有测试
just test
# 或
nix develop --command cargo test --workspace

# 运行单元测试
cargo test --lib --workspace

# 运行集成测试
cargo test --test '*' --workspace

# 运行特定测试
cargo test test_insert_window_event

# 运行测试并显示输出
cargo test -- --nocapture

# 运行测试并显示详细信息
cargo test -- --show-output

# 运行被忽略的测试 (E2E)
cargo test -- --ignored

# 测试覆盖率
cargo tarpaulin --workspace --out Html
```

### Docker 测试

```bash
# 构建测试镜像
docker build -f Dockerfile.test -t tail-test .

# 运行测试
docker run --rm tail-test

# 使用 docker-compose
docker-compose -f docker-compose.test.yml up
```

### Nix 测试

```bash
# 在 Nix 环境中运行测试
nix develop --command cargo test --workspace

# 构建并测试
nix build .#tail-app && nix develop --command cargo test
```

---

## 测试覆盖率目标

| 模块 | 目标覆盖率 | 当前状态 |
|------|-----------|---------|
| tail-core | 90%+ | 待实现 |
| tail-hyprland | 80%+ | 待实现 |
| tail-afk | 85%+ | 待实现 |
| tail-service | 75%+ | 待实现 |
| tail-gui | 60%+ | 待实现 |

**总体目标**: 80%+ 代码覆盖率

---

## Mock 和 Stub 策略

### Mock Hyprland IPC

```rust
// tests/common/mocks.rs
pub struct MockHyprlandIpc {
    events: Vec<HyprlandEvent>,
}

impl MockHyprlandIpc {
    pub fn new() -> Self {
        Self { events: vec![] }
    }

    pub fn add_event(&mut self, event: HyprlandEvent) {
        self.events.push(event);
    }

    pub async fn subscribe_events<F>(&self, mut callback: F)
    where
        F: FnMut(HyprlandEvent),
    {
        for event in &self.events {
            callback(event.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_ipc() {
        let mut mock = MockHyprlandIpc::new();
        mock.add_event(HyprlandEvent::ActiveWindowChanged {
            class: "firefox".to_string(),
            title: "Test".to_string(),
        });

        let mut received = vec![];
        mock.subscribe_events(|event| {
            received.push(event);
        }).await;

        assert_eq!(received.len(), 1);
    }
}
```

### Mock AFK Detector

```rust
pub struct MockAfkDetector {
    is_afk: bool,
}

impl MockAfkDetector {
    pub fn new(is_afk: bool) -> Self {
        Self { is_afk }
    }

    pub fn check_afk(&self) -> bool {
        self.is_afk
    }

    pub fn set_afk(&mut self, is_afk: bool) {
        self.is_afk = is_afk;
    }
}
```

---

## 性能测试

### Benchmark 测试

```rust
// benches/db_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tail_core::{Repository, DbConfig, WindowEvent};

fn benchmark_insert_window_event(c: &mut Criterion) {
    let config = DbConfig {
        path: ":memory:".to_string(),
    };
    let repo = Repository::new(&config).unwrap();

    c.bench_function("insert_window_event", |b| {
        b.iter(|| {
            let event = create_test_event();
            repo.insert_window_event(black_box(&event)).unwrap()
        });
    });
}

fn benchmark_query_app_usage(c: &mut Criterion) {
    // 测试查询性能
}

criterion_group!(benches, benchmark_insert_window_event, benchmark_query_app_usage);
criterion_main!(benches);
```

```toml
# Cargo.toml
[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "db_benchmark"
harness = false
```

---

## 测试最佳实践

### 1. 测试命名规范

```rust
// ✅ 好的命名
#[test]
fn test_insert_window_event_returns_valid_id() { }

#[test]
fn test_get_app_usage_with_empty_database_returns_empty_vec() { }

#[test]
fn test_parse_event_with_invalid_format_returns_none() { }

// ❌ 不好的命名
#[test]
fn test1() { }

#[test]
fn it_works() { }
```

### 2. 测试隔离

```rust
// ✅ 每个测试使用独立的数据库
#[test]
fn test_isolated() {
    let ctx = TestContext::new(); // 创建临时数据库
    // 测试逻辑
} // 自动清理

// ❌ 共享数据库状态
static SHARED_DB: OnceCell<Repository> = OnceCell::new();
```

### 3. 测试数据清理

```rust
#[test]
fn test_with_cleanup() {
    let temp_dir = tempfile::tempdir().unwrap();
    // 测试逻辑
    // temp_dir 在作用域结束时自动删除
}
```

### 4. 异步测试

```rust
// 使用 tokio::test
#[tokio::test]
async fn test_async_operation() {
    let result = async_function().await;
    assert!(result.is_ok());
}
```

---

## 持续集成检查清单

每次提交必须通过以下检查：

- [ ] ✅ 所有单元测试通过
- [ ] ✅ 所有集成测试通过
- [ ] ✅ Clippy 无警告
- [ ] ✅ 代码格式化检查通过
- [ ] ✅ Docker 测试通过
- [ ] ✅ 代码覆盖率 ≥ 80%
- [ ] ✅ 性能基准测试无退化
- [ ] ✅ 文档构建成功

---

## 总结

TaiL 项目的测试策略：

1. **多层次测试**: 单元测试 + 集成测试 + E2E 测试
2. **统一测试目录**: 所有集成测试放在 `tests/` 目录
3. **Nix 管理**: 使用 Nix 确保测试环境一致性
4. **Docker 验证**: 使用 Docker 进行自动化验证
5. **CI/CD 自动化**: GitHub Actions 自动运行所有测试
6. **高覆盖率目标**: 80%+ 代码覆盖率
7. **Mock 和 Stub**: 隔离外部依赖，提高测试可靠性

这套测试架构确保了代码质量和功能正确性，为项目的长期维护提供了保障。