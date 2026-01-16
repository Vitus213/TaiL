# TaiL 快速开始

本指南将帮助你在 5 分钟内开始使用 TaiL。

## 前置条件

- Hyprland 窗口管理器
- 在 Wayland 会话中运行

## 快速安装

### NixOS 用户

```bash
nix run github:vitus213/tail
```

### 其他发行版

```bash
cargo install tail
```

## 启动和使用

### 步骤 1：启动后台服务

```bash
tail-service
```

服务会：
- 连接到 Hyprland IPC
- 监听窗口切换事件
- 记录使用时间到数据库
- 检测 AFK（空闲）状态

### 步骤 2：运行 GUI

打开新终端：

```bash
tail-app
```

你会看到：
- **今日统计** - 总使用时间和应用排行
- **应用列表** - 每个应用的使用时长和进度条
- **时间导航** - 切换今天/昨天/本周等时间范围

### 步骤 3：开始追踪

现在你可以正常使用电脑，TaiL 会自动在后台记录每个窗口的使用时间。

切换到不同的窗口或应用时，服务会自动更新记录。

## 基本使用

### 查看今日统计

GUI 默认显示今日的使用统计：

```
┌─────────────────────────────────────┐
│  今日统计                           │
│  ┌─────────────────────────────┐   │
│  │  总时长: 4小时 32分钟        │   │
│  │  ━━━━━━━━━━━━━━━━━━━━━━━━━  │   │
│  └─────────────────────────────┘   │
│                                     │
│  🏆 应用排行                        │
│  [图标] code        2h 15m  ████   │
│  [图标] firefox     1h 30m  ██     │
│  [图标] terminal    45m     █      │
└─────────────────────────────────────┘
```

### 切换时间范围

在统计页面，你可以：
- 点击"今天"查看今日统计
- 点击"昨天"查看昨日统计
- 点击"本周"查看本周统计
- 点击左/右箭头切换日期

### 设置使用目标

在设置页面，你可以为应用设置每日使用目标：

1. 切换到"设置"标签
2. 点击"添加目标"
3. 选择应用并设置时长限制

## 服务管理

### 检查服务状态

```bash
systemctl --user status tail
```

### 查看服务日志

```bash
# 实时查看
journalctl --user -u tail -f

# 查看最近日志
journalctl --user -u tail -n 50
```

### 重启服务

```bash
systemctl --user restart tail
```

### 停止服务

```bash
systemctl --user stop tail
```

## 数据位置

所有数据保存在：

```
~/.local/share/tail/tail.db
```

### 查看数据库

```bash
sqlite3 ~/.local/share/tail/tail.db "SELECT * FROM window_events ORDER BY start_time DESC LIMIT 10;"
```

## 下一步

- 查看 [使用指南](usage.md) 了解更多功能
- 查看 [配置说明](configuration.md) 配置 NixOS 模块
- 查看 [故障排查](troubleshooting.md) 解决问题

## 常见问题

**Q: GUI 无法启动？**

确保在 Hyprland 会话中运行：
```bash
echo $HYPRLAND_INSTANCE_SIGNATURE
```

**Q: 没有记录数据？**

检查服务是否正在运行：
```bash
systemctl --user status tail
```

**Q: 如何重置数据？**

```bash
# 停止服务
systemctl --user stop tail

# 删除数据库
rm ~/.local/share/tail/tail.db

# 重启服务
systemctl --user start tail
```
