{ config, lib, pkgs, ... }:

let
  cfg = config.hardware.noos-htpc.gpu;
in
{
  config = lib.mkIf (cfg.profile == "nvidia-legacy") {
    # 1. Pilote propriétaire Nvidia Legacy (v470 pour GeForce séries 600/700/800 et GTX antérieures)
    services.xserver.videoDrivers = [ "nvidia" ];

    hardware.nvidia = {
      modesetting.enable = true;
      powerManagement.enable = false;
      open = false;
      nvidiaSettings = true;
      # Sélecteur de version legacy : 470xx par défaut
      package = config.boot.kernelPackages.nvidiaPackages.legacy_470;
    };

    # 2. Accélération graphique matérielle et VDPAU
    hardware.graphics = {
      enable = true;
      enable32Bit = true;
      extraPackages = with pkgs; [
        libva-vdpau-driver
        libvdpau-va-gl
      ];
    };

    # 3. Variables d'environnement optimisées pour les cartes legacy
    environment.variables = {
      VDPAU_DRIVER = "nvidia";
      __GLX_VENDOR_LIBRARY_NAME = "nvidia";
    };

    environment.systemPackages = with pkgs; [
      vulkan-tools
    ];
  };
}
