{ config, lib, pkgs, ... }:

let
  cfg = config.services.noos-htpc.ambilight;

  # Utilitaire CLI de gestion du service Ambilight Noos HTPC
  noosAmbilightScript = pkgs.writeShellScriptBin "noos-ambilight" ''
    set -euo pipefail

    ACTION="''${1:-status}"

    case "$ACTION" in
      status)
        echo -e "\033[1;34m[Noos HTPC Ambilight]\033[0m État du service HyperHDR :"
        systemctl --user status hyperhdr --no-pager || true
        ;;
      on|start)
        echo -e "\033[1;32m[Noos HTPC Ambilight]\033[0m Démarrage du service Ambilight..."
        systemctl --user start hyperhdr
        ;;
      off|stop)
        echo -e "\033[1;33m[Noos HTPC Ambilight]\033[0m Arrêt du service Ambilight..."
        systemctl --user stop hyperhdr
        ;;
      restart)
        echo -e "\033[1;34m[Noos HTPC Ambilight]\033[0m Redémarrage du service Ambilight..."
        systemctl --user restart hyperhdr
        ;;
      web|gui|config)
        IP=$(ip route get 1.1.1.1 2>/dev/null | grep -oP 'src \K\S+' || echo "localhost")
        echo -e "\033[1;34m[Noos HTPC Ambilight]\033[0m Interface d'administration HyperHDR :"
        echo -e "  -> \033[1;32mhttp://$IP:8090\033[0m (ou http://localhost:8090)"
        echo "Configurez vos rubans LED (WLED Wi-Fi, Philips Hue, USB Serial Adalight, ESP32) depuis tout appareil du réseau."
        ;;
      *)
        echo "Usage: noos-ambilight {status|on|off|restart|web}"
        exit 1
        ;;
    esac
  '';
in
{
  options.services.noos-htpc.ambilight = {
    enable = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = ''
        Activation de l'Ambilight dynamique Noos HTPC via HyperHDR.
        Capture continue de l'écran en temps réel (PipeWire / Wayland)
        pour éclairage immersif dans MPV, le Dashboard et tous les menus.
      '';
    };
    webPort = lib.mkOption {
      type = lib.types.port;
      default = 8090;
      description = "Port Web d'administration HyperHDR.";
    };
  };

  config = lib.mkIf cfg.enable {
    # 1. Paquet HyperHDR avec capture PipeWire native & Script de contrôle
    environment.systemPackages = [
      pkgs.hyperhdr
      noosAmbilightScript
    ];

    # 2. Permissions pour les contrôleurs LED USB (Adalight, Arduino, ESP32)
    users.users.noos.extraGroups = [ "dialout" "tty" ];

    # 3. Ouverture des ports firewall pour l'Ambilight (WLED, DDP, Web UI)
    networking.firewall = {
      allowedTCPPorts = [
        cfg.webPort  # Web UI HyperHDR (8090)
        8092         # JSON-RPC API HyperHDR
        19444        # FlatBuffers Hyperion/HyperHDR
        19445        # Proto port
      ];
      allowedUDPPorts = [
        19446        # Stream UDP
        21324        # WLED DDP Protocol (temps réel ultra basse latence)
        5568         # sACN / E1.31
      ];
    };

    # 4. Service utilisateur Systemd : lancé automatiquement à l'ouverture de session TV
    systemd.user.services.hyperhdr = {
      description = "HyperHDR Ambilight Daemon (Capture PipeWire & Rendu LED)";
      wantedBy = [ "default.target" ];
      after = [ "pipewire.service" "wireplumber.service" ];
      serviceConfig = {
        ExecStart = "${pkgs.hyperhdr}/bin/hyperhdr --service --pipewire";
        Restart = "on-failure";
        RestartSec = 3;
        Nice = -5; # Priorité temps réel fluide (60 FPS sans saccades)
      };
    };
  };
}
