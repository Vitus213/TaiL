# TaiL 开发者指南

欢迎来到 TaiL 开发者指南！本文档面向希望了解 TaiL 架构、参与开发或贡献代码的开发者。

## 项目概述

TaiL 是一个使用 Rust 语言开发的窗口时间追踪工具，采用模块化架构设计。

### 技术栈

| 类别 | 技术 | 版本 |
|------|------|------|
| 语言 | Rust | 1.84+ |
| 异步运行时 | Tokio | 1.40 |
| 数据库 | SQLite + rusqlite | 0.32 |
| GUI | egui/eframe | 0.28 |
| 构建 | Nix Flakes | - |

## 文档目录

| 文档 | 描述 |
|------|------|
| [架构设计](architecture.md) | 项目架构、模块设计和数据流 |
| [开发环境](development.md) | 搭建开发环境和构建项目 |
| [测试指南](testing.md) | 运行和编写测试 |
| [贡献指南](contributing.md) | 代码规范和提交流程 |
| [CI/CD](ci-cd.md) | 持续集成和部署配置 |

## 项目结构

```
tail/
├── tail-core/        # 核心数据模型和数据库
├── tail-hyprland/    # Hyprand IPC 客户端
├── tail-afk/         # AFK 检测模块
├── tail-gui/         # egui 界面
├── tail-service/     # 后台服务
├── tail-app/         # 应用入口
└── tests/            # 集成测试
```

## 快速开始

### 克隆仓库

```bash
git clone https://github.com/vitus213/tail.git
cd tail
```

### 使用 Nix（推荐）

```bash
nix develop
cargo build
cargo test
```

### 手动安装依赖

```bash
# Arch Linux
pacman -S rust pkg-config wayland libxkbcommon

# Ubuntu/Debian
apt install rustc cargo pkg-config libwayland-dev libxkbcommon-dev
```

## 下一步

- 查看 [架构设计](architecture.md) 了解项目整体架构
- 查看 [开发环境](development.md) 搭建开发环境
- 查看 [贡献指南](contributing.md) 了解如何贡献代码

## 获取帮助

- 提交 [Issue](https://github.com/vitus213/tail/issues)
- 加入 [Discussions](https://github.com/vitus213/tail/discussions)
