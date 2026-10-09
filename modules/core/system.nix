{ config, lib, pkgs, ... }:

let
  # Script noos-update embarqué dans le PATH système
  noosUpdateScript = pkgs.writeShellScriptBin "noos-update" ''
    set -euo pipefail
    echo -e "\033[1;34m[Noos HTPC]\033[0m Lancement de la mise à jour du système..."
    
    TARGET_FLAKE="/home/chomiam/Projets/noos_htpc"
    if [ ! -d "$TARGET_FLAKE" ]; then
      TARGET_FLAKE="/etc/nixos"
    fi

    echo -e "\033[1;33m[*] Reconstruction de la configuration depuis $TARGET_FLAKE...\033[0m"
    if sudo nixos-rebuild switch --flake "$TARGET_FLAKE#htpc"; then
      echo -e "\033[1;32m[✓] Mise à jour appliquée avec succès !\033[0m"
    else
      echo -e "\033[1;31m[✗] Échec de la mise à jour. Aucun changement appliqué (sécurité rollback active).\033[0m"
      exit 1
    fi
  '';

  # Script de rollback rapide
  noosRollbackScript = pkgs.writeShellScriptBin "noos-rollback" ''
    set -euo pipefail
    echo -e "\033[1;33m[Noos HTPC]\033[0m Rétrogradation vers la génération précédente..."
    sudo nixos-rebuild switch --rollback
    echo -e "\033[1;32m[✓] Rétrogradation effectuée avec succès !\033[0m"
  '';
in
{
  # 1. Fuseau horaire et localisation en français
  time.timeZone = lib.mkDefault "Europe/Paris";
  i18n.defaultLocale = "fr_FR.UTF-8";
  i18n.extraLocaleSettings = {
    LC_ADDRESS = "fr_FR.UTF-8";
    LC_IDENTIFICATION = "fr_FR.UTF-8";
    LC_MEASUREMENT = "fr_FR.UTF-8";
    LC_MONETARY = "fr_FR.UTF-8";
    LC_NAME = "fr_FR.UTF-8";
    LC_NUMERIC = "fr_FR.UTF-8";
    LC_PAPER = "fr_FR.UTF-8";
    LC_TELEPHONE = "fr_FR.UTF-8";
    LC_TIME = "fr_FR.UTF-8";
  };
  console.keyMap = "fr";

  # 2. Gestion de la mémoire et swap ZRAM (optimal pour mini-PC HTPC)
  zramSwap = {
    enable = true;
    algorithm = "zstd";
    memoryPercent = 50;
  };

  # 3. Microcodes CPU et firmwares propriétaires indispensables (Wi-Fi, Bluetooth, GPU)
  hardware.enableRedistributableFirmware = true;
  hardware.cpu.intel.updateMicrocode = lib.mkDefault true;
  hardware.cpu.amd.updateMicrocode = lib.mkDefault true;

  # 4. Activation de Flakes et optimisation du store Nix
  nix.settings = {
    experimental-features = [ "nix-command" "flakes" ];
    auto-optimise-store = true;
    warn-dirty = false;
  };

  # 5. Paquets système essentiels et utilitaires Noos
  environment.systemPackages = with pkgs; [
    # Utilitaires Noos HTPC
    noosUpdateScript
    noosRollbackScript

    # Outils de diagnostic et monitoring
    btop
    fastfetch
    pciutils
    usbutils
    lshw
    lm_sensors
    htop

    # Réseau et transfert
    curl
    wget
    git
    rsync
    jq

    # Utilitaires système
    parted
    e2fsprogs
    dosfstools
    ntfs3g
  ];
}
