# TaiL 安装指南

本文档介绍如何在各种 Linux 发行版上安装 TaiL。

## 目录

- [NixOS 安装](#nixos-安装)
- [其他 Linux 发行版](#其他-linux-发行版)
- [从源码构建](#从源码构建)
- [验证安装](#验证安装)

---

## NixOS 安装

TaiL 提供完整的 Nix Flakes 支持，可以轻松集成到 NixOS 系统中。

### 方法一：使用 Flake 输入（推荐）

#### 1. 添加到你的 flake.nix

编辑你的系统 flake 配置（通常在 `/etc/nixos/configuration.nix` 或你的 flake 目录）：

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    # 添加 TaiL
    tail.url = "github:vitus213/tail";
  };

  outputs = { self, nixpkgs, tail, ... }: {
    nixosConfigurations.yourhostname = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        # 导入 TaiL 模块（会自动应用 overlay）
        tail.nixosModules.default
        ./configuration.nix
      ];
    };
  };
}
```
建议添加Cachix，避免在本地构建
```
    trusted-substituters = [
      "https://tail.cachix.org"
    ];

    trusted-public-keys = [
      "tail.cachix.org-1:8wrCmBbcfPfvYdZ3b/bmkcPqs0AukBJug08DIBu19Ao="
    ];
    builders-use-substitutes = true;

```

#### 2. 在 configuration.nix 中启用服务

```nix
{ config, pkgs, ... }:

{
  # 启用 TaiL 服务
  services.tail = {
    enable = true;
    user = "yourusername";        # 替换为你的用户名
    afkTimeout = 300;             # AFK 超时时间（秒）
    logLevel = "info";            # 日志级别: error, warn, info, debug, trace
    autoStart = true;             # 自动启动
  };
}
```

#### 3. 重建系统

```bash
sudo nixos-rebuild switch --flake .#yourhostname
```

### 方法二：直接运行（无需安装）

```bash
# 运行 GUI 应用
nix run github:vitus213/tail#tail-app

# 运行后台服务
nix run github:vitus213/tail#tail-service
```

### 方法三：安装到用户环境

```bash
# 安装到用户 profile
nix profile install github:vitus213/tail#tail-app
nix profile install github:vitus213/tail#tail-service

# 运行
tail-app
tail-service
```

### 方法四：本地构建

```bash
# 克隆仓库
git clone https://github.com/vitus213/tail.git
cd tail

# 构建
nix build .#tail-app
nix build .#tail-service

# 运行
./result/bin/tail-app
```

## NixOS 模块配置选项

### services.tail 完整选项

```nix
services.tail = {
  # 是否启用服务（必需）
  enable = true;

  # 运行服务的用户（必需）
  user = "yourusername";

  # AFK 超时时间（秒）
  # 默认: 300 (5分钟)
  afkTimeout = 300;

  # 日志级别
  # 可选: "error", "warn", "info", "debug", "trace"
  # 默认: "info"
  logLevel = "info";

  # 是否自动启动
  # 默认: true
  autoStart = true;

  # 是否安装 GUI 应用
  # 默认: true
  installGui = true;
};
```

### Home Manager 集成

如果你使用 Home Manager，可以这样配置桌面图标：

```nix
{ config, pkgs, ... }:

{
  # 确保 overlay 已应用
  # 在 flake.nix 中: overlays = [ tail.overlays.default ];

  # 安装 GUI 应用
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

  # 配置 systemd 用户服务（如果不用系统模块）
  systemd.user.services.tail = {
    Unit = {
      Description = "TaiL Window Time Tracker";
      After = [ "graphical-session.target" ];
    };

    Service = {
      Type = "simple";
      ExecStart = "${pkgs.tail-service}/bin/tail-service";
      Restart = "on-failure";
      Environment = [ "RUST_LOG=info" ];
    };

    Install = {
      WantedBy = [ "graphical-session.target" ];
    };
  };
}
```

### 应用 Home Manager 配置

```bash
home-manager switch --flake .#yourusername
```

---

## 其他 Linux 发行版

### Arch Linux

```bash
# 安装依赖
sudo pacman -S rust pkg-config wayland libxkbcommon

# 使用 Cargo 安装
cargo install tail

# 或从源码构建
git clone https://github.com/vitus213/tail.git
cd tail
cargo build --release
sudo install target/release/tail-app /usr/local/bin/
sudo install target/release/tail-service /usr/local/bin/
```

### Ubuntu/Debian

```bash
# 安装依赖
sudo apt update
sudo apt install rustc cargo pkg-config libwayland-dev libxkbcommon-dev

# 使用 Cargo 安装
cargo install tail

# 或从源码构建
git clone https://github.com/vitus213/tail.git
cd tail
cargo build --release
sudo install target/release/tail-app /usr/local/bin/
sudo install target/release/tail-service /usr/local/bin/
```

### 使用 Nix（非 NixOS）

如果你使用其他发行版但想用 Nix 包管理器：

```bash
# 安装 Nix
curl --proto '=https' --tlsv1.2 -sSf -L https://nixos.org/nix/install | sh

# 启用 Flakes
mkdir -p ~/.config/nix
echo "experimental-features = nix-command flakes" >> ~/.config/nix/nix.conf

# 运行 TaiL
nix run github:vitus213/tail
```

---

## 从源码构建

### 1. 克隆仓库

```bash
git clone https://github.com/vitus213/tail.git
cd tail
```

### 2. 安装依赖

**Arch Linux:**
```bash
sudo pacman -S rust pkg-config wayland libxkbcommon
```

**Ubuntu/Debian:**
```bash
sudo apt install rustc cargo pkg-config libwayland-dev libxkbcommon-dev
```

### 3. 构建

```bash
# Debug 构建
cargo build

# Release 构建（推荐）
cargo build --release
```

### 4. 运行

```bash
# 运行 GUI
cargo run --release -p tail-app

# 运行服务
cargo run --release -p tail-service
```

---

## 验证安装

### 检查二进制文件

```bash
which tail-app
which tail-service
```

### 检查服务状态（NixOS）

```bash
systemctl --user status tail
```

### 运行 GUI

```bash
tail-app
```

你应该能看到 TaiL 的主界面，显示今日使用统计。

### 检查数据库

```bash
ls -lh ~/.local/share/tail/tail.db
```

---

## 卸载

### NixOS 系统级卸载

在 `configuration.nix` 中禁用服务：

```nix
services.tail.enable = false;
```

然后重建：

```bash
sudo nixos-rebuild switch
```

### 用户环境卸载

```bash
nix profile remove tail-app
nix profile remove tail-service
```

### 清理数据

```bash
# 删除数据库
rm -rf ~/.local/share/tail
```

---

## 常见问题

### 构建时出现 `error: attribute 'tail-service' missing`

确保你使用的是 `tail.nixosModules.default` 而不是手动导入 `nix/module.nix`。

```bash
nix flake update
sudo nixos-rebuild switch --flake .#yourhostname
```

### 服务无法启动

检查你是否在 Hyprland 会话中：

```bash
echo $HYPRLAND_INSTANCE_SIGNATURE
```

如果为空，说明不在 Hyprland 环境中。

### 找不到 tail-app 命令

```bash
# 检查包是否在 PATH 中
which tail-app

# 或直接运行
nix run .#tail-app
```

更多问题请查看 [故障排查文档](troubleshooting.md)。
