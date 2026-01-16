# TaiL 开发环境搭建

本文档介绍如何搭建 TaiL 的开发环境。

## 目录

- [前置要求](#前置要求)
- [使用 Nix（推荐）](#使用-nix推荐)
- [手动安装依赖](#手动安装依赖)
- [构建项目](#构建项目)
- [运行项目](#运行项目)
- [开发工具](#开发工具)

---

## 前置要求

- **Rust** 1.84 或更高版本
- **Hyprland** 窗口管理器（用于测试）
- **SQLite** 开发库

---

## 使用 Nix（推荐）

TaiL 使用 Nix Flakes 管理开发环境，确保依赖版本一致。

### 启用 Flakes

如果尚未启用 Nix Flakes：

```bash
mkdir -p ~/.config/nix
echo "experimental-features = nix-command flakes" >> ~/.config/nix/nix.conf
```

### 进入开发环境

```bash
cd /path/to/tail
nix develop
```

这将自动安装所有依赖并设置环境变量。

### 使用 direnv（可选）

安装 direnv 后，可以自动进入开发环境：

```bash
# 安装 direnv（如果未安装）
nix profile install nixpkgs#direnv

# 允许 direnv
direnv allow
```

现在每次进入目录都会自动进入开发环境。

---

## 手动安装依赖

### Arch Linux

```bash
# Rust 工具链
sudo pacman -S rust cargo

# 系统依赖
sudo pacman -S pkg-config wayland libxkbcommon sqlite3

# Wayland 开发包
sudo pacman -S wayland-protocols
```

### Ubuntu/Debian

```bash
# Rust 工具链
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 系统依赖
sudo apt update
sudo apt install -y \
    pkg-config \
    libwayland-dev \
    libxkbcommon-dev \
    libsqlite3-dev \
    wayland-protocols
```

### Fedora

```bash
# Rust 工具链
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 系统依赖
sudo dnf install -y \
    pkg-config \
    wayland-devel \
    libxkbcommon-devel \
    sqlite-devel \
    wayland-protocols-devel
```

---

## 构建项目

### 使用 Cargo

```bash
# 克隆仓库
git clone https://github.com/vitus213/tail.git
cd tail

# Debug 构建（快速编译，用于开发）
cargo build

# Release 构建（优化性能，用于测试）
cargo build --release

# 只构建特定包
cargo build -p tail-app
cargo build -p tail-service
```

### 使用 Nix

```bash
# 构建
nix build .#tail-app
nix build .#tail-service

# 或使用 just
just build
```

### 使用 just 命令

项目提供了 `justfile` 简化常用命令：

```bash
# 查看所有命令
just --list

# 构建
just build

# 运行
just run

# 运行服务
just run-service

# 测试
just test

# 代码检查
just clippy
```

---

## 运行项目

### 运行 GUI

```bash
# 使用 Cargo
cargo run -p tail-app

# 使用 Nix
nix run .#tail-app

# 使用 just
just run
```

### 运行服务

```bash
# 使用 Cargo
cargo run -p tail-service

# 使用 Nix
nix run .#tail-service

# 使用 just
just run-service
```

### 运行测试

```bash
# 所有测试
cargo test --workspace

# 单元测试
cargo test --lib

# 集成测试
cargo test --test '*'

# 显示输出
cargo test -- --nocapture
```

---

## 开发工具

### 代码格式化

```bash
# 检查格式
cargo fmt --check

# 自动格式化
cargo fmt

# 或使用 just
just fmt
```

### 代码检查

```bash
# Clippy 检查
cargo clippy --workspace --all-targets

# 修复可自动修复的问题
cargo clippy --workspace --fix

# 或使用 just
just clippy
```

### IDE 配置

#### VS Code

安装以下扩展：
- **rust-analyzer** - Rust 语言服务器
- **CodeLLDB** - 调试器
- **Even Better TOML** - TOML 支持

`.vscode/settings.json` 推荐配置：

```json
{
    "rust-analyzer.cargo.features": "all",
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.cargo.loadOutDirsFromCheck": true
}
```

#### Neovim

使用 `rust-analyzer` 配置：

```lua
require('lspconfig').rust_analyzer.setup({
    settings = {
        ['rust-analyzer'] = {
            cargo = {
                features = "all",
            },
            checkOnSave = {
                command = "clippy",
            },
        },
    },
})
```

### 调试

#### 查看日志

设置日志级别：

```bash
RUST_LOG=debug cargo run -p tail-service
```

#### 使用 lldb

```bash
# 安装 lldb
sudo apt install lldb  # Ubuntu/Debian
sudo pacman -S lldb     # Arch

# 调试
rust-lldb target/debug/tail-app
```

---

## 开发工作流

### 典型开发流程

1. **拉取最新代码**
   ```bash
   git pull origin main
   ```

2. **更新依赖**
   ```bash
   nix flake update  # 如果使用 Nix
   cargo update      # 或使用 Cargo
   ```

3. **创建功能分支**
   ```bash
   git checkout -b feature/your-feature
   ```

4. **开发**
   ```bash
   # 编辑代码
   # 运行测试
   cargo test
   # 格式化代码
   cargo fmt
   # 检查代码
   cargo clippy
   ```

5. **提交**
   ```bash
   git add .
   git commit -m "feat: add your feature"
   ```

6. **推送**
   ```bash
   git push origin feature/your-feature
   ```

---

## 常见问题

### 构建错误：缺少链接库

**错误信息：**
```
cannot find -lwayland-client
```

**解决方法：**
安装 Wayland 开发包（参见手动安装依赖）。

### 测试失败：不在 Hyprland 中

**错误信息：**
```
Socket path not found. Is HYPRLAND_INSTANCE_SIGNATURE set?
```

**解决方法：**
某些测试需要 Hyprland 环境。可以跳过这些测试：
```bash
cargo test -- --ignored
```

### Nix 构建缓慢

**解决方法：**
使用 Cachix 或本地二进制缓存：

```bash
# 启用 nix-command
experimental-features = nix-command flakes

# 使用 substitutors
echo "max-jobs = auto" >> ~/.config/nix/nix.conf
echo "builders = @" >> ~/.config/nix/nix.conf
```

---

## 下一步

- 查看 [架构设计](architecture.md) 了解项目结构
- 查看 [测试指南](testing.md) 了解测试方法
- 查看 [贡献指南](contributing.md) 了解贡献流程
