#!/usr/bin/env bash
# ==============================================================================
# Noos HTPC - Lanceur d'applications Console / TV
# ==============================================================================
set -euo pipefail

ACTION="''${1:-es-de}"

case "$ACTION" in
    es-de|gaming)
        echo "[Noos HTPC] Lancement d'EmulationStation-DE..."
        exec es-de
        ;;
    retroarch)
        echo "[Noos HTPC] Lancement direct de RetroArch..."
        exec retroarch
        ;;
    mpv|media)
        echo "[Noos HTPC] Lancement de MPV en mode TV..."
        exec mpv --fs /home/noos/Videos
        ;;
    iptv)
        echo "[Noos HTPC] Lancement d'IPTV..."
        exec noos-iptv
        ;;
    files|thunar)
        echo "[Noos HTPC] Lancement de l'explorateur de fichiers..."
        exec thunar /home/noos
        ;;
    poweroff|shutdown)
        echo "[Noos HTPC] Extinction du système..."
        exec systemctl poweroff
        ;;
    reboot)
        echo "[Noos HTPC] Redémarrage du système..."
        exec systemctl reboot
        ;;
    *)
        echo "Usage: $0 {es-de|retroarch|mpv|iptv|files|poweroff|reboot}"
        exit 1
        ;;
esac
