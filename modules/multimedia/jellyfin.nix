{ config, lib, pkgs, ... }:

let
  # Lanceur optimisé pour TV grand écran avec MPV, Direct Play 4K HDR et plein écran forcé
  jellyfinTvLauncher = pkgs.writeShellScriptBin "jellyfin-tv" ''
    set -euo pipefail

    echo "[Noos HTPC] Lancement de Jellyfin (Lecteur MPV HDR 4K & Interface TV)..."

    export XDG_RUNTIME_DIR="''${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
    export DBUS_SESSION_BUS_ADDRESS="''${DBUS_SESSION_BUS_ADDRESS:-unix:path=''${XDG_RUNTIME_DIR}/bus}"
    export WAYLAND_DISPLAY="''${WAYLAND_DISPLAY:-wayland-0}"
    export QT_QPA_PLATFORM="wayland;xcb"
    export QT_WAYLAND_DISABLE_WINDOWDECORATION=1
    export QT_WAYLAND_SHELL_INTEGRATION=xdg-shell

    # Configuration automatique du profil Jellyfin Desktop
    CONF_DIR="$HOME/.local/share/jellyfin-desktop"
    mkdir -p "$CONF_DIR"

    # 1. Configuration principale pour le mode TV 10-foot et le plein écran universel sans bordures
    write_jellyfin_conf() {
      cat << 'EOF' > "$1"
{
    "sections": {
        "main": {
            "allowBrowserZoom": true,
            "alwaysOnTop": false,
            "autodetectCertBundle": true,
            "checkForUpdates": false,
            "disablemouse": false,
            "enableInputRepeat": true,
            "enableMPV": true,
            "enableWindowsMediaIntegration": true,
            "enableWindowsTaskbarIntegration": true,
            "forceAlwaysFS": true,
            "forceFSScreen": "",
            "fullscreen": true,
            "hdmi_poweron": false,
            "ignoreSSLErrors": false,
            "layout": "tv",
            "logLevel": "info",
            "minimizeOnDefocus": false,
            "sdlEnabled": true,
            "showPowerOptions": true,
            "useOpenGL": false,
            "useSystemVideoCodecs": true,
            "userWebClient": "",
            "webMode": "desktop"
        },
        "video": {
            "allow_transcode_to_hevc": false,
            "always_force_transcode": false,
            "force_transcode_4k": false,
            "force_transcode_av1": false,
            "force_transcode_dovi": false,
            "force_transcode_hdr": false,
            "force_transcode_hevc": false,
            "force_transcode_hi10p": false,
            "hardwareDecoding": "auto-safe",
            "prefer_transcode_to_h265": false,
            "refreshrate.auto_switch": false,
            "sync_mode": "audio"
        },
        "audio": {
            "channels": "auto",
            "device": "auto",
            "devicetype": "basic",
            "exclusive": false,
            "normalize": true,
            "passthrough.ac3": true,
            "passthrough.dts": true,
            "passthrough.dts-hd": true,
            "passthrough.eac3": true,
            "passthrough.truehd": true
        },
        "cec": {
            "activatesource": true,
            "enable": true
        }
    },
    "version": 7
}
EOF
    }

    write_jellyfin_conf "$CONF_DIR/jellyfin-desktop.conf"

    # Application récursive à tous les profils de profils/
    if [ -d "$CONF_DIR/profiles" ]; then
      for p in "$CONF_DIR"/profiles/*; do
        if [ -d "$p" ]; then
          write_jellyfin_conf "$p/jellyfin-desktop.conf"
        fi
      done
    fi

    # 2. Configuration MPV embarquée pour Jellyfin (Moteur de rendu HDR Rec.2020 sans bordures)
    cat << 'EOF' > "$CONF_DIR/mpv.conf"
vo=gpu-next
gpu-context=wayland
target-colorspace-hint=yes
tone-mapping=auto
hdr-compute-peak=yes
hwdec=auto-safe
fs=yes
border=no
keep-open=no
EOF

    # Propagation vers ~/.config/mpv/mpv.conf
    mkdir -p "$HOME/.config/mpv"
    cp "$CONF_DIR/mpv.conf" "$HOME/.config/mpv/mpv.conf"

    exec ${pkgs.jellyfin-media-player}/bin/jellyfin-desktop --tv --fullscreen "$@"
  '';
in
{
  # Déploiement du paquet Jellyfin Media Player (moteur libmpv natif) et du wrapper TV
  environment.systemPackages = with pkgs; [
    jellyfin-media-player
    jellyfinTvLauncher
  ];

  # Raccourci d'application .desktop
  environment.etc."xdg/autostart/jellyfin.desktop".text = ''
    [Desktop Entry]
    Type=Application
    Name=Jellyfin TV
    Comment=Lecteur multimédia Jellyfin avec moteur MPV 4K HDR
    Exec=jellyfin-tv
    Icon=jellyfin
    Categories=AudioVideo;Video;Player;TV;
  '';
}
