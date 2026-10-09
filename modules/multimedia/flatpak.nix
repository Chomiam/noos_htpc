{ config, lib, pkgs, ... }:

let
  # Lanceur pratique en ligne de commande ou pour les raccourcis TV
  vacuumTubeLauncher = pkgs.writeShellScriptBin "vacuumtube" ''
    set -euo pipefail

    echo "[Noos Flatpak] Lancement de VacuumTube (YouTube TV) sous Wayland..."

    # Définition des variables de session utilisateur Wayland et D-Bus
    export XDG_RUNTIME_DIR="''${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
    export DBUS_SESSION_BUS_ADDRESS="''${DBUS_SESSION_BUS_ADDRESS:-unix:path=''${XDG_RUNTIME_DIR}/bus}"
    export WAYLAND_DISPLAY="''${WAYLAND_DISPLAY:-wayland-0}"

    if ! flatpak list --app 2>/dev/null | grep -q "rocks.shy.VacuumTube"; then
      echo "[Noos Flatpak] Installation de VacuumTube depuis Flathub..."
      flatpak install -y flathub rocks.shy.VacuumTube
    fi

    exec ${pkgs.flatpak}/bin/flatpak run \
      --nosocket=x11 \
      --socket=wayland \
      rocks.shy.VacuumTube \
      --ozone-platform-hint=auto \
      --ozone-platform=wayland \
      --enable-features=WaylandWindowDecorations \
      "$@"
  '';

  # Lanceur Sober (Roblox) en mode console TV et support manette intégral
  soberLauncher = pkgs.writeShellScriptBin "sober" ''
    set -euo pipefail

    echo "[Noos Gaming] Lancement de Sober (Roblox) en mode console TV..."

    # Définition des variables de session utilisateur Wayland et D-Bus
    export XDG_RUNTIME_DIR="''${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
    export DBUS_SESSION_BUS_ADDRESS="''${DBUS_SESSION_BUS_ADDRESS:-unix:path=''${XDG_RUNTIME_DIR}/bus}"
    export WAYLAND_DISPLAY="''${WAYLAND_DISPLAY:-wayland-0}"

    # Assurer la configuration console & manette dans ~/.var/app/org.vinegarhq.Sober/config/sober/config.json
    SOBER_CONF_DIR="$HOME/.var/app/org.vinegarhq.Sober/config/sober"
    mkdir -p "$SOBER_CONF_DIR"
    SOBER_CONF="$SOBER_CONF_DIR/config.json"

    if [ ! -f "$SOBER_CONF" ]; then
      cat << 'EOF' > "$SOBER_CONF"
{
    "allow_gamepad_permission": true,
    "close_on_leave": true,
    "discord_rpc_enabled": false,
    "discord_rpc_show_join_button": false,
    "enable_gamemode": true,
    "enable_hidpi": true,
    "enable_mobile_home_screen": false,
    "graphics_optimization_mode": "quality",
    "server_location_indicator_enabled": false,
    "touch_mode": "off",
    "use_console_experience": true,
    "use_libsecret": false,
    "use_opengl": false
}
EOF
    else
      ${pkgs.gnused}/bin/sed -i 's/"use_console_experience": false/"use_console_experience": true/g' "$SOBER_CONF" || true
      ${pkgs.gnused}/bin/sed -i 's/"allow_gamepad_permission": false/"allow_gamepad_permission": true/g' "$SOBER_CONF" || true
      ${pkgs.gnused}/bin/sed -i 's/"close_on_leave": false/"close_on_leave": true/g' "$SOBER_CONF" || true
    fi

    if ! flatpak list --app 2>/dev/null | grep -q "org.vinegarhq.Sober"; then
      echo "[Noos Gaming] Installation de Sober depuis Flathub..."
      flatpak install -y flathub org.vinegarhq.Sober
    fi

    exec ${pkgs.flatpak}/bin/flatpak run \
      --device=all \
      --socket=wayland \
      --nosocket=x11 \
      --nosocket=fallback-x11 \
      org.vinegarhq.Sober \
      "$@"
  '';
in
{
  # 1. Gestion déclarative des Flatpaks via nix-flatpak
  services.flatpak = {
    enable = true;
    packages = [
      "rocks.shy.VacuumTube"
      "org.vinegarhq.Sober"
    ];
    overrides = {
      "org.vinegarhq.Sober" = {
        Context = {
          devices = [ "all" ];
          sockets = [ "wayland" "!x11" "!fallback-x11" "pulseaudio" ];
          shared = [ "network" "ipc" ];
        };
      };
    };
    update.auto = {
      enable = true;
      onCalendar = "weekly";
    };
  };

  # 2. Portails XDG requis pour l'exécution fluide des Flatpaks sous Wayland / Cage / Gamescope
  xdg.portal = {
    enable = true;
    wlr.enable = lib.mkDefault true;
    extraPortals = with pkgs; [
      xdg-desktop-portal-gtk
    ];
    config.common.default = "*";
  };

  # 3. Paquets système Flatpak et scripts lanceurs
  environment.systemPackages = with pkgs; [
    flatpak
    vacuumTubeLauncher
    soberLauncher
  ];

  # 4. Raccourcis .desktop pour VacuumTube et Sober
  environment.etc."xdg/autostart/vacuumtube.desktop".text = ''
    [Desktop Entry]
    Type=Application
    Name=VacuumTube
    Comment=Client YouTube TV open-source sans publicité
    Exec=vacuumtube
    Icon=video-television
    Categories=AudioVideo;Video;Player;TV;
  '';

  environment.etc."xdg/autostart/sober.desktop".text = ''
    [Desktop Entry]
    Type=Application
    Name=Sober (Roblox)
    Comment=Plateforme Roblox en mode console avec support manette
    Exec=sober
    Icon=input-gaming
    Categories=Game;ArcadeGame;
  '';
}
