# TaiL 架构总结

## 核心架构概览

TaiL 是一个为 Hyprland/Wayland 设计的窗口时间追踪工具，采用 **分层模块化架构**。

### 架构层次

```
表现层 (tail-gui, tail-app)
    ↓
业务层 (tail-service)
    ↓
数据层 (tail-core)
    ↓
集成层 (tail-hyprland, tail-afk)
```

---

## 六大核心模块

| 模块 | 职责 | 关键技术 |
|------|------|----------|
| **tail-app** | 应用入口 | eframe 启动 |
| **tail-gui** | 用户界面 | egui UI 框架 |
| **tail-service** | 业务协调 | Tokio 异步 + mpsc channel |
| **tail-core** | 数据持久化 | SQLite + r2d2 连接池 |
| **tail-hyprland** | Hyprland 集成 | Unix Socket IPC |
| **tail-afk** | 空闲检测 | 输入设备监听 |

---

## 关键数据流

### 窗口追踪流程

```
Hyprland → tail-hyprland → tail-service → tail-core → SQLite
  (IPC)      (事件解析)      (时长计算)    (Repository)  (持久化)
```

### 数据查询流程

```
tail-gui → tail-core → SQLite
 (UI请求)  (Repository)  (查询)
```

---

## 核心数据模型

### WindowEvent (窗口事件)
记录每次窗口切换的详细信息和使用时长

**关键字段**:
- `timestamp`: 事件时间
- `app_name`: 应用名称
- `duration_secs`: 使用时长
- `is_afk`: 是否空闲

### AfkEvent (空闲事件)
追踪用户离开计算机的时段

### DailyGoal (每日目标)
设置应用使用时长限制

---

## 技术栈

| 类别 | 技术 | 用途 |
|------|------|------|
| 语言 | Rust 1.84+ | 系统编程 |
| 运行时 | Tokio | 异步事件处理 |
| 数据库 | SQLite + rusqlite | 本地数据存储 |
| 连接池 | r2d2 | 数据库连接管理 |
| GUI | egui/eframe | 原生跨平台界面 |
| 构建 | Nix Flakes | 可复现构建 |
| 日志 | tracing | 结构化日志 |
| 错误处理 | thiserror + anyhow | 类型安全错误 |

---

## 并发模型

### 服务端 (异步)

```rust
Tokio Runtime
├─ Task 1: Hyprland IPC 订阅 (异步读取)
│  └─ mpsc::send(Event)
├─ Task 2: 事件处理循环
│  └─ mpsc::recv() → 计算时长 → 写数据库
└─ Task 3: AFK 检测 (定期检查)
```

### 客户端 (同步)

```rust
eframe::App
└─ update() 每帧
   ├─ 处理用户输入
   ├─ 查询数据 (同步)
   └─ 渲染 UI
```

---

## 设计原则

### 1. 高内聚低耦合
- 每个模块职责单一
- 通过明确接口通信
- 避免循环依赖

### 2. 事件驱动
- 基于 Tokio 异步运行时
- 使用 channel 解耦组件
- 响应式处理事件

### 3. 可扩展性
- 支持多窗口管理器 (Hyprland, Sway, ...)
- 支持多 GUI 框架 (egui, Tauri, ...)
- 插件化 AFK 检测

### 4. 可维护性
- Nix Flakes 保证构建一致性
- 完善的文档和注释
- 清晰的错误处理策略

---

## 性能优化

### 数据库层面
- ✅ 连接池 (r2d2, 最大 10 连接)
- ✅ 索引优化 (timestamp, app_name)
- ✅ 批量写入 (事务)

### 内存层面
- ✅ 有界 channel (容量 100)
- ✅ 分页加载历史数据

### 延迟层面
- ✅ 异步数据库写入
- ✅ UI 数据缓存

---

## 当前实现状态

### ✅ 已完成
- 基础项目结构
- 数据模型和 Schema
- Hyprland IPC 客户端
- 基础 GUI 框架
- Nix 构建配置

### 🚧 进行中
- 事件处理逻辑 (部分)
- AFK 检测集成
- GUI 视图实现

### ❌ 待实现
- 窗口时长计算
- 完整数据查询 API
- 图表和统计展示
- 目标限制和通知
- 数据导出功能

---

## 架构优势

### 1. 模块化设计
- 清晰的职责划分
- 易于测试和维护
- 支持独立演进

### 2. 技术选型合理
- Rust: 内存安全 + 高性能
- Tokio: 成熟异步生态
- SQLite: 零配置嵌入式数据库
- egui: 原生跨平台 GUI

### 3. 扩展性强
- 接口抽象支持多实现
- 插件化设计
- 配置驱动

---

## 改进建议

### 短期 (1-2 周)
1. **完善状态管理**: 在 `TailService` 中添加完整的窗口状态跟踪
2. **实现时长计算**: 计算窗口切换时的使用时长
3. **集成 AFK 检测**: 将 AFK 状态与窗口事件关联

### 中期 (1 个月)
1. **添加配置系统**: 使用 TOML 配置文件
2. **实现 GUI 图表**: 使用 egui_plot 展示统计数据
3. **添加通知系统**: 使用 notify-rust 发送桌面通知

### 长期 (3 个月)
1. **GUI-Service 通信**: 实现进程间通信
2. **数据导出功能**: 支持 CSV/JSON 导出
3. **系统托盘集成**: 后台运行 + 托盘图标
4. **多窗口管理器支持**: 扩展到 Sway, i3 等

---

## 关键文件路径

```
TaiL/
├── tail-core/src/
│   ├── models.rs          # 数据模型定义
│   └── db.rs              # 数据库访问层
├── tail-hyprland/src/
│   └── ipc.rs             # Hyprland IPC 客户端
├── tail-service/src/
│   └── service.rs         # 业务逻辑协调
├── tail-gui/src/
│   ├── app.rs             # GUI 应用主体
│   ├── views.rs           # UI 视图组件
│   └── theme.rs           # 主题系统
└── tail-app/src/
    └── main.rs            # 应用入口点
```

---

## 数据库位置

**默认路径**: `~/.local/share/tail/tail.db`

**Schema**:
- `window_events`: 窗口使用记录
- `afk_events`: 空闲时段记录
- `daily_goals`: 应用使用限制

---

## 测试策略

### 测试金字塔

```
    ┌─────────────────┐
    │   E2E 测试      │  (少量，Docker 验证)
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
TaiL/
├── tests/                    # 统一集成测试目录
│   ├── common/              # 测试工具和辅助函数
│   ├── integration/         # 集成测试
│   └── e2e/                 # 端到端测试
├── Dockerfile.test          # 测试环境镜像
└── docker-compose.test.yml  # Docker 测试配置
```

### 测试覆盖率目标

| 模块 | 目标覆盖率 |
|------|-----------|
| tail-core | 90%+ |
| tail-hyprland | 80%+ |
| tail-service | 75%+ |
| **总体** | **80%+** |

---

## CI/CD 流程

### GitHub Actions 工作流

```
Git Push
    ↓
┌─────────────────────────────┐
│  并行执行                    │
│  - Nix Build               │
│  - Unit Tests              │
│  - Clippy Lint             │
└────────┬────────────────────┘
         ↓
┌─────────────────────────────┐
│  - Integration Tests       │
│  - Format Check            │
└────────┬────────────────────┘
         ↓
┌─────────────────────────────┐
│  Docker E2E Tests          │
└────────┬────────────────────┘
         ↓
┌─────────────────────────────┐
│  Coverage Report           │
└────────┬────────────────────┘
         ↓ (仅 main 分支)
┌─────────────────────────────┐
│  - Build Release           │
│  - Create Docker Image     │
│  - Push to Registry        │
└─────────────────────────────┘
```

### CI 检查清单

每次提交必须通过：
- ✅ 所有单元测试
- ✅ 所有集成测试
- ✅ Clippy 无警告
- ✅ 代码格式化检查
- ✅ Docker 测试
- ✅ 代码覆盖率 ≥ 80%

---

## 开发命令速查

### 本地开发

```bash
# 进入开发环境
nix develop

# 构建项目
just build

# 运行 GUI
just run

# 运行服务
just run-service

# 代码检查
just check
just clippy

# 格式化
just fmt

# 测试
just test                    # 所有测试
cargo test --lib            # 单元测试
cargo test --test '*'       # 集成测试
cargo test -- --ignored     # E2E 测试
```

### Docker 测试

```bash
# 构建测试镜像
docker build -f Dockerfile.test -t tail-test .

# 运行测试
docker-compose -f docker-compose.test.yml up

# 测试覆盖率
cargo tarpaulin --workspace --out Html
```

---

## 架构决策记录 (ADR)

### ADR-001: 选择 Rust 作为主要语言
**原因**: 内存安全、高性能、丰富的异步生态

### ADR-002: 使用 SQLite 而非 PostgreSQL
**原因**: 嵌入式、零配置、适合单用户场景

### ADR-003: 选择 egui 而非 GTK/Qt
**原因**: 纯 Rust、跨平台、即时模式 GUI

### ADR-004: 使用 Nix Flakes 构建
**原因**: 可复现构建、依赖版本锁定、开发环境一致性

### ADR-005: 分层模块化架构
**原因**: 高内聚低耦合、易于测试、支持扩展

---

## 总结

TaiL 项目采用了**清晰的分层架构**和**模块化设计**，核心特点:

1. **职责明确**: 6 个模块各司其职
2. **依赖合理**: 单向依赖流，无循环
3. **技术成熟**: Rust 生态成熟库
4. **易于扩展**: 接口抽象 + 插件化
5. **性能优良**: 异步 + 数据库优化

**当前状态**: 核心架构完成，需补充业务逻辑

**下一步**: 完善事件处理 → 实现 GUI 展示 → 添加通知功能