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
in
{
  # 1. Gestion déclarative des Flatpaks via nix-flatpak
  services.flatpak = {
    enable = true;
    packages = [
      "rocks.shy.VacuumTube"
    ];
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

  # 3. Paquets système Flatpak et script lanceur
  environment.systemPackages = with pkgs; [
    flatpak
    vacuumTubeLauncher
  ];

  # 4. Raccourci .desktop pour VacuumTube
  environment.etc."xdg/autostart/vacuumtube.desktop".text = ''
    [Desktop Entry]
    Type=Application
    Name=VacuumTube
    Comment=Client YouTube TV open-source sans publicité
    Exec=vacuumtube
    Icon=video-television
    Categories=AudioVideo;Video;Player;TV;
  '';
}
