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

        # 自定义源码过滤器，包含 Cargo 源码、字体文件和图标文件
        # cleanCargoSource 默认只保留 Rust/Cargo 文件，会过滤掉 .ttf 字体和 .svg 图标
        srcWithAssets = pkgs.lib.cleanSourceWith {
          src = ./.;
          filter = path: type:
          # 保留字体文件
            (pkgs.lib.hasSuffix ".ttf" path)
            ||
            # 保留 SVG 图标文件
            (pkgs.lib.hasSuffix ".svg" path)
            ||
            # 保留 desktop 文件
            (pkgs.lib.hasSuffix ".desktop" path)
            ||
            # 保留 Crane 默认的 Cargo 源码
            (craneLib.filterCargoSources path type);
        };

        # Common build inputs for the crate
        commonArgs = {
          src = srcWithAssets;

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

        # 字体包
        fonts = with pkgs; [
          jetbrains-mono # 等宽英文字体
          lxgw-wenkai # 霞鹜文楷（中文）
          noto-fonts-cjk-sans # Noto Sans CJK（中文后备）
        ];

        # 运行时库依赖（用于 makeWrapper）
        runtimeLibs = pkgs.lib.makeLibraryPath [
          pkgs.wayland
          pkgs.libxkbcommon
          pkgs.libGL
          pkgs.mesa
        ];

        # 字体路径（用于环境变量）
        fontPaths = pkgs.lib.makeSearchPath "share/fonts" fonts;

        # Binary build - 原始构建
        tail-app-unwrapped = craneLib.buildPackage (commonArgs
          // {
            inherit cargoArtifacts;
            cargoExtraArgs = "--package tail-app --bin tail-app";
          });

        tail-service-unwrapped = craneLib.buildPackage (commonArgs
          // {
            inherit cargoArtifacts;
            cargoExtraArgs = "--package tail-app --bin tail-service";
          });

        # 图标文件路径
        tailIconSvg = ./tail-gui/assets/icons/tail.svg;

        # 包装后的二进制文件，设置运行时库路径、字体路径，并安装图标和 desktop 文件
        tail-app =
          pkgs.runCommand "tail-app" {
            nativeBuildInputs = [pkgs.makeWrapper];
          } ''
                      mkdir -p $out/bin
                      mkdir -p $out/share/applications
                      mkdir -p $out/share/icons/hicolor/scalable/apps

                      # 包装二进制文件
                      makeWrapper ${tail-app-unwrapped}/bin/tail-app $out/bin/tail-app \
                        --prefix LD_LIBRARY_PATH : "${runtimeLibs}" \
                        --set TAIL_FONT_PATH "${fontPaths}"

                      # 安装图标
                      cp ${tailIconSvg} $out/share/icons/hicolor/scalable/apps/tail.svg

                      # 安装 desktop 文件
                      cat > $out/share/applications/tail.desktop << EOF
            [Desktop Entry]
            Name=TaiL
            GenericName=Time Tracker
            Comment=Track your application usage time
            Exec=$out/bin/tail-app
            Icon=tail
            Terminal=false
            Type=Application
            Categories=Utility;Monitor;
            Keywords=time;tracker;productivity;usage;
            StartupWMClass=tail
            EOF
          '';

        tail-service =
          pkgs.runCommand "tail-service" {
            nativeBuildInputs = [pkgs.makeWrapper];
          } ''
            mkdir -p $out/bin
            makeWrapper ${tail-service-unwrapped}/bin/tail-service $out/bin/tail-service \
              --prefix LD_LIBRARY_PATH : "${runtimeLibs}"
          '';
      in {
        # Development environment
        devShells.default = pkgs.mkShell {
          buildInputs =
            [
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
            ]
            ++ (with pkgs;
              [
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
                act # GitHub Actions 本地运行工具

                # For testing IPC (can use socat to test socket)
                socat
                just
              ]
              ++ fonts); # 添加字体依赖

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

            # 设置 TaiL 字体路径
            export TAIL_FONT_PATH="${fontPaths}"

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
      nixosModules.default = {
        config,
        pkgs,
        ...
      }: {
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
