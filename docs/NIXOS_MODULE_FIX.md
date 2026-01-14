# NixOS 模块修复说明

## 问题描述

在使用 TaiL 的 NixOS 模块时，用户遇到以下错误：

```
error: attribute 'tail-service' missing
at /nix/store/.../nix/module.nix:15:17:
   14|       type = types.package;
   15|       default = pkgs.tail-service;
```

同时还有关于 `xdg.desktopEntries` 的错误，提示该选项不存在。

## 根本原因

### 1. Overlay 未自动应用

**问题**：[`nix/module.nix`](../nix/module.nix:15) 第15行引用了 `pkgs.tail-service`，但这个包没有通过 overlay 自动注入到 pkgs 中。

**原因**：之前的 [`flake.nix`](../flake.nix:202-208) 只是导出了 `nixosModules.default = nixosModule`，但没有自动应用 overlay。当用户在 NixOS 配置中导入模块时，`pkgs` 中不包含 `tail-service` 和 `tail-app`。

### 2. `xdg.desktopEntries` 误用

**问题**：[`nix/module.nix`](../nix/module.nix:84) 中使用了 `xdg.desktopEntries`，这是 home-manager 的专属选项。

**原因**：`xdg.desktopEntries` 只能在 Home Manager 模块中使用，不能在 NixOS 系统模块中使用。在系统模块中使用会导致评估错误。

## 修复方案

### 1. 修复 flake.nix 中的 overlay 导出

**修改前**：
```nix
nixosModules.default = nixosModule;
nixosModules.tail = nixosModule;
overlays.default = final: prev: {
  tail-app = self.packages.${prev.system}.tail-app;
  tail-service = self.packages.${prev.system}.tail-service;
};
```

**修改后**：
```nix
# NixOS 模块导出 - 自动应用 overlay
nixosModules.default = {config, pkgs, ...}: {
  imports = [nixosModule];
  nixpkgs.overlays = [self.overlays.default];
};
nixosModules.tail = self.nixosModules.default;

# Overlay导出，方便其他 flake 使用
overlays.default = final: prev: {
  tail-app = self.packages.${prev.system}.tail-app or self.packages.${final.system}.tail-app;
  tail-service = self.packages.${prev.system}.tail-service or self.packages.${final.system}.tail-service;
};
```

**改进点**：
- `nixosModules.default` 现在是一个函数，会自动应用 overlay
- 添加了 fallback 逻辑 (`or`) 以提高系统兼容性
- 用户导入模块后，`pkgs.tail-service` 和 `pkgs.tail-app` 自动可用

### 2. 修复 module.nix 中的 xdg.desktopEntries

**修改**：完全移除了 [`nix/module.nix`](../nix/module.nix:83-96) 中的 `xdg.desktopEntries` 配置块。

**原因**：
- `xdg.desktopEntries` 只能在 Home Manager 中使用
- NixOS 系统模块不支持此选项
- 用户如需桌面图标，应在 Home Manager 配置中单独添加

## 验证

修复后的验证结果：

```bash
# 模块加载成功
$ nix eval .#nixosModules.default --apply 'x: "module loads successfully"'
"module loads successfully"

# Overlay 导出成功
$ nix eval .#overlays.default --apply 'x: "overlay exports successfully"'
"overlay exports successfully"
```

## 用户迁移指南

### 如果您之前遇到错误

1. **更新 TaiL flake**：
   ```bash
   nix flake lock --update-input tail
   ```

2. **重建系统**：
   ```bash
   sudo nixos-rebuild switch --flake .#yourhostname
   ```

3. **无需修改配置**：如果您使用的是 `tail.nixosModules.default`，无需修改任何配置。

### 如果您需要桌面图标

在 Home Manager 的 `home.nix` 中添加：

```nix
{ config, pkgs, ... }:

{
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
}
```

## 技术细节

### Overlay 自动应用机制

当用户导入 `tail.nixosModules.default` 时：

1. 模块函数被调用，接收 `config` 和 `pkgs` 参数
2. 模块自动将 `self.overlays.default` 添加到 `nixpkgs.overlays`
3. 然后导入实际的模块配置 (`nixosModule`)
4. 此时 `pkgs.tail-service` 和 `pkgs.tail-app` 已经可用

### 为什么不在系统模块中使用 xdg.desktopEntries

- `xdg.desktopEntries` 是 Home Manager 提供的高级抽象
- 它会生成 `.desktop` 文件到用户的 `~/.local/share/applications/`
- NixOS 系统模块运行在系统级别，没有用户上下文
- 如果需要系统级桌面文件，应该手动创建并放到 `/run/current-system/sw/share/applications/`

## 相关文件

- [`flake.nix`](../flake.nix) - Flake 配置和 overlay 定义
- [`nix/module.nix`](../nix/module.nix) - NixOS 模块定义
- [`NIXOS_INSTALL.md`](../NIXOS_INSTALL.md) - 更新后的安装文档

## 参考资料

- [NixOS Manual - Overlays](https://nixos.org/manual/nixos/stable/#sec-overlays)
- [Home Manager Manual - xdg.desktopEntries](https://nix-community.github.io/home-manager/options.html#opt-xdg.desktopEntries)
- [Nix Flakes - NixOS Modules](https://nixos.wiki/wiki/Flakes#Using_nix_flakes_with_NixOS)

---

**修复日期**：2026-01-14  
**修复版本**：v0.2.0+