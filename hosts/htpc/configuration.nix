{ config, lib, pkgs, inputs, ... }:

let
  # Recherche déclarative du matériel local (soit dans le Flake, soit sur le système hôte)
  # Permet à NixOS de toujours trouver la configuration matérielle propre à cette machine physique,
  # y compris lorsque la commande rebuild est lancée dans un dépôt Git avec fichiers ignorés.
  findLocalFile = relativePath:
    let
      candidatePaths = [
        (./. + "/${relativePath}")
        (/etc/nixos/hosts/htpc + "/${relativePath}")
        (/home/chomiam/Projets/noos_htpc/hosts/htpc + "/${relativePath}")
      ];
      existingPaths = builtins.filter builtins.pathExists candidatePaths;
    in
    if existingPaths != [] then builtins.head existingPaths else null;

  localHwConfig = findLocalFile "hardware-configuration.local.nix";
  localHardwareGraft = findLocalFile "hardware.local.nix";
  localHostSettings = findLocalFile "host-settings.local.nix";
in
{
  imports = [
    # 1. Matériel détecté par nixos-generate-config (disques, partitions UUID, modules noyau)
    (if localHwConfig != null
     then localHwConfig
     else ./hardware-configuration.nix)

    # 2. Modules système Noos HTPC
    ../../modules
  ]
  # 3. Greffe matérielle spécifique (GPU: amd.nix, intel.nix, etc. - unique à cette machine et jamais écrasé par Git)
  ++ lib.optional (localHardwareGraft != null) localHardwareGraft
  ++ lib.optional (localHostSettings != null && localHardwareGraft == null) localHostSettings;

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
  # GESTION DES PAQUETS NON-LIBRES, FLAKES & VERSION SYSTÈME
  # ==============================================================================
  nixpkgs.config.allowUnfree = true;
  nix.settings.experimental-features = [ "nix-command" "flakes" ];

  # Version d'état NixOS (synchronisée sur le standard Noos 26.05)
  system.stateVersion = "26.05";
}
