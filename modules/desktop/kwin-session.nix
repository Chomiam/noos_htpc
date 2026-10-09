{ config, lib, pkgs, ... }:

let
  gpuCfg = config.hardware.noos-htpc.gpu;
  dashboardPkg = pkgs.callPackage ../../dashboard/default.nix { };

  # Règles de fenêtrage KWin pour forcer le plein écran sans bordures façon Smart TV
  kwinRules = ''
    [1]
    Description=Noos HTPC Plein Ecran Universel
    noborder=true
    noborderrule=2
    fullscreen=true
    fullscreenrule=2
    wmclass=.*
    wmclassmatch=3
  '';

  # Script de lancement de session TV Wayland KWin HTPC
  noosTvSession = pkgs.writeShellScriptBin "noos-tv-session" ''
    set -e

    echo "[Noos HTPC] Démarrage de la session Wayland KWin..."

    # 1. Variables d'environnement Wayland pour grand écran TV
    export XDG_SESSION_TYPE=wayland
    export XDG_CURRENT_DESKTOP=KDE
    export GDK_BACKEND="wayland,x11"
    export QT_QPA_PLATFORM="wayland;xcb"
    export SDL_VIDEODRIVER="wayland,x11"
    export MOZ_ENABLE_WAYLAND=1
    export WEBKIT_DISABLE_COMPOSITING_MODE=0

    # 2. Vérification et création des dossiers multimédias et rétro
    mkdir -p /home/noos/Retro/ROMS /home/noos/Retro/BIOS /home/noos/IPTV /home/noos/.config

    # 3. Activation en tâche de fond du HDR et de la gestion des couleurs dès que KWin est actif
    ${lib.optionalString gpuCfg.enableHDR ''
    (
      for i in $(seq 1 10); do
        sleep 1
        if ${pkgs.kdePackages.libkscreen}/bin/kscreen-doctor -j >/dev/null 2>&1; then
          echo "[Noos HTPC] Activation du profil colorimétrique HDR10 (Rec.2020)..."
          ${pkgs.kdePackages.libkscreen}/bin/kscreen-doctor output.1.hdr.enable || true
          ${pkgs.kdePackages.libkscreen}/bin/kscreen-doctor output.HDMI-A-1.hdr.enable || true
          ${pkgs.kdePackages.libkscreen}/bin/kscreen-doctor output.DP-1.hdr.enable || true
          break
        fi
      done
    ) &
    ''}

    # 4. Lancement prioritaire de KWin Wayland avec le Dashboard Noos TV
    # Si le matériel ne supporte pas KWin (ex: VM sans 3D), repli immédiat sur Cage
    if ${pkgs.kdePackages.kwin}/bin/kwin_wayland \
         --no-lockscreen \
         --xwayland \
         --exit-with-session ${dashboardPkg}/bin/noos-tv-dashboard; then
      exit 0
    fi

    echo "[Noos HTPC] KWin Wayland indisponible sur ce matériel, bascule sur le compositeur de secours Cage..."
    exec ${pkgs.cage}/bin/cage -s -- ${dashboardPkg}/bin/noos-tv-dashboard
  '';
in
{
  options.services.noos-htpc.desktop = {
    enableKwinSession = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Activer la session TV KWin Wayland avec gestion colorimétrique HDR native et Dashboard Noos";
    };
  };

  config = lib.mkIf config.services.noos-htpc.desktop.enableKwinSession {
    # 1. Déploiement des règles KWin pour forcer le plein écran sans bordures sur la TV
    systemd.tmpfiles.rules = [
      "d /home/noos/.config 0755 noos users -"
      "C+ /home/noos/.config/kwinrulesrc 0644 noos users - ${pkgs.writeText "kwinrulesrc" kwinRules}"
    ];

    # 2. Paquets de session et compositeur Wayland
    environment.systemPackages = with pkgs; [
      dashboardPkg
      noosTvSession
      kdePackages.kwin
      kdePackages.libkscreen
      cage
      gamescope
    ];
  };
}
