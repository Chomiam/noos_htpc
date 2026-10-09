#!/usr/bin/env bash
# ==============================================================================
# Noos HTPC - Script de Mise à Jour Système Sécurisée
# ==============================================================================
set -euo pipefail

COLOR_RESET="\033[0m"
COLOR_INFO="\033[1;34m"
COLOR_SUCCESS="\033[1;32m"
COLOR_WARN="\033[1;33m"
COLOR_ERROR="\033[1;31m"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo -e "${COLOR_INFO}======================================================${COLOR_RESET}"
echo -e "${COLOR_INFO}           NOOS HTPC - MISE À JOUR SYSTÈME            ${COLOR_RESET}"
echo -e "${COLOR_INFO}======================================================${COLOR_RESET}"

# 1. Vérification du répertoire de configuration
if [ ! -f "$SCRIPT_DIR/flake.nix" ]; then
    echo -e "${COLOR_ERROR}[ERREUR] flake.nix introuvable dans $SCRIPT_DIR${COLOR_RESET}"
    exit 1
fi

echo -e "${COLOR_INFO}[*] Répertoire du projet : ${SCRIPT_DIR}${COLOR_RESET}"

# 2. Vérification de l'état Git (s'il y a des modifications non commitées)
if command -v git >/dev/null 2>&1 && [ -d "$SCRIPT_DIR/.git" ]; then
    echo -e "${COLOR_INFO}[*] Branche Git active : $(git -C "$SCRIPT_DIR" rev-parse --abbrev-ref HEAD)${COLOR_RESET}"
    if [ -n "$(git -C "$SCRIPT_DIR" status --porcelain)" ]; then
        echo -e "${COLOR_WARN}[AVERTISSEMENT] Des fichiers non suivis ou modifiés sont présents.${COLOR_RESET}"
        echo -e "${COLOR_WARN}Nix Flakes requiert que tous les fichiers soient ajoutés au stage Git (git add).${COLOR_RESET}"
        git -C "$SCRIPT_DIR" add -N . || true
    fi
fi

# 3. Exécution de la reconstruction NixOS
echo -e "\n${COLOR_INFO}[*] Déclenchement de 'nixos-rebuild switch'...${COLOR_RESET}"
if sudo nixos-rebuild switch --flake "$SCRIPT_DIR#htpc"; then
    echo -e "\n${COLOR_SUCCESS}======================================================${COLOR_RESET}"
    echo -e "${COLOR_SUCCESS}[✓] Mise à jour réussie et activée immédiatement !${COLOR_RESET}"
    echo -e "${COLOR_SUCCESS}======================================================${COLOR_RESET}"
    echo -e "En cas de souci lors d'un prochain redémarrage :"
    echo -e "  1. Sélectionnez simplement la génération précédente dans le menu de démarrage UEFI."
    echo -e "  2. Ou exécutez la commande : \033[1mnoos-rollback\033[0m"
else
    echo -e "\n${COLOR_ERROR}======================================================${COLOR_RESET}"
    echo -e "${COLOR_ERROR}[✗] Échec de la mise à jour.${COLOR_RESET}"
    echo -e "${COLOR_ERROR}Votre système en cours n'a subi AUCUNE altération (rollback automatique).${COLOR_RESET}"
    echo -e "${COLOR_ERROR}======================================================${COLOR_RESET}"
    exit 1
fi
