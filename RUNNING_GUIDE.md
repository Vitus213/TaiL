# TaiL 运行指南

## 前置要求

1. **Hyprland 窗口管理器**
   - 必须在 Hyprland 环境下运行
   - 确保 `HYPRLAND_INSTANCE_SIGNATURE` 环境变量已设置

2. **Rust 工具链**
   ```bash
   # 检查 Rust 版本（需要 1.84+）
   rustc --version
   ```

3. **系统依赖**
   ```bash
   # Arch Linux
   sudo pacman -S pkg-config wayland libxkbcommon

   # Ubuntu/Debian
   sudo apt install pkg-config libwayland-dev libxkbcommon-dev
   ```

## 快速开始

### 方法一：使用 Nix（推荐）

```bash
# 1. 进入开发环境
nix develop

# 2. 构建项目
nix build .#tail-app
nix build .#tail-service

# 3. 运行 GUI 应用
./result/bin/tail-app

# 或运行后台服务
./result/bin/tail-service
```

### 方法二：使用 Cargo

```bash
# 1. 构建项目
cargo build --release

# 2. 运行 GUI 应用
cargo run --release -p tail-app

# 或运行后台服务
cargo run --release -p tail-service
```

### 方法三：使用 justfile（如果已安装 just）

```bash
# 查看可用命令
just --list

# 构建项目
just build

# 运行 GUI
just run

# 运行服务
just run-service

# 运行测试
just test
```

## 运行模式

### 1. GUI 模式（推荐用于查看统计）

```bash
cargo run --release -p tail-app
```

**功能：**
- 📊 查看今日使用统计
- 📈 查看历史数据（支持多时间范围）
- ⚙️ 设置每日使用目标
- 🎨 自动主题切换

**界面说明：**
- **仪表板**：显示今日应用使用排行和总时长
- **统计**：查看不同时间范围的详细数据
- **设置**：管理每日使用目标

### 2. 后台服务模式（用于持续追踪）

```bash
cargo run --release -p tail-service
```

**功能：**
- 🔍 自动监听 Hyprland 窗口切换
- ⏱️ 实时计算窗口使用时长
- 💾 自动保存到数据库
- 😴 检测 AFK（空闲）状态

**日志级别：**
```bash
# 设置日志级别
RUST_LOG=info cargo run --release -p tail-service
RUST_LOG=debug cargo run --release -p tail-service
```

**后台运行：**
```bash
# 使用 nohup 后台运行
nohup cargo run --release -p tail-service > tail.log 2>&1 &

# 或使用 systemd（推荐）
# 创建 systemd 服务文件（见下文）
```

## 数据存储位置

数据库文件默认存储在：
```
~/.local/share/tail/tail.db
```

包含三个表：
- `window_events` - 窗口使用记录
- `afk_events` - 空闲时段记录
- `daily_goals` - 每日使用目标

## 开发模式

### 运行测试

```bash
# 运行所有测试
cargo test --workspace

# 运行单元测试
cargo test --lib --workspace

# 运行集成测试
cargo test -p tail-tests

# 查看测试输出
cargo test -- --nocapture
```

### 代码检查

```bash
# Clippy 检查
cargo clippy --workspace --all-targets

# 格式化检查
cargo fmt --check

# 自动格式化
cargo fmt
```

### 构建优化

```bash
# Debug 构建（快速编译，用于开发）
cargo build

# Release 构建（优化性能，用于生产）
cargo build --release

# 检查编译（不生成二进制文件）
cargo check
```

## 系统集成

### 创建 systemd 服务（自动启动）

1. 创建服务文件：
```bash
sudo nano /etc/systemd/user/tail-service.service
```

2. 添加以下内容：
```ini
[Unit]
Description=TaiL Window Time Tracker Service
After=graphical-session.target

[Service]
Type=simple
ExecStart=/path/to/tail/target/release/tail-service
Restart=on-failure
Environment="RUST_LOG=info"

[Install]
WantedBy=default.target
```

3. 启用并启动服务：
```bash
# 重新加载 systemd
systemctl --user daemon-reload

# 启用开机自启
systemctl --user enable tail-service

# 启动服务
systemctl --user start tail-service

# 查看状态
systemctl --user status tail-service

# 查看日志
journalctl --user -u tail-service -f
```

### 添加到 Hyprland 自动启动

编辑 `~/.config/hypr/hyprland.conf`：
```bash
# 添加以下行
exec-once = /path/to/tail/target/release/tail-service
```

## 故障排查

### 问题 1：找不到 Hyprland socket

**错误信息：**
```
Socket path not found. Is HYPRLAND_INSTANCE_SIGNATURE set?
```

**解决方法：**
```bash
# 检查环境变量
echo $HYPRLAND_INSTANCE_SIGNATURE

# 如果为空，确保在 Hyprland 会话中运行
# 或手动设置（不推荐）
export HYPRLAND_INSTANCE_SIGNATURE=$(ls /tmp/hypr/)
```

### 问题 2：数据库权限错误

**解决方法：**
```bash
# 确保目录存在且有写权限
mkdir -p ~/.local/share/tail
chmod 755 ~/.local/share/tail
```

### 问题 3：GUI 无法启动

**解决方法：**
```bash
# 检查 Wayland 相关库
ldd target/release/tail-app | grep -i wayland

# 安装缺失的依赖
sudo pacman -S wayland libxkbcommon  # Arch
sudo apt install libwayland-client0 libxkbcommon0  # Ubuntu
```

### 问题 4：编译错误

**解决方法：**
```bash
# 清理构建缓存
cargo clean

# 更新依赖
cargo update

# 重新构建
cargo build --release
```

## 性能优化建议

1. **使用 Release 构建**
   ```bash
   cargo build --release
   ```
   Release 版本比 Debug 版本快 10-100 倍

2. **定期清理旧数据**
   ```sql
   -- 删除 30 天前的数据
   DELETE FROM window_events WHERE timestamp < datetime('now', '-30 days');
   DELETE FROM afk_events WHERE start_time < datetime('now', '-30 days');
   ```

3. **优化数据库**
   ```bash
   sqlite3 ~/.local/share/tail/tail.db "VACUUM;"
   ```

## 使用技巧

1. **查看实时日志**
   ```bash
   RUST_LOG=info cargo run --release -p tail-service 2>&1 | tee tail.log
   ```

2. **导出数据**
   ```bash
   sqlite3 ~/.local/share/tail/tail.db ".mode csv" ".output usage.csv" "SELECT * FROM window_events;"
   ```

3. **备份数据库**
   ```bash
   cp ~/.local/share/tail/tail.db ~/.local/share/tail/tail.db.backup
   ```

## 下一步

- 🎯 设置每日使用目标
- 📊 查看使用统计，了解时间分配
- ⚙️ 根据需要调整 AFK 超时时间
- 🔔 等待通知功能（即将推出）

## 获取帮助

- 查看 [DEVELOPMENT_SUMMARY.md](DEVELOPMENT_SUMMARY.md) 了解架构详情
- 查看 [plans/](plans/) 目录了解设计文档
- 提交 Issue 报告问题或建议功能