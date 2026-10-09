{ config, lib, pkgs, ... }:

let
  cfg = config.hardware.noos-htpc.gpu;
in
{
  imports = [
    ./amd.nix
    ./intel.nix
    ./nvidia.nix
    ./nvidia-legacy.nix
    ./controllers.nix
  ];

  options.hardware.noos-htpc.gpu = {
    profile = lib.mkOption {
      type = lib.types.enum [ "amd" "intel" "nvidia" "nvidia-legacy" "generic" ];
      default = "amd";
      description = ''
        Sélectionne le profil graphique cible pour Noos HTPC.
        - "amd": Pilote amdgpu open-source, Vulkan RADV, support HDR Gamescope/Wayland.
        - "intel": QuickSync VA-API moderne (iHD), OneVPL, optimisé pour les mini-PC NUC/Beelink.
        - "nvidia": Pilotes propriétaires récents (GTX 1650+ / RTX), modesetting et NVDEC.
        - "nvidia-legacy": Pilotes propriétaires legacy 470xx pour anciennes cartes GeForce.
        - "generic": Pilotes Mesa génériques sans spécificité propriétaire.
      '';
    };

    enableHDR = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = ''
        Active les variables d'environnement et drapeaux de compatibilité HDR
        sur les GPU compatibles (principalement AMD et Nvidia récents via Gamescope).
      '';
    };
  };

  config = lib.mkIf (cfg.profile == "generic") {
    services.xserver.videoDrivers = [ "modesetting" ];
    hardware.graphics.enable = true;
  };
}
