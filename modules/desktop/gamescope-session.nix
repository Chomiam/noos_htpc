{ config, lib, pkgs, ... }:

let
  gpuCfg = config.hardware.noos-htpc.gpu;

  # Script de lancement de session TV Gamescope
  noosTvSession = pkgs.writeShellScriptBin "noos-tv-session" ''
    set -e

    # Variables d'environnement Wayland / SDL / Qt pour TV
    export XDG_SESSION_TYPE=wayland
    export SDL_VIDEODRIVER="wayland,x11"
    export QT_QPA_PLATFORM="wayland;xcb"
    export GDK_BACKEND="wayland,x11"

    # Vérification et création des dossiers rétro si nécessaire
    mkdir -p /home/noos/Retro/ROMS /home/noos/Retro/BIOS /home/noos/IPTV

    # Détection des arguments Gamescope selon profil GPU
    GAMESCOPE_ARGS=(
      "-W" "1920"
      "-H" "1080"
      "-r" "60"
      "--fullscreen"
      "--adaptive-sync"
      "-F" "fsr"
    )

    ${lib.optionalString gpuCfg.enableHDR ''
    # Activation du passthrough HDR si supporté
    GAMESCOPE_ARGS+=( "--hdr-enabled" "--hdr-itm-enable" )
    ''}

    echo "[Noos HTPC] Démarrage de la session Gamescope avec args : ''${GAMESCOPE_ARGS[@]}"

    # Lancement prioritaire d'EmulationStation-DE ou du launcher HTPC
    if command -v es-de >/dev/null 2>&1; then
      exec ${pkgs.gamescope}/bin/gamescope "''${GAMESCOPE_ARGS[@]}" -- es-de
    else
      # Secours : lance RetroArch ou un terminal TV
      exec ${pkgs.gamescope}/bin/gamescope "''${GAMESCOPE_ARGS[@]}" -- retroarch
    fi
  '';
in
{
  options.services.noos-htpc.desktop = {
    enableGamescope = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Activer Gamescope comme micro-compositeur haute performance pour la TV";
    };
  };

  config = lib.mkIf config.services.noos-htpc.desktop.enableGamescope {
    # 1. Activation de Gamescope avec privilèges temps réel (capSysNice)
    programs.gamescope = {
      enable = true;
      capSysNice = true;
    };

    # 2. Activation de GameMode (optimisation CPU/GPU automatique en jeu)
    programs.gamemode = {
      enable = true;
      enableRenice = true;
      settings = {
        general = {
          renice = 10;
        };
        custom = {
          start = "${pkgs.libnotify}/bin/notify-send 'GameMode' 'Optimisations activées'";
          end = "${pkgs.libnotify}/bin/notify-send 'GameMode' 'Optimisations désactivées'";
        };
      };
    };

    # 3. Export du script de session dans les paquets système
    environment.systemPackages = [
      noosTvSession
      pkgs.gamescope
    ];
  };
}
