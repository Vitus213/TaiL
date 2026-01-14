#!/usr/bin/env bash
# TaiL 一键安装脚本 for NixOS

set -e

BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  TaiL - Window Time Tracker Installer ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
echo ""

# 检查是否在 NixOS 上
if [ ! -f /etc/NIXOS ]; then
    echo -e "${YELLOW}⚠️  警告: 此脚本设计用于 NixOS${NC}"
    echo -e "${YELLOW}   如果您在其他发行版上，请使用 Nix 包管理器${NC}"
    echo ""
fi

# 检查 Nix Flakes 是否启用
if ! nix flake --help &> /dev/null; then
    echo -e "${RED}❌ Nix Flakes 未启用${NC}"
    echo -e "${YELLOW}正在启用 Flakes...${NC}"
    mkdir -p ~/.config/nix
    echo "experimental-features = nix-command flakes" >> ~/.config/nix/nix.conf
    echo -e "${GREEN}✅ Flakes 已启用${NC}"
fi

echo -e "${BLUE}请选择安装方式:${NC}"
echo "  1) 直接运行（无需安装）"
echo "  2) 安装到用户环境"
echo "  3)仅构建（不安装）"
echo "  4) 系统级安装（需要修改 configuration.nix）"
echo ""
read -p "请输入选项(1-4): " choice

case $choice in
    1)
        echo -e "${BLUE}🚀 正在启动 TaiL...${NC}"
        nix run .#tail-app
        ;;
    2)
        echo -e "${BLUE}📦 正在安装到用户环境...${NC}"
        nix profile install .#tail-app
        nix profile install .#tail-service
        echo -e "${GREEN}✅ 安装完成！${NC}"
        echo ""
        echo -e "${GREEN}运行命令:${NC}"
        echo "  tail-app      - 启动 GUI"
        echo "  tail-service  - 启动后台服务"
        ;;
    3)
        echo -e "${BLUE}🔨 正在构建...${NC}"
        nix build .#tail-app
        nix build .#tail-service
        echo -e "${GREEN}✅ 构建完成！${NC}"
        echo ""
        echo -e "${GREEN}二进制文件位置:${NC}"
        ls -lh result/bin/
        echo ""
        echo -e "${BLUE}运行:${NC}"
        echo "  ./result/bin/tail-app"
        echo "  ./result/bin/tail-service"
        ;;
    4)
        echo -e "${BLUE}📝 系统级安装说明${NC}"
        echo ""
        echo "请按以下步骤操作："
        echo ""
        echo "1️⃣  编辑您的 flake.nix，添加 TaiL 输入:"
        echo ""
        cat << 'EOF'
{
  inputs = {
    tail.url = "github:yourusername/TaiL";
    # 或使用本地路径
    # tail.url = "path:${PWD}";
  };
  
  outputs = { tail, ... }: {
    nixosConfigurations.yourhostname = {
      modules = [
        tail.nixosModules.default# ...
      ];
    };
  };
}
EOF
        echo ""
        echo "2️⃣  编辑 configuration.nix，启用服务:"
        echo ""
        cat << 'EOF'
services.tail = {
  enable = true;
  user = "yourusername";  # 替换为您的用户名
  afkTimeout = 300;
  logLevel = "info";
};
EOF
        echo ""
        echo "3️⃣  重建系统:"
        echo ""
        echo "  sudo nixos-rebuild switch --flake .#yourhostname"
        echo ""
        echo -e "${YELLOW}详细说明请查看: NIXOS_INSTALL.md${NC}"
        ;;
    *)
        echo -e "${RED}❌ 无效的选项${NC}"
        exit 1
        ;;
esac

echo ""
echo -e "${GREEN}╔════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║         安装/运行完成！🎉             ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}📖 更多信息:${NC}"
echo "  -运行指南: RUNNING_GUIDE.md"
echo "  - NixOS 安装: NIXOS_INSTALL.md"
echo "  - 开发总结: DEVELOPMENT_SUMMARY.md"
echo ""
echo -e "${BLUE}💡 提示:${NC}"
echo "  - 数据库位置: ~/.local/share/tail/tail.db"
echo "  - 查看日志: journalctl --user -u tail -f"
echo "  - 运行 GUI: tail-app"
echo "  - 后台服务: tail-service"
echo ""