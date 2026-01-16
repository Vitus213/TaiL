# TaiL 配置说明

本文档介绍 TaiL 的各种配置选项。

## 目录

- [NixOS 模块配置](#nixos-模块配置)
- [Home Manager 配置](#home-manager-配置)
- [环境变量](#环境变量)
- [数据目录](#数据目录)

---

## NixOS 模块配置

### 基本配置

在 `configuration.nix` 中：

```nix
{ config, pkgs, ... }:

{
  services.tail = {
    enable = true;
    user = "yourusername";
  };
}
```

### 完整配置选项

```nix
services.tail = {
  # 是否启用服务（必需）
  enable = true;

  # 运行服务的用户（必需）
  user = "yourusername";

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

  # 是否安装 GUI 应用
  # 默认: true
  installGui = true;
};
```

### 推荐配置

#### 日常使用

```nix
services.tail = {
  enable = true;
  user = "yourusername";
  autoStart = true;
  logLevel = "info";
  afkTimeout = 300;  # 5分钟
};
```

#### 高灵敏度（更快检测 AFK）

```nix
services.tail = {
  enable = true;
  user = "yourusername";
  autoStart = true;
  logLevel = "info";
  afkTimeout = 180;  # 3分钟
};
```

#### 调试模式

```nix
services.tail = {
  enable = true;
  user = "yourusername";
  autoStart = true;
  logLevel = "debug";  # 详细日志
  afkTimeout = 300;
};
```

---

## Home Manager 配置

### 桌面图标

`xdg.desktopEntries` 只能在 Home Manager 中使用：

```nix
{ config, pkgs, ... }:

{
  # 确保 overlay 已应用（在 flake.nix 中配置）
  home.packages = [
    pkgs.tail-app
  ];

  # 配置桌面图标
  xdg.desktopEntries.tail = {
    name = "TaiL";
    genericName = "Window Time Tracker";
    comment = "Track window usage time on Hyprland/Wayland";
    exec = "${pkgs.tail-app}/bin/tail-app";
    icon = "utilities-system-monitor";
    terminal = false;
    type = "Application";
    categories = [ "Utility" "System" "Monitor" ];
    keywords = [ "time" "tracker" "window" "hyprland" "wayland" ];
  };
}
```

### systemd 用户服务

如果不使用 NixOS 模块，可以在 Home Manager 中配置服务：

```nix
{ config, pkgs, ... }:

{
  systemd.user.services.tail = {
    Unit = {
      Description = "TaiL Window Time Tracker";
      After = [ "graphical-session.target" ];
    };

    Service = {
      Type = "simple";
      ExecStart = "${pkgs.tail-service}/bin/tail-service";
      Restart = "on-failure";
      Environment = [
        "RUST_LOG=info"
        "RUST_BACKTRACE=1"
      ];
    };

    Install = {
      WantedBy = [ "graphical-session.target" ];
    };
  };
}
```

### 自定义环境变量

```nix
{ config, pkgs, ... }:

{
  systemd.user.services.tail = {
    Service = {
      Environment = [
        "RUST_LOG=debug"
        "TAIL_DB_PATH=/custom/path/tail.db"
      ];
    };
  };
}
```

---

## 环境变量

### RUST_LOG

控制日志输出级别：

| 值 | 描述 |
|----|------|
| `error` | 只显示错误 |
| `warn` | 显示警告和错误 |
| `info` | 显示信息（默认） |
| `debug` | 显示调试信息 |
| `trace` | 显示详细追踪 |

```bash
# 在 systemd 服务中配置
Environment = "RUST_LOG=debug"

# 或在命令行中
RUST_LOG=debug tail-service
```

### TAIL_DB_PATH

自定义数据库路径：

```bash
export TAIL_DB_PATH=/custom/path/tail.db
```

在 systemd 中配置：

```nix
Service = {
  Environment = "TAIL_DB_PATH=/custom/path/tail.db";
};
```

---

## 数据目录

### 默认位置

```
~/.local/share/tail/
├── tail.db          # 数据库文件
└── tail.db.backup   # 备份文件（如果存在）
```

### XDG 数据目录

TaiL 遵循 [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html)：

| 类型 | 默认位置 | 可通过环境变量覆盖 |
|------|----------|-------------------|
| 数据 | `~/.local/share/tail/` | `XDG_DATA_HOME` |
| 配置 | `~/.config/tail/` | `XDG_CONFIG_HOME` |

### 修改数据目录

#### 方法一：环境变量

```bash
export XDG_DATA_HOME=/custom/data/path
```

#### 方法二：systemd 服务配置

```nix
Service = {
  Environment = "XDG_DATA_HOME=/custom/data/path";
};
```

---

## Hyprland 集成

### 自动启动

编辑 `~/.config/hypr/hyprland.conf`：

```bash
# 自动启动 TaiL 服务
exec-once = tail-service

# 或使用完整路径
exec-once = /run/current-system/sw/bin/tail-service
```

### 绑定快捷键

```bash
# 打开 TaiL GUI
bind = SUPER_SHIFT, T, exec, tail-app
```

---

## 配置文件

TaiL 未来将支持配置文件（计划中）。

### 计划中的配置文件格式（TOML）

```toml
# ~/.config/tail/tail.toml

[general]
log_level = "info"
afk_timeout = 300

[database]
path = "~/.local/share/tail/tail.db"
auto_vacuum = true

[ui]
theme = "dark"
refresh_interval = 10

[notification]
enabled = true
sound = false
```

---

## 验证配置

### 检查服务状态

```bash
systemctl --user status tail
```

### 查看服务配置

```bash
systemctl --user show tail
```

### 查看日志

```bash
journalctl --user -u tail -n 50
```

---

## 故障排查

### 配置不生效

1. 重建系统：
   ```bash
   sudo nixos-rebuild switch
   ```

2. 重启服务：
   ```bash
   systemctl --user restart tail
   ```

3. 查看日志：
   ```bash
   journalctl --user -u tail -n 50
   ```

更多问题请查看 [故障排查文档](troubleshooting.md)。
