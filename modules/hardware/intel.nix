{ config, lib, pkgs, ... }:

let
  cfg = config.hardware.noos-htpc.gpu;
in
{
  config = lib.mkIf (cfg.profile == "intel") {
    # 1. Modules noyau pour Intel iGPU
    boot.initrd.kernelModules = [ "i915" ];
    boot.kernelModules = [ "i915" ];

    # Optimisation du pilote i915 pour l'économie d'énergie et l'accélération vidéo
    boot.kernelParams = [
      "i915.enable_guc=3"
      "i915.enable_fbc=1"
    ];

    # 2. Pilote X11 / Wayland
    services.xserver.videoDrivers = [ "modesetting" ];

    # 3. Accélération matérielle QuickSync / VA-API (Intel Media Driver iHD)
    hardware.graphics = {
      enable = true;
      enable32Bit = true;
      extraPackages = with pkgs; [
        intel-media-driver      # VA-API moderne (iHD) pour Broadwell+ (Gen8+)
        intel-vaapi-driver      # VA-API legacy (i965)
        libvdpau-va-gl          # Backend VDPAU basé sur VA-API
        intel-compute-runtime   # OpenCL pour Intel HD/UHD/Iris/Arc
        vpl-gpu-rt              # OneVPL runtime pour encodage/décodage moderne
      ];
      extraPackages32 = with pkgs.pkgsi686Linux; [
        intel-media-driver
        intel-vaapi-driver
      ];
    };

    # 4. Variables d'environnement pour forcer le pilote VA-API iHD performant
    environment.variables = {
      LIBVA_DRIVER_NAME = "iHD";
      VDPAU_DRIVER = "va_gl";
    };

    # Utilitaires de surveillance et validation Intel VA-API
    environment.systemPackages = with pkgs; [
      intel-gpu-tools
      libva-utils
      vulkan-tools
    ];
  };
}
