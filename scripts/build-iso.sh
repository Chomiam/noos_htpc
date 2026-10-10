#!/usr/bin/env bash
# ==============================================================================
# Noos HTPC - Script de compilation locale de l'Image ISO
# ==============================================================================
set -euo pipefail

COLOR_INFO="\033[1;34m"
COLOR_SUCCESS="\033[1;32m"
COLOR_WARN="\033[1;33m"
COLOR_RESET="\033[0m"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo -e "${COLOR_INFO}======================================================${COLOR_RESET}"
echo -e "${COLOR_INFO}        GÉNÉRATION DE L'IMAGE ISO NOOS HTPC          ${COLOR_RESET}"
echo -e "${COLOR_INFO}======================================================${COLOR_RESET}"

if ! command -v nix >/dev/null 2>&1; then
    echo -e "${COLOR_WARN}[!] La commande 'nix' n'est pas installée sur cette machine.${COLOR_RESET}"
    echo -e "Pour compiler l'ISO localement, installez Nix ou déléguez le build à GitHub Actions."
    exit 1
fi

echo -e "${COLOR_INFO}[*] Compilation de la dérivation ISO (nix build .#iso)...${COLOR_RESET}"
nix build "$SCRIPT_DIR#iso" --extra-experimental-features "nix-command flakes" --impure --print-out-paths

ISO_FILE=$(find "$SCRIPT_DIR/result/iso" -name "*.iso" | head -n 1)

if [ -n "$ISO_FILE" ]; then
    echo -e "\n${COLOR_SUCCESS}======================================================${COLOR_RESET}"
    echo -e "${COLOR_SUCCESS}[✓] Image ISO générée avec succès :${COLOR_RESET}"
    echo -e "👉 \033[1m$ISO_FILE\033[0m"
    echo -e "${COLOR_SUCCESS}======================================================${COLOR_RESET}"
    echo -e "Pour flasher sur une clé USB (ex: /dev/sdX) :"
    echo -e "  sudo dd if=$ISO_FILE of=/dev/sdX bs=4M status=progress conv=fsync"
else
    echo -e "${COLOR_WARN}[!] Fichier ISO introuvable dans result/iso.${COLOR_RESET}"
fi
