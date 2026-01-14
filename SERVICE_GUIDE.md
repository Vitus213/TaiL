# TaiL 服务运行指南

## 方式一：临时运行（用于测试）

### 启动后台服务
```bash
nix develop --command cargo run --package tail-app --bin tail-service
```

### 启动 GUI 应用
```bash
nix develop --command cargo run --package tail-app --bin tail-app
```

**优点**：快速测试
**缺点**：需要手动启动，关闭终端后服务停止

---

## 方式二：使用 systemd 用户服务（推荐）

### 1. 安装服务

#### 方式 A: 使用 NixOS 模块（推荐）
在你的 NixOS 配置中添加：

```nix
{
  # 导入TaiL 模块
  imports = [ /path/to/TaiL/nix/module.nix ];

  # 启用服务
  services.tail = {
    enable = true;
    autoStart = true;  # 开机自动启动
    logLevel = "info"; # 日志级别
  };
}
```

然后重建系统：
```bash
sudo nixos-rebuild switch
```

#### 方式 B: 手动安装服务文件
```bash
# 创建用户服务目录
mkdir -p ~/.config/systemd/user/

# 复制服务文件
cp nix/tail-service.service ~/.config/systemd/user/

# 重新加载 systemd
systemctl --user daemon-reload
```

### 2. 管理服务

#### 启动服务
```bash
systemctl --user start tail.service
```

#### 停止服务
```bash
systemctl --user stop tail.service
```

#### 重启服务
```bash
systemctl --user restart tail.service
```

#### 查看服务状态
```bash
systemctl --user status tail.service
```

#### 查看服务日志
```bash
# 实时查看日志
journalctl --user -u tail.service -f

# 查看最近的日志
journalctl --user -u tail.service -n 100
```

#### 开机自动启动
```bash
systemctl --user enable tail.service
```

#### 禁用开机自动启动
```bash
systemctl --user disable tail.service
```

### 3. 验证服务运行

```bash
# 检查服务状态
systemctl --user is-active tail.service

# 查看进程
ps aux | grep tail-service

# 查看数据库
ls -lh ~/.local/share/tail/tail.db
```

---

## 方式三：使用 justfile 快捷命令

```bash
# 进入开发环境
nix develop

# 运行后台服务
just run-service

# 运行 GUI 应用
just run
```

---

## 数据存储位置

- **数据库**: `~/.local/share/tail/tail.db`
- **日志**:使用 systemd journal（使用 `journalctl` 查看）

---

## 常见问题

### Q: 服务无法启动？
```bash
# 查看详细错误信息
journalctl --user -u tail.service -n 50

# 检查 Hyprland 环境变量
echo $HYPRLAND_INSTANCE_SIGNATURE
```

### Q: 如何查看记录的数据？
启动 GUI 应用查看统计数据，或使用 SQL 客户端直接查询数据库。

### Q: 服务占用太多资源？
服务非常轻量，通常只占用几 MB 内存。如果有问题，检查日志文件大小。

### Q: 如何重置数据？
```bash
# 停止服务
systemctl --user stop tail.service

# 删除数据库
rm ~/.local/share/tail/tail.db

# 重启服务
systemctl --user start tail.service
```

---

## 推荐配置

### 生产环境（日常使用）
- 使用 systemd 用户服务
- 启用开机自动启动
- 设置日志级别为 `info`

### 开发环境
- 使用 `nix develop` 临时运行
- 设置 `RUST_LOG=debug` 查看详细日志
- 使用 `just run-service` 快速测试