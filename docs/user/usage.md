# TaiL 使用指南

本文档详细介绍 TaiL 的功能和使用方法。

## 目录

- [运行模式](#运行模式)
- [GUI 界面](#gui-界面)
- [服务管理](#服务管理)
- [数据管理](#数据管理)

---

## 运行模式

TaiL 有两种运行模式：

### 后台服务模式

后台服务 `tail-service` 负责持续追踪窗口活动：

```bash
tail-service
```

**功能：**
- 监听 Hyprland 窗口切换事件
- 计算窗口使用时长
- 检测 AFK（空闲）状态
- 持久化数据到数据库

### GUI 模式

GUI 应用 `tail-app` 用于查看统计数据：

```bash
tail-app
```

**功能：**
- 查看今日使用统计
- 查看历史数据（多时间范围）
- 设置每日使用目标
- 查看详细记录

---

## GUI 界面

### 主界面布局

```
┌─────────────────────────────────────────────────────────┐
│  TaiL                    [仪表板] [统计] [设置]         │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  📊 今日统计                                             │
│  ┌─────────────────────────────────────────────────┐   │
│  │  总使用时间                                      │   │
│  │  6小时 23分钟                                    │   │
│  │  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  │   │
│  └─────────────────────────────────────────────────┘   │
│                                                         │
│  🏆 应用使用排行                                         │
│  ┌─────────────────────────────────────────────────┐   │
│  │  [图标] Visual Studio Code        5h 23m  ████  │   │
│  │        /home/user/project                      │   │
│  │                                                 │   │
│  │  [图标] Firefox                   2h 15m  ██    │   │
│  │        github.com                              │   │
│  │                                                 │   │
│  │  [图标] Terminal                  54m     █     │   │
│  │        bash                                    │   │
│  └─────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

### 视图说明

#### 仪表板（Dashboard）

显示今日使用统计：
- **今日总时长** - 今日累计使用电脑的总时间
- **应用排行** - 按使用时长排序的应用列表
- **进度条** - 每个应用占用今日总时长的比例

#### 统计（Statistics）

查看历史数据：
- **时间范围选择** - 今天/昨天/本周/本月/自定义
- **柱形图** - 按时间维度的使用分布
- **详细列表** - 应用/分类的详细数据

**时间导航：**
- 点击"今天"、"昨天"等快捷按钮
- 使用左右箭头切换相邻时间段
- 点击柱形图可下钻到更细粒度

#### 设置（Settings）

管理应用使用目标：
- **添加目标** - 为应用设置每日使用时长限制
- **删除目标** - 移除不再需要的目标
- **数据库信息** - 查看数据库路径和大小

---

## 服务管理

### 使用 systemd（推荐）

#### 启动服务

```bash
systemctl --user start tail
```

#### 停止服务

```bash
systemctl --user stop tail
```

#### 重启服务

```bash
systemctl --user restart tail
```

#### 查看状态

```bash
systemctl --user status tail
```

#### 查看日志

```bash
# 实时查看
journalctl --user -u tail -f

# 查看最近 50 行
journalctl --user -u tail -n 50

# 查看今天的日志
journalctl --user -u tail --since today
```

#### 开机自动启动

```bash
systemctl --user enable tail
```

#### 禁用自动启动

```bash
systemctl --user disable tail
```

### 使用 Hyprland 自动启动

编辑 `~/.config/hypr/hyprland.conf`：

```bash
# 自动启动 TaiL 服务
exec-once = tail-service
```

### 手动运行

```bash
# 前台运行
tail-service

# 后台运行
nohup tail-service > tail.log 2>&1 &
```

---

## 数据管理

### 数据存储位置

```
~/.local/share/tail/tail.db
```

### 数据库结构

- `window_events` - 窗口使用记录
- `afk_events` - 空闲时段记录
- `daily_goals` - 应用使用限制

### 查询数据

```bash
# 查看最近 10 条窗口事件
sqlite3 ~/.local/share/tail/tail.db \
  "SELECT * FROM window_events ORDER BY start_time DESC LIMIT 10;"

# 查看今日应用使用统计
sqlite3 ~/.local/share/tail/tail.db \
  "SELECT app_name, SUM(duration_secs) as total FROM window_events
   WHERE date(start_time) = date('now')
   GROUP BY app_name ORDER BY total DESC;"
```

### 导出数据

```bash
# 导出为 CSV
sqlite3 ~/.local/share/tail/tail.db \
  -header -csv \
  "SELECT * FROM window_events" > usage.csv
```

### 备份数据库

```bash
# 创建备份
cp ~/.local/share/tail/tail.db ~/.local/share/tail/tail.db.backup

# 或使用 SQLite 导出
sqlite3 ~/.local/share/tail/tail.db ".backup ~/.local/share/tail/tail.db.backup"
```

### 删除数据

```bash
# 停止服务
systemctl --user stop tail

# 删除数据库
rm ~/.local/share/tail/tail.db

# 重启服务（会创建新数据库）
systemctl --user start tail
```

### 优化数据库

```bash
# 清理数据库碎片
sqlite3 ~/.local/share/tail/tail.db "VACUUM;"
```

---

## 环境变量

### RUST_LOG

控制日志输出级别：

```bash
# 只显示错误
RUST_LOG=error tail-service

# 显示信息（默认）
RUST_LOG=info tail-service

# 显示调试信息
RUST_LOG=debug tail-service

# 显示详细追踪
RUST_LOG=trace tail-service
```

### HYPRLAND_INSTANCE_SIGNATURE

TaiL 需要此环境变量来连接 Hyprland。通常由 Hyprland 自动设置。

如果服务无法连接，检查：

```bash
echo $HYPRLAND_INSTANCE_SIGNATURE
```

---

## 故障排查

### 问题：找不到 Hyprland socket

**错误信息：**
```
Socket path not found. Is HYPRLAND_INSTANCE_SIGNATURE set?
```

**解决方法：**
1. 确保在 Hyprland 会话中运行
2. 检查环境变量：`echo $HYPRLAND_INSTANCE_SIGNATURE`
3. 查看服务日志：`journalctl --user -u tail -n 50`

### 问题：GUI 无法启动

**解决方法：**
1. 检查是否安装了 Wayland 相关库
2. 查看错误信息：`tail-app 2>&1`

### 问题：没有记录数据

**解决方法：**
1. 确认服务正在运行：`systemctl --user status tail`
2. 查看服务日志是否有错误
3. 检查数据库文件是否存在

更多问题请查看 [故障排查文档](troubleshooting.md)。
