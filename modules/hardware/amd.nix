{ config, lib, pkgs, ... }:

let
  cfg = config.hardware.noos-htpc.gpu;
in
{
  config = lib.mkIf (cfg.profile == "amd") {
    # 1. Modules noyau pour AMDGPU
    boot.initrd.kernelModules = [ "amdgpu" ];
    boot.kernelModules = [ "amdgpu" ];

    # 2. Options noyau pour FreeSync / VRR et affichage TV fluide
    boot.kernelParams = [
      "amdgpu.freesync_video=1"
      "amdgpu.ppfeaturemask=0xffffffff"
    ];

    # 3. Pilote d'affichage X11 / Wayland
    services.xserver.videoDrivers = [ "amdgpu" ];

    # 4. Accélération graphique matérielle (OpenGL & Vulkan RADV)
    hardware.graphics = {
      enable = true;
      enable32Bit = true;
      extraPackages = with pkgs; [
        mesa.drivers
        amdvlk
        vaapiVdpau
        libvdpau-va-gl
      ];
      extraPackages32 = with pkgs.pkgsi686Linux; [
        mesa.drivers
        amdvlk
      ];
    };

    # 5. Variables d'environnement pour RADV, Gamescope et HDR
    environment.variables = {
      AMD_VULKAN_ICD = "RADV";
      RADV_PERFTEST = "aco";
    } // lib.optionalAttrs cfg.enableHDR {
      ENABLE_GAMESCOPE_WSI = "1";
      DXVK_HDR = "1";
      WINE_ENABLE_HDR = "1";
    };

    # Utilitaires de surveillance GPU AMD
    environment.systemPackages = with pkgs; [
      radeontop
      clinfo
      vulkan-tools
    ];
  };
}
