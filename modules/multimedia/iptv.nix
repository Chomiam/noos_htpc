{ config, lib, pkgs, ... }:

let
  # Script de lancement IPTV universel pour Noos HTPC
  noosIptvLauncher = pkgs.writeShellScriptBin "noos-iptv" ''
    set -euo pipefail

    PLAYLIST_DIR="/home/noos/IPTV"
    mkdir -p "$PLAYLIST_DIR"

    # Vérifie si une playlist locale est présente
    LOCAL_M3U=$(find "$PLAYLIST_DIR" -name "*.m3u" -o -name "*.m3u8" | head -n 1 || true)

    if [ -n "$LOCAL_M3U" ]; then
      echo "[Noos IPTV] Lecture de la playlist locale : $LOCAL_M3U"
      exec ${pkgs.mpv}/bin/mpv --fullscreen --profile=auto-upscale "$LOCAL_M3U"
    elif command -v hypnotix >/dev/null 2>&1; then
      echo "[Noos IPTV] Lancement d'Hypnotix..."
      exec ${pkgs.hypnotix}/bin/hypnotix
    else
      echo "[Noos IPTV] Aucune playlist trouvée dans $PLAYLIST_DIR"
      notify-send "Noos IPTV" "Déposez vos fichiers .m3u dans /home/noos/IPTV/ ou configurez Hypnotix."
    fi
  '';
in
{
  # 1. Installation du lecteur IPTV Hypnotix et des dépendances IPTV
  environment.systemPackages = with pkgs; [
    hypnotix            # Lecteur IPTV dédié avec gestion de chaînes et EPG
    noosIptvLauncher    # Script lanceur TV Noos IPTV
  ];

  # 2. Raccourci .desktop pour l'accès TV
  environment.etc."xdg/autostart/noos-iptv-init.desktop".text = ''
    [Desktop Entry]
    Type=Application
    Name=Noos IPTV
    Comment=Lecteur de chaînes TV et flux en direct
    Exec=noos-iptv
    Icon=tv
    Categories=AudioVideo;TV;
  '';
}
