# TaiL NixOS 安装指南

## 🚀 一键安装到NixOS

TaiL 提供了完整的 Nix Flakes 支持，可以轻松集成到您的 NixOS 系统中。

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
        # 导入 TaiL 模块
        tail.nixosModules.default
        # 您的配置
        ./configuration.nix
      ];
    };
  };
}
```

### 2. 在configuration.nix 中启用服务

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

  # （可选）将 tail-app 添加到系统包
  environment.systemPackages = with pkgs; [
    tail.packages.${system}.tail-app
  ];
}
```

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
  #默认: 300 (5分钟)
  afkTimeout = 300;

  # 日志级别
  # 可选: "error", "warn", "info", "debug", "trace"
  # 默认: "info"
  logLevel = "info";

  # 是否自动启动
  # 默认: true
  autoStart = true;

  # 自定义包（高级用法）
  # package = pkgs.tail-service;
};
```

## Home Manager 集成

如果您使用 Home Manager，可以这样配置：

### 1. 添加到 home.nix

```nix
{ config, pkgs, tail, ... }:

{
  # 安装 GUI 应用
  home.packages = [
    tail.packages.${pkgs.system}.tail-app
  ];

  # 配置 systemd 用户服务
  systemd.user.services.tail = {
    Unit = {
      Description = "TaiL Window Time Tracker";
      After = [ "graphical-session.target" ];};

    Service = {
      Type = "simple";
      ExecStart = "${tail.packages.${pkgs.system}.tail-service}/bin/tail-service";
      Restart = "on-failure";
      Environment = [
        "RUST_LOG=info"
      ];
    };

    Install = {
      WantedBy = [ "graphical-session.target" ];
    };
  };
}
```

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

## 使用 Overlay

如果您想在其他地方使用 TaiL 包：

```nix
{
  nixpkgs.overlays = [
    tail.overlays.default
  ];

  environment.systemPackages = with pkgs; [
    tail-app
    tail-service
  ];
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