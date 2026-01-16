# TaiL 贡献指南

感谢你对 TaiL 的关注！本文档介绍如何为 TaiL 贡献代码。

## 目录

- [行为准则](#行为准则)
- [贡献方式](#贡献方式)
- [开发流程](#开发流程)
- [代码规范](#代码规范)
- [提交 Pull Request](#提交-pull-request)

---

## 行为准则

- 尊重所有贡献者
- 接受建设性批评
- 关注什么对社区最有利
- 对不同观点保持同理心

---

## 贡献方式

### 报告 Bug

在 [GitHub Issues](https://github.com/vitus213/tail/issues) 报告 Bug 时，请包含：

1. **Bug 描述**
2. **复现步骤**
3. **预期行为**
4. **实际行为**
5. **环境信息**
   - 操作系统版本
   - Hyprland 版本
   - TaiL 版本

### 提交功能请求

在提交功能请求时，请说明：

1. **功能描述**
2. **使用场景**
3. **可能的实现方案**（可选）

### 改进文档

文档是项目的重要组成部分！你可以：

- 修正错别字和错误
- 改进现有文档的清晰度
- 添加缺失的文档
- 翻译文档

### 贡献代码

如果你想贡献代码，请先查看 [Issues](https://github.com/vitus213/tail/issues)：

- 标记为 `good first issue` 的问题适合新手
- 标记为 `help wanted` 的问题欢迎贡献

---

## 开发流程

### 1. Fork 仓库

点击 GitHub 页面右上角的 Fork 按钮。

### 2. 克隆你的 Fork

```bash
git clone https://github.com/yourusername/tail.git
cd tail
```

### 3. 添加上游远程仓库

```bash
git remote add upstream https://github.com/vitus213/tail.git
```

### 4. 创建功能分支

```bash
git checkout -b feature/your-feature-name
```

分支命名约定：
- `feature/xxx` - 新功能
- `fix/xxx` - Bug 修复
- `docs/xxx` - 文档更新
- `refactor/xxx` - 代码重构
- `test/xxx` - 测试相关

### 5. 进行开发

```bash
# 进入开发环境
nix develop

# 进行修改
# ...

# 运行测试
cargo test --workspace

# 格式化代码
cargo fmt

# 检查代码
cargo clippy --workspace --all-targets
```

### 6. 提交更改

```bash
git add .
git commit -m "feat: add your feature"
```

### 7. 同步上游

```bash
git fetch upstream
git rebase upstream/main
```

### 8. 推送到你的 Fork

```bash
git push origin feature/your-feature-name
```

### 9. 创建 Pull Request

在 GitHub 上创建 Pull Request 到 `vitus213/tail:main`。

---

## 代码规范

### Rust 代码风格

遵循 Rust 官方风格指南：

```bash
# 格式化代码
cargo fmt

# 检查格式
cargo fmt --check
```

### 使用 Clippy

```bash
# 运行 Clippy
cargo clippy --workspace --all-targets

# 自动修复
cargo clippy --workspace --fix
```

### 代码组织

- 每个模块应该有单一职责
- 使用清晰的命名
- 添加必要的文档注释
- 保持函数简短

### 文档注释

公开的函数和结构体应该有文档注释：

```rust
/// Represents a window usage event.
///
/// This struct contains information about a window's usage,
/// including the application name, window title, duration, etc.
pub struct WindowEvent {
    /// The application name (e.g., "code", "firefox")
    pub app_name: String,

    /// The window title
    pub window_title: String,

    /// Start time of the event
    pub start_time: DateTime<Utc>,

    /// Duration in seconds
    pub duration_secs: i64,

    /// Whether the user was AFK during this event
    pub is_afk: bool,
}
```

---

## Commit 消息规范

使用 [Conventional Commits](https://www.conventionalcommits.org/) 格式：

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Type 类型

- `feat`: 新功能
- `fix`: Bug 修复
- `docs`: 文档更新
- `style`: 代码格式（不影响代码运行的变动）
- `refactor`: 重构（既不是新功能也不是修复）
- `perf`: 性能优化
- `test`: 添加测试
- `chore`: 构建过程或辅助工具的变动

### 示例

```bash
feat(service): add AFK detection integration

Implement AFK detector integration in TailService to track
user idle time and mark events accordingly.

Closes #123
```

```bash
fix(gui): resolve window resize rendering issue

The window content was not properly resizing when the
window was maximized or resized manually.

Fixes #456
```

---

## Pull Request 规范

### PR 标题

使用与 Commit 消息相同的格式：

```
feat: add category support for applications
fix: resolve database connection leak
docs: update installation guide for NixOS
```

### PR 描述

在 PR 描述中包含：

1. **变更摘要**
2. **相关 Issue**（如 `Closes #123`）
3. **测试说明**
4. **截图**（如果是 UI 变更）

### PR 检查清单

在提交 PR 前，确保：

- [ ] 代码通过所有测试
- [ ] 代码通过 Clippy 检查
- [ ] 代码已格式化
- [ ] 添加了必要的测试
- [ ] 更新了相关文档
- [ ] Commit 消息符合规范

---

## 代码审查

所有 PR 都需要经过代码审查：

1. **自动检查** - CI 必须通过
2. **人工审查** - 维护者会审查代码
3. **修改** - 根据反馈进行修改
4. **合并** - 审查通过后合并

---

## 获取帮助

如果你在贡献过程中遇到问题：

- 查看 [开发文档](development.md)
- 查看 [测试指南](testing.md)
- 在 [Discussions](https://github.com/vitus213/tail/discussions) 提问
- 在 Issue 中请求帮助

---

## 许可证

贡献的代码将采用与项目相同的 [MIT License](../LICENSE)。
