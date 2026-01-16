# TaiL 故障排查

本文档帮助解决使用 TaiL 时遇到的常见问题。

## 目录

- [安装问题](#安装问题)
- [运行问题](#运行问题)
- [数据问题](#数据问题)
- [性能问题](#性能问题)

---

## 安装问题

### 构建失败

#### 错误：`error: attribute 'tail-service' missing`

**原因：** Overlay 未自动应用到 NixOS 模块中。

**解决方法：**

确保使用 `tail.nixosModules.default`：

```nix
{
  inputs.tail.url = "github:vitus213/tail";

  outputs = { self, nixpkgs, tail, ... }: {
    nixosConfigurations.hostname = nixpkgs.lib.nixosSystem {
      modules = [
        tail.nixosModules.default  # 使用 default 模块
        ./configuration.nix
      ];
    };
  };
}
```

然后重建：

```bash
nix flake update
sudo nixos-rebuild switch --flake .#hostname
```

#### 错误：`xdg.desktopEntries` 不存在

**原因：** `xdg.desktopEntries` 是 Home Manager 专属选项，不能在 NixOS 系统模块中使用。

**解决方法：**

在 Home Manager 的 `home.nix` 中配置桌面图标：

```nix
{ config, pkgs, ... }:
{
  xdg.desktopEntries.tail = {
    name = "TaiL";
    exec = "${pkgs.tail-app}/bin/tail-app";
    icon = "utilities-system-monitor";
    # ...
  };
}
```

---

## 运行问题

### 服务无法启动

#### 检查服务状态

```bash
systemctl --user status tail
```

#### 查看详细日志

```bash
journalctl --user -u tail -n 50 --no-pager
```

#### 常见原因

**1. 不在 Hyprland 会话中**

检查：
```bash
echo $HYPRLAND_INSTANCE_SIGNATURE
```

如果为空，说明不在 Hyprland 环境中。TaiL 必须在 Hyprland 会话中运行。

**2. 用户名配置错误**

确认 `configuration.nix` 中的用户名正确：

```nix
services.tail = {
  user = "yourusername";  # 必须与实际用户名一致
};
```

**3. 数据目录权限问题**

确保数据目录存在且有写权限：

```bash
mkdir -p ~/.local/share/tail
chmod 755 ~/.local/share/tail
```

### GUI 无法启动

#### 检查可执行文件

```bash
which tail-app
```

#### 手动运行查看错误

```bash
tail-app 2>&1
```

#### 常见原因

**1. Wayland 环境变量缺失**

```bash
export WAYLAND_DISPLAY=wayland-0
```

**2. 缺少图形库**

Arch Linux:
```bash
sudo pacman -S wayland libxkbcommon
```

Ubuntu/Debian:
```bash
sudo apt install libwayland-client0 libxkbcommon0
```

**3. 数据库损坏**

删除数据库让服务重建：

```bash
systemctl --user stop tail
rm ~/.local/share/tail/tail.db
systemctl --user start tail
```

### 没有记录数据

#### 检查服务是否运行

```bash
systemctl --user is-active tail
```

#### 检查数据库

```bash
ls -lh ~/.local/share/tail/tail.db
sqlite3 ~/.local/share/tail/tail.db "SELECT COUNT(*) FROM window_events;"
```

#### 查看服务日志

```bash
journalctl --user -u tail -f
```

应该看到类似的日志：
```
INFO tail_service::service: Active window changed: code
INFO tail_service::service: Inserted new window event: code (id: 1)
```

---

## 数据问题

### 数据库损坏

#### 症状

- GUI 无法显示数据
- 服务日志出现数据库错误
- 查询数据库返回错误

#### 解决方法

```bash
# 停止服务
systemctl --user stop tail

# 备份现有数据库
cp ~/.local/share/tail/tail.db ~/.local/share/tail/tail.db.backup

# 尝试修复
sqlite3 ~/.local/share/tail/tail.db "PRAGMA integrity_check;"

# 如果无法修复，删除重建
rm ~/.local/share/tail/tail.db

# 重启服务
systemctl --user start tail
```

### 数据库文件过大

#### 清理旧数据

```bash
sqlite3 ~/.local/share/tail/tail.db "
  DELETE FROM window_events WHERE date(start_time) < date('now', '-30 days');
  DELETE FROM afk_events WHERE date(start_time) < date('now', '-30 days');
  VACUUM;
"
```

#### 定期清理（ systemd timer）

创建 `~/.config/systemd/user/tail-cleanup.service`：

```ini
[Unit]
Description=TaiL Database Cleanup

[Service]
Type=oneshot
ExecStart=/usr/bin/sqlite3 %h/.local/share/tail/tail.db "DELETE FROM window_events WHERE date(start_time) < date('now', '-30 days'); VACUUM;"
```

创建 `~/.config/systemd/user/tail-cleanup.timer`：

```ini
[Unit]
Description=TaiL Database Cleanup Timer

[Timer]
OnCalendar=weekly
Persistent=true

[Install]
WantedBy=timers.target
```

启用：
```bash
systemctl --user enable --now tail-cleanup.timer
```

---

## 性能问题

### 服务占用资源过多

#### 检查资源使用

```bash
# 内存和 CPU
ps aux | grep tail-service

# 打开的文件
lsof -p $(pgrep tail-service)
```

#### 优化方法

1. **降低日志级别**

```nix
services.tail = {
  logLevel = "warn";  # 或 "error"
};
```

2. **定期清理数据库**

参考上面的清理方法。

### GUI 卡顿

#### 可能原因

1. 数据库查询慢 - 数据量太大
2. 数据刷新太频繁 - 默认 10 秒

#### 解决方法

1. 清理旧数据
2. 重启 GUI 应用

---

## 获取帮助

如果以上方法无法解决问题：

### 收集诊断信息

```bash
# 服务状态
systemctl --user status tail > tail-diagnostic.txt

# 最近日志
journalctl --user -u tail -n 100 >> tail-diagnostic.txt

# 数据库信息
sqlite3 ~/.local/share/tail/tail.db "PRAGMA integrity_check;" >> tail-diagnostic.txt

# 系统信息
echo "Hyprland: $HYPRLAND_INSTANCE_SIGNATURE" >> tail-diagnostic.txt
```

### 提交 Issue

在 [GitHub Issues](https://github.com/vitus213/tail/issues) 提交问题时，请附上：

1. 问题的详细描述
2. 复现步骤
3. 诊断信息
4. 系统版本（`nixos-version` 或 `uname -a`）
