{ config, lib, pkgs, ... }:

let
  cfg = config.hardware.noos-htpc.gpu;
in
{
  config = lib.mkIf (cfg.profile == "nvidia") {
    # 1. Pilote propriétaire Nvidia
    services.xserver.videoDrivers = [ "nvidia" ];

    # 2. Paramètres matériels Nvidia modernes (Turing, Ampere, Ada Lovelace...)
    hardware.nvidia = {
      modesetting.enable = true;
      powerManagement.enable = true;
      powerManagement.finegrained = false;
      open = false; # Propriétaire recommandé pour la stabilité maximale sur GPU desktop/mini-PC
      nvidiaSettings = true;
      package = config.boot.kernelPackages.nvidiaPackages.stable;
    };

    # 3. Accélération matérielle graphique et VA-API via NVDEC
    hardware.graphics = {
      enable = true;
      enable32Bit = true;
      extraPackages = with pkgs; [
        nvidia-vaapi-driver
        libva-vdpau-driver
        libvdpau-va-gl
      ];
    };

    # 4. Variables d'environnement pour Wayland, Gamescope et accélération matérielle
    environment.variables = {
      LIBVA_DRIVER_NAME = "nvidia";
      GBM_BACKEND = "nvidia-drm";
      __GLX_VENDOR_LIBRARY_NAME = "nvidia";
      NVD_BACKEND = "direct"; # Amélioration pour nvidia-vaapi-driver
    } // lib.optionalAttrs cfg.enableHDR {
      ENABLE_GAMESCOPE_WSI = "1";
      DXVK_HDR = "1";
    };

    # Utilitaires de surveillance Nvidia
    environment.systemPackages = with pkgs; [
      nvtopPackages.nvidia
      vulkan-tools
    ];
  };
}
