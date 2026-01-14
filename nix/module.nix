{
  config,
  lib,
  pkgs,
  ...
}:
with lib; let
  cfg = config.services.tail;
in {
  options.services.tail = {
    enable = mkEnableOption "TaiL window time tracker service";

    package = mkOption {
      type = types.package;
      default = pkgs.tail-service;
      defaultText = literalExpression "pkgs.tail-service";
      description = "The TaiL service package to use.";
    };

    user = mkOption {
      type = types.str;
      default = "user";
      description = "User to run the TaiL service as.";
    };

    afkTimeout = mkOption {
      type = types.int;
      default = 300;
      description = "AFK timeout in seconds (default: 300 = 5 minutes).";
    };

    logLevel = mkOption {
      type = types.enum ["error" "warn" "info" "debug" "trace"];
      default = "info";
      description = "Log level for the service.";
    };

    autoStart = mkOption {
      type = types.bool;
      default = true;
      description = "Whether to start TaiL service automatically.";
    };
  };

  config = mkIf cfg.enable {
    # 确保包可用
    environment.systemPackages = [cfg.package];

    # Systemd 用户服务
    systemd.user.services.tail = {
      description = "TaiL Window Time Tracker Service";
      documentation = ["https://github.com/yourusername/tail"];
      
      wantedBy = mkIf cfg.autoStart ["graphical-session.target"];
      after = ["graphical-session.target"];
      partOf = ["graphical-session.target"];

      serviceConfig = {
        Type = "simple";
        ExecStart = "${cfg.package}/bin/tail-service";
        Restart = "on-failure";
        RestartSec = "5s";
        
        # 环境变量
        Environment = [
          "RUST_LOG=${cfg.logLevel}"
          "RUST_BACKTRACE=1"
        ];

        # 安全设置
        PrivateTmp = true;
        ProtectSystem = "strict";
        ProtectHome = "read-only";
        NoNewPrivileges = true;

        # 允许写入数据目录
        ReadWritePaths = [
          "%h/.local/share/tail"
        ];
      };
    };

    # 为 GUI 应用创建桌面入口
    xdg.desktopEntries = mkIf (cfg.package != null) {
      tail = {
        name = "TaiL";
        genericName = "Window Time Tracker";
        comment = "Track window usage time on Hyprland/Wayland";
        exec = "${cfg.package}/bin/tail-app";
        icon = "utilities-system-monitor";
        terminal = false;
        type = "Application";
        categories = ["Utility" "System" "Monitor"];
        keywords = ["time" "tracker" "window" "hyprland" "wayland"];
      };
    };
  };
}