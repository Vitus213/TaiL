# TaiL 快速启用指南

## 📦 安装后启用步骤

### 方法一：NixOS 系统级启用（推荐）

#### 1. 在您的 NixOS 配置中启用服务

编辑 `/home/vitus/nixos-config/configuration.nix` 或相应的配置文件：

```nix
{ config, pkgs, ... }:

{
  # 启用 TaiL 服务
  services.tail = {
    enable = true;              # 启用服务
    user = "vitus";             # 替换为您的用户名
    autoStart = true;           # 开机自动启动
    logLevel = "info";          # 日志级别
    afkTimeout = 300;           # AFK 超时（秒）
  };
}
```

#### 2. 重建系统

```bash
cd /home/vitus/nixos-config
sudo nixos-rebuild switch --flake .#Vitus5600
```

#### 3. 验证服务状态

```bash
# 检查服务是否运行
systemctl --user status tail

# 查看实时日志
journalctl --user -u tail -f
```

#### 4. 运行 GUI 应用

```bash
# 直接运行
tail-app

# 或从应用菜单启动（如果配置了桌面图标）
```

### 方法二：手动启动（临时测试）

如果您只想临时测试，无需修改系统配置：

```bash
# 启动后台服务
tail-service &

# 运行 GUI 查看数据
tail-app
```

### 方法三：Hyprland 自动启动

如果您使用 Hyprland，可以在配置中添加自动启动：

编辑 `~/.config/hypr/hyprland.conf`：

```bash
# 自动启动 TaiL 服务
exec-once = tail-service
```

然后重新加载 Hyprland 配置或重新登录。

## 🎯 启用后的使用

### 查看服务状态

```bash
# 检查服务是否运行
systemctl --user status tail

# 输出示例：
# ● tail.service - TaiL Window Time Tracker Service
#    Loaded: loaded
#    Active: active (running)
```

### 查看日志

```bash
# 实时查看日志
journalctl --user -u tail -f

# 查看最近的日志
journalctl --user -u tail -n 50
```

### 检查数据库

```bash
# 查看数据库文件
ls -lh ~/.local/share/tail/tail.db

# 查询数据
sqlite3 ~/.local/share/tail/tail.db "SELECT * FROM window_events ORDER BY start_time DESC LIMIT 10;"
```

### 使用 GUI 应用

```bash
# 启动 GUI
tail-app
```

GUI 会显示：
- 窗口使用时间统计
- 应用程序使用时长
- 时间线视图
- AFK 状态

## 🔧 配置选项说明

### services.tail 完整选项

```nix
services.tail = {
  # 是否启用服务（必需）
  enable = true;

  # 运行服务的用户（必需）
  user = "vitus";

  # AFK 超时时间（秒）
  # 默认: 300 (5分钟)
  # 如果超过这个时间没有活动，会标记为 AFK
  afkTimeout = 300;

  # 日志级别
  # 可选: "error", "warn", "info", "debug", "trace"
  # 默认: "info"
  # 建议: 日常使用 "info"，调试时使用 "debug"
  logLevel = "info";

  # 是否自动启动
  # 默认: true
  # true: 登录后自动启动
  # false: 需要手动启动
  autoStart = true;
};
```

### 推荐配置

**日常使用**：
```nix
services.tail = {
  enable = true;
  user = "vitus";
  autoStart = true;
  logLevel = "info";
  afkTimeout = 300;  # 5分钟
};
```

**高灵敏度**（更快检测 AFK）：
```nix
services.tail = {
  enable = true;
  user = "vitus";
  autoStart = true;
  logLevel = "info";
  afkTimeout = 180;  # 3分钟
};
```

**调试模式**：
```nix
services.tail = {
  enable = true;
  user = "vitus";
  autoStart = true;
  logLevel = "debug";  # 详细日志
  afkTimeout = 300;
};
```

## 🚀 启动后的工作流程

### 1. 服务自动运行

启用后，TaiL 服务会：
- ✅ 在您登录 Hyprland 后自动启动
- ✅ 监听窗口切换事件
- ✅ 记录每个窗口的使用时间
- ✅ 检测 AFK 状态
- ✅ 将数据保存到 SQLite 数据库

### 2. 查看统计数据

随时运行 GUI 查看统计：

```bash
tail-app
```

### 3. 数据持久化

所有数据保存在：
```
~/.local/share/tail/tail.db
```

## 🔍 验证安装

### 完整验证清单

```bash
# 1. 检查包是否安装
which tail-app
which tail-service

# 2. 检查服务状态
systemctl --user status tail

# 3. 检查数据库
ls -lh ~/.local/share/tail/tail.db

# 4. 测试 GUI
tail-app

# 5. 查看日志
journalctl --user -u tail -n 20
```

### 预期输出

**服务状态**：
```
● tail.service - TaiL Window Time Tracker Service
   Loaded: loaded (/etc/systemd/user/tail.service)
   Active: active (running) since ...
```

**日志示例**：
```
INFO tail_service::service: Active window changed: code - ...
INFO tail_service::service: Inserted new window event: code (id: 1)
INFO tail_service::service: Updated window event: code used for 10 seconds
```

## ❓ 常见问题

### Q: 服务启动失败

**检查**：
```bash
# 查看详细错误
journalctl --user -u tail -n 50

# 检查是否在 Hyprland 中
echo $HYPRLAND_INSTANCE_SIGNATURE
```

**解决**：
- 确保在 Hyprland 会话中
- 检查用户名是否正确
- 查看日志中的具体错误信息

### Q: GUI 无法启动

**检查**：
```bash
# 检查包是否安装
which tail-app

# 手动运行查看错误
tail-app
```

**解决**：
- 确保已执行 `sudo nixos-rebuild switch`
- 检查是否有 Wayland 环境变量

### Q: 没有记录数据

**检查**：
```bash
# 检查服务是否运行
systemctl --user status tail

# 检查数据库
sqlite3 ~/.local/share/tail/tail.db "SELECT COUNT(*) FROM window_events;"
```

**解决**：
- 确保服务正在运行
- 检查数据库文件权限
- 查看服务日志是否有错误

### Q: 如何停止服务

```bash
# 临时停止
systemctl --user stop tail

# 禁用自动启动
systemctl --user disable tail

# 或在配置中设置
services.tail.autoStart = false;
```

### Q: 如何重启服务

```bash
# 重启服务
systemctl --user restart tail

# 重新加载配置后重启
sudo nixos-rebuild switch --flake .#Vitus5600
systemctl --user restart tail
```

## 📚 更多信息

- 完整安装指南：[NIXOS_INSTALL.md](../NIXOS_INSTALL.md)
- 运行指南：[RUNNING_GUIDE.md](../RUNNING_GUIDE.md)
- 修复说明：[NIXOS_MODULE_FIX.md](./NIXOS_MODULE_FIX.md)

---

**享受使用 TaiL！** 🎉