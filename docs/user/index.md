# TaiL 用户指南

欢迎使用 TaiL！本指南将帮助你安装、配置和使用 TaiL。

## 什么是 TaiL？

TaiL 是一个专为 Hyprland/Wayland 窗口管理器设计的窗口使用时间追踪工具。它可以帮助你：

- 自动追踪每个窗口的使用时间
- 通过 GUI 界面查看统计数据
- 检测空闲时间（AFK）
- 设置应用使用时长目标

## 文档目录

| 文档 | 描述 |
|------|------|
| [安装指南](installation.md) | 在 NixOS 或其他 Linux 发行版上安装 TaiL |
| [快速开始](quick-start.md) | 5 分钟快速上手 TaiL |
| [使用指南](usage.md) | 详细的功能说明和使用方法 |
| [配置说明](configuration.md) | NixOS 模块配置和服务管理 |
| [故障排查](troubleshooting.md) | 常见问题和解决方案 |

## 前置要求

使用 TaiL 需要满足以下条件：

1. **Hyprland 窗口管理器** - TaiL 目前仅支持 Hyprland
2. **Wayland 会话** - 确保在 Wayland 环境下运行
3. **Rust 1.84+** - 如果从源码构建

## 下一步

如果你是第一次使用 TaiL，建议从 [安装指南](installation.md) 开始。

如果你已经安装了 TaiL，可以直接查看 [快速开始](quick-start.md) 了解基本用法。
