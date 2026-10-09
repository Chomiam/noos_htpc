{ config, lib, pkgs, inputs, ... }:

{
  imports = [
    ./hardware-configuration.nix
    ../../modules
  ] ++ lib.optional (builtins.pathExists ./host-settings.local.nix) ./host-settings.local.nix;

  # ==============================================================================
  # IDENTITÉ DE LA MACHINE & RÉSEAU
  # ==============================================================================
  networking = {
    hostName = lib.mkDefault "noos-htpc";
    networkmanager.enable = true;
    firewall = {
      enable = true;
      # Ports ouverts par défaut : SSH (22), Samba (139, 445), WSDD (5357/3702), Avahi/mDNS (5353)
      allowedTCPPorts = [ 22 139 445 5357 ];
      allowedUDPPorts = [ 137 138 3702 5353 ];
    };
  };

  # ==============================================================================
  # SÉLECTION DU PROFIL GRAPHIQUE (AMD / INTEL / NVIDIA / NVIDIA-LEGACY)
  # ==============================================================================
  # Changez simplement la valeur ci-dessous selon le matériel de votre Mini PC :
  #   - "amd"           : Pour Mini PC AMD Ryzen / Radeon (Beelink SER, Minisforum, etc.)
  #   - "intel"         : Pour Mini PC Intel NUC, Celeron N100, Core i3/i5/i7 (QuickSync iHD)
  #   - "nvidia"        : Pour cartes graphiques Nvidia modernes (GTX 1650 et supérieur)
  #   - "nvidia-legacy" : Pour anciennes cartes GeForce (pilotes 470xx)
  hardware.noos-htpc.gpu = {
    profile = lib.mkDefault "amd";    # <-- Modifiez ici selon votre GPU
    enableHDR = lib.mkDefault true;   # Active le support HDR sur TV compatible (AMD / Nvidia)
  };

  # ==============================================================================
  # GESTION DES PAQUETS NON-LIBRES & VERSION SYSTÈME
  # ==============================================================================
  nixpkgs.config.allowUnfree = true;

  # Version d'état NixOS (synchronisée sur le standard Noos 26.05)
  system.stateVersion = "26.05";
}
