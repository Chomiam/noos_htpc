{ config, lib, pkgs, ... }:

let
  # Script noos-update embarqué dans le PATH système
  noosUpdateScript = pkgs.writeShellScriptBin "noos-update" ''
    set -euo pipefail
    
    ACTION="''${1:---apply}"
    CHANNEL="''${2:-testing}"
    
    TARGET_DIR="/etc/nixos"
    if [ ! -d "$TARGET_DIR/.git" ] && [ -d "/home/chomiam/Projets/noos_htpc/.git" ]; then
      TARGET_DIR="/home/chomiam/Projets/noos_htpc"
    fi

    # Si /etc/nixos n'a pas encore de dépôt Git initialisé, le configurer avec le remote officiel
    if [ ! -d "$TARGET_DIR/.git" ]; then
      echo -e "\033[1;34m[Noos Update]\033[0m Initialisation du suivi Git local dans $TARGET_DIR..."
      cd "$TARGET_DIR"
      git init -b "$CHANNEL"
      git remote add origin "https://github.com/Chomiam/noos_htpc.git"
      git config user.name "Noos HTPC"
      git config user.email "htpc@noos.local"
    fi

    if [ "$ACTION" = "--check" ]; then
      echo "[Noos Update] Recherche de mises à jour sur le canal '$CHANNEL'..."
      cd "$TARGET_DIR"
      git fetch origin "$CHANNEL" --quiet || true
      REMOTE_HASH=$(git rev-parse "origin/$CHANNEL" 2>/dev/null || echo "")
      LOCAL_HASH=$(git rev-parse HEAD 2>/dev/null || echo "")
      
      if [ -n "$REMOTE_HASH" ] && [ "$REMOTE_HASH" != "$LOCAL_HASH" ]; then
        echo "UPDATE_AVAILABLE"
        git log "HEAD..origin/$CHANNEL" --oneline -n 10
      else
        echo "UP_TO_DATE"
      fi
      exit 0
    fi

    echo -e "\033[1;34m[Noos HTPC]\033[0m Lancement de la mise à jour (Canal: $CHANNEL)..."
    cd "$TARGET_DIR"

    # 1. Sauvegarde préventive de la configuration matérielle unique à ce PC physique
    echo -e "\033[1;33m[1/4] Protection des paramètres matériels locaux (*.local.nix)...\033[0m"
    mkdir -p "$TARGET_DIR/.local-backups"
    if [ -f "$TARGET_DIR/hosts/htpc/hardware-configuration.local.nix" ]; then
      cp -f "$TARGET_DIR/hosts/htpc/hardware-configuration.local.nix" "$TARGET_DIR/.local-backups/"
    fi
    if [ -f "$TARGET_DIR/hosts/htpc/hardware.local.nix" ]; then
      cp -f "$TARGET_DIR/hosts/htpc/hardware.local.nix" "$TARGET_DIR/.local-backups/"
    fi
    if [ -f "$TARGET_DIR/hosts/htpc/host-settings.local.nix" ]; then
      cp -f "$TARGET_DIR/hosts/htpc/host-settings.local.nix" "$TARGET_DIR/.local-backups/"
    fi

    # 2. Synchronisation des sources Git sans écraser les fichiers locaux ignorés
    echo -e "\033[1;33m[2/4] Synchronisation des sources Git ($CHANNEL)...\033[0m"
    git fetch origin "$CHANNEL"
    git checkout -B "$CHANNEL" "origin/$CHANNEL"

    # Restauration immédiate des fichiers matériels locaux si besoin
    if [ -f "$TARGET_DIR/.local-backups/hardware-configuration.local.nix" ]; then
      cp -f "$TARGET_DIR/.local-backups/hardware-configuration.local.nix" "$TARGET_DIR/hosts/htpc/hardware-configuration.local.nix"
    fi
    if [ -f "$TARGET_DIR/.local-backups/hardware.local.nix" ]; then
      cp -f "$TARGET_DIR/.local-backups/hardware.local.nix" "$TARGET_DIR/hosts/htpc/hardware.local.nix"
    fi
    if [ -f "$TARGET_DIR/.local-backups/host-settings.local.nix" ]; then
      cp -f "$TARGET_DIR/.local-backups/host-settings.local.nix" "$TARGET_DIR/hosts/htpc/host-settings.local.nix"
    fi
    echo -e "\033[1;32m[✓] Configuration matérielle déclarative préservée (hardware-configuration.local.nix & greffe GPU)\033[0m"

    # 3. Mise à jour impérative du fichier flake.lock
    echo -e "\033[1;33m[3/4] Mise à jour des dépendances et du flake.lock...\033[0m"
    nix flake update

    # 4. Reconstruction déclarative NixOS avec prise en compte des fichiers locaux (--impure path:...)
    echo -e "\033[1;33m[4/4] Reconstruction de la configuration NixOS ($TARGET_DIR#htpc)...\033[0m"
    if sudo nixos-rebuild switch --impure --flake "path:$TARGET_DIR#htpc"; then
      echo -e "\033[1;32m[✓] Mise à jour appliquée avec succès !\033[0m"
      exit 0
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
  # 1. Services d'accessibilité universelle et périphériques virtuels
  services.gnome.at-spi2-core.enable = true;
  hardware.uinput.enable = true;

  # 2. Fuseau horaire et localisation en français
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

  # Configuration du clavier physique en Français AZERTY (X11, Wayland, Cage, KWin)
  services.xserver.xkb = {
    layout = "fr";
    variant = "";
  };

  environment.sessionVariables = {
    XKB_DEFAULT_LAYOUT = "fr";
    XKB_DEFAULT_MODEL = "pc105";
  };

  # 3. Gestion de la mémoire et swap ZRAM (optimal pour mini-PC HTPC)
  zramSwap = {
    enable = true;
    algorithm = "zstd";
    memoryPercent = 50;
  };

  # 4. Microcodes CPU et firmwares propriétaires indispensables (Wi-Fi, Bluetooth, GPU)
  hardware.enableRedistributableFirmware = true;
  hardware.cpu.intel.updateMicrocode = lib.mkDefault true;
  hardware.cpu.amd.updateMicrocode = lib.mkDefault true;

  # 5. Activation de Flakes, optimisation du store Nix & Cache binaire Cachix Noos
  nix.settings = {
    experimental-features = [ "nix-command" "flakes" ];
    auto-optimise-store = true;
    warn-dirty = false;
    trusted-users = [ "root" "noos" "@wheel" ];
    substituters = [
      "https://cache.nixos.org"
      "https://noos.cachix.org"
    ];
    trusted-public-keys = [
      "cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY="
      "noos.cachix.org-1:oA+kmOj0Yvzq6XWXVDFlTq5wGdY2gpr0cpzB0P9ndKI="
    ];
  };

  # 6. Garantie d'intégrité et de permissions du dépôt /etc/nixos pour les mises à jour
  system.activationScripts.noosRepoInit = lib.stringAfter [ "users" "groups" ] ''
    if [ -d /etc/nixos ]; then
      ${pkgs.git}/bin/git config --system --add safe.directory /etc/nixos || true
      chown -R noos:users /etc/nixos || true
      if [ ! -d /etc/nixos/.git ]; then
        echo "[Noos Init] Initialisation automatique du dépôt Git dans /etc/nixos..."
        ${pkgs.git}/bin/git init -b testing /etc/nixos || true
        ${pkgs.git}/bin/git -C /etc/nixos remote add origin "https://github.com/Chomiam/noos_htpc.git" || true
        ${pkgs.git}/bin/git -C /etc/nixos fetch origin testing || true
        ${pkgs.git}/bin/git -C /etc/nixos reset --mixed origin/testing || true
        chown -R noos:users /etc/nixos || true
      fi
    fi
  '';

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

  # 6. Identité de distribution Noos HTPC (renommage des entrées GRUB et de l'OS)
  system.nixos = {
    distroName = "Noos-HTPC";
    distroId = "noos-htpc";
  };
}
