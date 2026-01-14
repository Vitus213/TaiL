{
  description = "TaiL - Window time tracker for Hyprland/Wayland";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    crane,
    fenix,
    ...
  }: let
    # NixOS 模块导入
    nixosModule = import ./nix/module.nix;
  in
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
        };

        # 使用 fenix 提供的 Rust 工具链
        rustToolchain = fenix.packages.${system}.stable.toolchain;

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        # Common build inputs for the crate
        commonArgs = {
          src = craneLib.cleanCargoSource ./.;

          buildInputs =
            []
            ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
              # GUI libraries for egui
              pkgs.libxkbcommon
              pkgs.wayland
              # OpenGL/EGL support for glow renderer
              pkgs.libGL
              pkgs.mesa
              # Font support
              pkgs.fontconfig
            ];

          nativeBuildInputs =
            [
              pkgs.pkg-config
            ]
            ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
              pkgs.wayland-scanner
            ];

          # Disable tests for now (they need Hyprland running)
          doCheck = false;
        };

        # Cargo Artifacts
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        # Binary build
        tail-app = craneLib.buildPackage (commonArgs
          // {
            inherit cargoArtifacts;
            cargoExtraArgs = "--package tail-app --bin tail-app";
          });

        tail-service = craneLib.buildPackage (commonArgs
          // {
            inherit cargoArtifacts;
            cargoExtraArgs = "--package tail-app --bin tail-service";
          });

      in {
        # Development environment
        devShells.default = pkgs.mkShell {
          buildInputs = [
            # Fenix Rust 工具链（包含完整工具）
            (fenix.packages.${system}.stable.withComponents [
              "cargo"
              "clippy"
              "rust-src"
              "rust-std"
              "rustc"
              "rustfmt"
            ])
            fenix.packages.${system}.rust-analyzer
          ] ++ (with pkgs; [
            # Build dependencies
            pkg-config

            # GUI libraries (for egui)
            libxkbcommon
            wayland
            wayland-scanner
            # OpenGL/EGL support
            libGL
            mesa

            # Development tools
            cargo-edit
            cargo-watch
            cargo-nextest
            bacon

            # Nix related
            nil
            nixpkgs-fmt

            # For testing IPC (can use socat to test socket)
            socat
            just
          ]);

          shellHook = ''
            # Set up environment for Wayland development
            export WAYLAND_DISPLAY="''${WAYLAND_DISPLAY:-}"
            export XDG_RUNTIME_DIR="''${XDG_RUNTIME_DIR:-/tmp}"

            # 设置动态库路径，解决 winit 运行时加载 Wayland 和 xkbcommon 库的问题
            export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath [
              pkgs.wayland
              pkgs.libxkbcommon
              pkgs.libGL
              pkgs.mesa
            ]}:$LD_LIBRARY_PATH"

            # 设置 fontconfig 路径，确保字体正确加载
            export FONTCONFIG_FILE="${pkgs.fontconfig.out}/etc/fonts/fonts.conf"
            export FONTCONFIG_PATH="${pkgs.fontconfig.out}/etc/fonts"

            # 设置 Rust 日志级别
            export RUST_LOG=info
            export RUST_BACKTRACE=1

            echo "════════════════════════════════════════════"
            echo "🦀 TaiL Development Environment (Fenix)"
            echo "════════════════════════════════════════════"
            echo ""
            echo "📦 Rust version: $(rustc --version)"
            echo "📦 Cargo version: $(cargo --version)"
            echo ""
            echo "🚀 Quick Start:"
            echo "  just-查看所有命令"
            echo "  just run                  - 运行 GUI 应用"
            echo "  just run-service          - 运行后台服务"
            echo "  just test                 - 运行测试"
            echo ""
            echo "🔨 Build Commands:"
            echo "  cargo build --workspace- 构建所有包"
            echo "  cargo build --release     - 发布构建"
            echo "  nix build .#tail-app      - Nix 构建 GUI"
            echo "  nix build .#tail-service  - Nix 构建服务"
            echo ""
            echo "🧪 Test Commands:"
            echo "  cargo test --workspace    - 运行所有测试"
            echo "  cargo test --lib          - 单元测试"
            echo "  cargo test -p tail-tests  - 集成测试"
            echo ""
            echo "📦 NixOS Packaging:"
            echo "  just nix-package- 一键打包"
            echo "  just nix-install-local- 安装到用户环境"
            echo ""
            echo "📚 Documentation:"
            echo "  RUNNING_GUIDE.md          - 运行指南"
            echo "  NIXOS_INSTALL.md          - NixOS 安装"
            echo "  DEVELOPMENT_SUMMARY.md    - 开发总结"
            echo ""
            echo "════════════════════════════════════════════"
          '';

          LIBCLANG_PATH = "${pkgs.llvmPackages_latest.libclang}/lib";
        };

        # Build outputs
        packages = {
          default = tail-app;
          inherit tail-app tail-service;
        };

        # Default app
        apps.default = flake-utils.lib.mkApp {
          drv = tail-app;
        };

        # Formatter
        formatter = pkgs.alejandra;
      }
    )
    // {
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
    };
}
