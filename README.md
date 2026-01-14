# TaiL - Window time tracker for Hyprland/Wayland

![Rust](https://img.shields.io/badge/rust-1.84+-orange.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)

专为 Hyprland/Wayland 设计的窗口使用时间追踪工具，参考 Windows 下 Tai 软件和 ActivityWatch 的设计理念。

## 功能特性

- 🔍 **自动窗口追踪** - 通过 Hyprland IPC 实时监听窗口活动
- 📊 **可视化统计** - 原生 GUI 界面展示使用数据
- ⏱️ **AFK 检测** - 自动检测空闲时间
- 🎯 **目标限制** - 设置应用使用时长限制和提醒
- 📈 **多维度统计** - 按小时/天/周/月查看时间分布

## 快速安装

### NixOS 用户（一键安装）

```bash
# 方法一：一键打包
just nix-package

# 方法二：直接运行（无需安装）
nix run github:yourusername/TaiL

# 方法三：安装到用户环境
just nix-install-local
```

**详细的 NixOS 安装指南请查看：[NIXOS_INSTALL.md](NIXOS_INSTALL.md)**

### 其他 Linux 发行版

```bash
# 使用 Nix 包管理器
curl --proto '=https' --tlsv1.2 -sSf -L https://nixos.org/nix/install | sh
nix run github:yourusername/TaiL
```

## 开发环境

### 使用 Nix (推荐)

```bash
# 启用 Flakes
mkdir -p ~/.config/nix
echo "experimental-features = nix-command flakes" >> ~/.config/nix/nix.conf

# 进入开发环境
nix develop

# 或者使用 direnv
direnv allow
```

### Docker 测试环境

```bash
docker build -t tail-dev .
docker run -it --rm tail-dev
```

### 手动安装依赖

```bash
# Arch Linux
pacman -S rust pkg-config wayland libxkbcommon

# Ubuntu/Debian
apt install rustc cargo pkg-config libwayland-dev libxkbcommon-dev
```

## 构建和运行

### 使用 just 命令（最简单）

```bash
# 查看所有命令
just

# 一键打包给 NixOS
just nix-package

# 运行 GUI
just run

# 运行后台服务
just run-service

# 运行测试
just test
```

### 使用 Nix

```bash
# 构建
nix build .#tail-app
nix build .#tail-service

# 运行
nix run .#tail-app
```

### 使用 Cargo

```bash
# 构建
cargo build --release

# 运行
cargo run --release -p tail-app
```

## 项目结构

```
tail/
├── flake.nix          # Nix Flakes 配置
├── Cargo.toml         # Workspace 配置
├── tail-core/         # 核心数据模型和数据库
├── tail-hyprland/     # Hyprland IPC 客户端
├── tail-afk/          # AFK 检测模块
├── tail-gui/          # egui 界面
├── tail-service/      # 后台服务
└── tail-app/          # 应用入口
```

## NixOS 集成

TaiL 提供完整的 NixOS 模块支持：

```nix
# 在 configuration.nix 中
services.tail = {
  enable = true;
  user = "yourusername";
  afkTimeout = 300;
  logLevel = "info";
  autoStart = true;
};
```

详细配置请查看 [NIXOS_INSTALL.md](NIXOS_INSTALL.md)

## 架构设计

- **高内聚低耦合** - 模块间通过明确的接口通信
- **可复现构建** - Nix Flakes 保证环境一致性
- **事件驱动** - 基于 Tokio 异步运行时

## 文档

- 📖 [运行指南](RUNNING_GUIDE.md) - 详细的运行说明
- 🐧 [NixOS 安装](NIXOS_INSTALL.md) - NixOS 一键安装指南
- 📊 [开发总结](DEVELOPMENT_SUMMARY.md) - 项目开发总结
- 🏗️ [架构文档](plans/architecture-summary.md) - 架构设计详解

## 测试

```bash
# 运行所有测试
just test

# 或使用 cargo
cargo test --workspace
```

✅ **27 个测试全部通过**（21 个单元测试 + 6 个集成测试）

## 许可证

MIT License
