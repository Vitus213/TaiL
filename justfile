#!/usr/bin/env just --justfile

# TaiL 开发命令集合

default:
    @just --list

# 进入 Nix 开发环境
dev:
    nix develop

# 构建所有组件
build:
    cargo build --workspace

# 发布构建
build-release:
    cargo build --workspace --release

# 运行 GUI 应用
run:
    cargo run --package tail-app --bin tail-app

# 运行后台服务
run-service:
    cargo run --package tail-app --bin tail-service

# 检查代码
check:
    cargo check --workspace

# 运行测试
test:
    cargo test --workspace

# 格式化代码
fmt:
    cargo fmt

# Lint 代码
clippy:
    cargo clippy --workspace -- -D warnings

# 清理构建产物
clean:
    cargo clean

# Docker 构建测试
docker-build:
    docker build -t tail-dev .

# Docker 运行测试
docker-test: docker-build
    docker run --rm tail-dev nix build .#tail-app

# Nix 构建
nix-build:
    nix build .#tail-app
    nix build .#tail-service

# 一键打包给 NixOS 使用
nix-package:
    @echo "📦 正在构建 TaiL 包..."
    nix build .#tail-app
    nix build .#tail-service
    @echo "✅ 构建完成！"
    @echo ""
    @echo "📍 二进制文件位置:"
    @ls -lh result/bin/
    @echo ""
    @echo "🚀 安装到 NixOS:"
    @echo "  1. 添加到 flake.nix inputs"
    @echo "  2. 在 configuration.nix 中启用: services.tail.enable = true;"
    @echo "  3. 运行: sudo nixos-rebuild switch"
    @echo ""
    @echo "📖 详细说明请查看: NIXOS_INSTALL.md"

# 创建 NixOS 安装包
nix-install-local:
    @echo "📦 安装到本地系统..."
    nix profile install .#tail-app
    nix profile install .#tail-service
    @echo "✅ 安装完成！"
    @echo "运行: tail-app 或 tail-service"

# 更新 Nix flake 输入
nix-update:
    nix flake update

# 检查 Nix flake
nix-check:
    nix flake check
