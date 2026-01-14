# TaiL NixOS 安装指南

## 🚀 一键安装到NixOS

TaiL 提供了完整的 Nix Flakes 支持，可以轻松集成到您的 NixOS 系统中。

## ⚠️ 重要提示

**TaiL 的 NixOS 模块会自动应用 overlay**，无需手动配置 `nixpkgs.overlays`。模块导入后，`pkgs.tail-service` 和 `pkgs.tail-app` 会自动可用。

**桌面图标说明**：`xdg.desktopEntries` 只能在 Home Manager 中使用，不能在 NixOS 系统模块中使用。如需桌面图标，请参考 [Home Manager 集成](#home-manager-集成) 部分。

## 方法一：使用 Flake 输入（推荐）

### 1. 添加到您的 flake.nix

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    
    # 添加 TaiL
    tail.url = "github:yourusername/TaiL";  # 替换为实际仓库地址
    # 或使用本地路径
    # tail.url = "path:/path/to/TaiL";
  };

  outputs = { self, nixpkgs, tail, ... }: {
    nixosConfigurations.yourhostname = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        # 导入 TaiL 模块（会自动应用 overlay）
        tail.nixosModules.default
        # 您的配置
        ./configuration.nix
      ];
    };
  };
}
```

### 2. 在 configuration.nix 中启用服务

```nix
{ config, pkgs, ... }:

{
  # 启用 TaiL 服务
  services.tail = {
    enable = true;
    user = "yourusername";  # 替换为您的用户名
    afkTimeout = 300;  # AFK 超时时间（秒）
    logLevel = "info"; # 日志级别: error, warn, info, debug, trace
    autoStart = true;  # 自动启动
  };

  # GUI 应用已通过 services.tail.enable 自动添加到系统包
  # 如果您想手动添加其他组件：
  # environment.systemPackages = with pkgs; [
  #   tail-app# 由overlay 提供，无需 tail.packages 前缀
  #   tail-service  # 由 overlay 提供
  # ];
}
```

**说明**：启用 `services.tail.enable = true` 后，`tail-service` 会自动添加到 `environment.systemPackages`，GUI 应用也会被包含。

### 3. 重建系统

```bash
sudo nixos-rebuild switch --flake .#yourhostname
```

## 方法二：直接运行（无需安装）

### 运行 GUI 应用

```bash
nix run github:yourusername/TaiL
# 或本地
nix run .#tail-app
```

### 运行后台服务

```bash
nix run .#tail-service
```

## 方法三：临时安装到用户环境

```bash
# 安装到用户环境
nix profile install github:yourusername/TaiL

# 或从本地
nix profile install .#tail-app
nix profile install .#tail-service

# 运行
tail-app
tail-service
```

## 方法四：本地构建安装

```bash
# 克隆仓库
git clone https://github.com/yourusername/TaiL.git
cd TaiL

# 构建
nix build .#tail-app
nix build .#tail-service

# 安装到系统
sudo cp result/bin/tail-app /usr/local/bin/
sudo cp result/bin/tail-service /usr/local/bin/
```

## 配置选项详解

### services.tail 可用选项

```nix
services.tail = {
  # 是否启用服务
  enable = true;

  # 运行服务的用户
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

  # 自定义包（高级用法）
  # package = pkgs.tail-service;      # 后台服务包
  # guiPackage = pkgs.tail-app;       # GUI 应用包
};
```

## Home Manager 集成

如果您使用 Home Manager，可以这样配置：

### 1. 在 flake.nix 中配置 Home Manager

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    home-manager.url = "github:nix-community/home-manager";
    home-manager.inputs.nixpkgs.follows = "nixpkgs";tail.url = "github:Vitus213/TaiL";
  };

  outputs = { self, nixpkgs, home-manager, tail, ... }: {
    homeConfigurations.yourusername = home-manager.lib.homeManagerConfiguration {
      pkgs = import nixpkgs {
        system = "x86_64-linux";
        overlays = [ tail.overlays.default ];  # 应用 TaiL overlay
      };
      modules = [ ./home.nix ];
    };
  };
}
```

### 2. 添加到 home.nix

```nix
{ config, pkgs, ... }:

{
  # 安装 GUI 应用（overlay 已应用，直接使用）
  home.packages = [
    pkgs.tail-app
  ];

  # 配置桌面图标（仅 Home Manager 支持）
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

  # 配置 systemd 用户服务
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
        "RUST_LOG=info""RUST_BACKTRACE=1"
      ];
    };

    Install = {
      WantedBy = [ "graphical-session.target" ];
    };
  };
}
```

**注意**：`xdg.desktopEntries` 只能在 Home Manager 中使用，不能在 NixOS 系统模块中使用。

### 2. 应用配置

```bash
home-manager switch --flake .#yourusername
```

## Hyprland 集成

### 在Hyprland 配置中自动启动

编辑 `~/.config/hypr/hyprland.conf`:

```bash
# 自动启动 TaiL 服务
exec-once = tail-service

# 或使用完整路径
exec-once = /run/current-system/sw/bin/tail-service
```

## 使用 Overlay（高级）

### NixOS 配置中手动应用 Overlay

如果您**不使用** `tail.nixosModules.default`，而是想手动应用 overlay：

```nix
{
  # 手动应用 overlay
  nixpkgs.overlays = [
    tail.overlays.default
  ];

  # 现在可以使用 pkgs.tail-app 和 pkgs.tail-service
  environment.systemPackages = with pkgs; [
    tail-app
    tail-service
  ];
}
```

**重要**：如果您已经使用了 `tail.nixosModules.default`，则**无需**手动配置 overlay，因为模块会自动应用。

### 在其他 Flake 中使用

```nix
{
  inputs.tail.url = "github:Vitus213/TaiL";

  outputs = { self, nixpkgs, tail, ... }: {
    packages = nixpkgs.lib.genAttrs [ "x86_64-linux" ] (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ tail.overlays.default ];
        };
      in {
        # 现在可以使用 tail包
        myApp = pkgs.stdenv.mkDerivation {
          buildInputs = [ pkgs.tail-service ];
          # ...
        };
      }
    );
  };
}
```

## 验证安装

### 检查服务状态

```bash
# 检查 systemd 服务
systemctl --user status tail

# 查看日志
journalctl --user -u tail -f
```

### 运行 GUI

```bash
tail-app
```

### 检查数据库

```bash
# 数据库位置
ls -lh ~/.local/share/tail/tail.db

# 查看数据
sqlite3 ~/.local/share/tail/tail.db "SELECT * FROM window_events LIMIT 10;"
```

## 卸载

### NixOS 系统级卸载

在 `configuration.nix` 中删除或禁用：

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

### Home Manager 卸载

在 `home.nix` 中删除相关配置，然后：

```bash
home-manager switch
```

### 清理数据

```bash
# 删除数据库
rm -rf ~/.local/share/tail
```

## 更新

### NixOS Flake 更新

```bash
# 更新 flake输入
nix flake update

# 重建系统
sudo nixos-rebuild switch --flake .#yourhostname
```

### 用户环境更新

```bash
nix profile upgrade tail-app
nix profile upgrade tail-service
```

## 开发者模式

### 进入开发环境

```bash
nix develop

# 或使用 direnv
direnv allow
```

### 本地构建测试

```bash
# 构建所有包
nix build .#tail-app
nix build .#tail-service

# 运行测试
nix develop --command cargo test --workspace

# 格式化代码
nix fmt
```

## 常见问题

### Q: 构建时出现`error: attribute 'tail-service' missing`

**A**: 这个错误已在最新版本中修复。请确保：

1. 您使用的是最新版本的 TaiL flake
2. 使用 `tail.nixosModules.default` 而不是手动导入 `nix/module.nix`
3. 如果问题仍然存在，尝试：
   ```bash
   nix flake update
   sudo nixos-rebuild switch --flake .#yourhostname
   ```

**技术细节**：之前的版本中，overlay 没有自动应用到 NixOS 模块中。现在 `tail.nixosModules.default` 会自动应用 overlay，使`pkgs.tail-service` 可用。

### Q: Home Manager 中出现 `xdg.desktopEntries` 错误

**A**: `xdg.desktopEntries` 是 Home Manager 的特性，需要：

1. 确保使用 Home Manager（不是纯 NixOS 配置）
2. 在 Home Manager 的 `home.nix` 中配置，而不是 `configuration.nix`
3. 参考本文档的 [Home Manager 集成](#home-manager-集成) 部分

**注意**：NixOS 系统模块中不能使用 `xdg.desktopEntries`。如果您想要桌面图标，必须使用 Home Manager。

### Q: Overlay 没有生效，找不到 `pkgs.tail-service`

**A**: 如果您使用 `tail.nixosModules.default`，overlay 会自动应用，无需手动配置。如果仍然有问题：

1. 确认您导入的是`tail.nixosModules.default` 而不是手动导入 `./nix/module.nix`
2. 检查 flake inputs是否正确
3. 尝试重新锁定 flake：
   ```bash
   nix flake lock --update-input tail
   sudo nixos-rebuild switch --flake .#yourhostname
   ```

### Q: 服务无法启动

**A**: 检查您是否在Hyprland 会话中：

```bash
echo $HYPRLAND_INSTANCE_SIGNATURE
```

如果为空，说明不在 Hyprland 环境中。

### Q: 找不到 tail-app 命令

**A**: 确保您已正确安装：

```bash
# 检查包是否在 PATH 中
which tail-app

# 或直接运行
nix run .#tail-app
```

### Q: 数据库权限错误

**A**: 确保数据目录存在且有写权限：

```bash
mkdir -p ~/.local/share/tail
chmod 755 ~/.local/share/tail
```

### Q: 如何查看服务日志

**A**: 使用 journalctl：

```bash
journalctl --user -u tail -f
```

## 高级配置

### 自定义数据库位置

编辑服务配置，添加环境变量：

```nix
systemd.user.services.tail = {
  serviceConfig = {
    Environment = [
      "TAIL_DB_PATH=/custom/path/tail.db"
    ];
  };
};
```

### 性能调优

```nix
services.tail = {
  enable = true;
  afkTimeout = 180;  # 降低 AFK 超时以提高灵敏度
  logLevel = "warn"; # 降低日志级别以提升性能
};
```

##贡献

欢迎提交 Issue 和 Pull Request！

## 许可证

MIT License

---

**享受使用 TaiL！** 🎉

如果有任何问题，请查看 [RUNNING_GUIDE.md](RUNNING_GUIDE.md) 或提交 Issue。