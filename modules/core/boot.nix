{ config, lib, pkgs, ... }:

let
  grubTheme = ../../themes/grub-theme;
in
{
  # 1. Chargeur d'amorçage GRUB moderne et stylisé Noos HTPC (Thème Catppuccin Mocha)
  boot.loader = {
    systemd-boot.enable = lib.mkForce false;
    grub = {
      enable = true;
      efiSupport = true;
      efiInstallAsRemovable = false;
      device = "nodev";
      configurationLimit = 15; # Conserve les 15 dernières générations pour un rollback instantané
      configurationName = "Noos-HTPC";
      theme = grubTheme;
      splashImage = null; # Intégré dans background.png du thème
      gfxmodeEfi = "1920x1080,auto";
      entryOptions = "--class noos-htpc --class gnu-linux --class os";
      subEntryOptions = "--class noos-htpc --class gnu-linux --class os";

      # Copie de sauvegarde EFI universelle (BOOTX64.EFI) en cas de réinitialisation NVRAM
      extraInstallCommands = ''
        ${pkgs.coreutils}/bin/mkdir -p /boot/EFI/BOOT
        ${pkgs.coreutils}/bin/cp -f /boot/EFI/*/grubx64.efi /boot/EFI/BOOT/BOOTX64.EFI || true
      '';
    };
    efi = {
      canTouchEfiVariables = true;
      efiSysMountPoint = "/boot";
    };
    timeout = 3; # 3 secondes d'affichage du menu boot TV avant démarrage automatique
  };

  # 2. Démarrage silencieux "Console Style" (aucun log textuel au démarrage sur la TV)
  boot.consoleLogLevel = 0;
  boot.initrd.verbose = false;
  boot.kernelParams = [
    "quiet"
    "splash"
    "loglevel=3"
    "udev.log_priority=3"
    "vt.global_cursor_default=0"
    "systemd.show_status=false"
    "rd.systemd.show_status=false"
    "rd.udev.log_level=3"
  ];

  # 3. Écran de démarrage Plymouth avec le logo officiel Noos HTPC
  boot.plymouth = {
    enable = true;
    logo = ../../logo.png;
    theme = "breeze";
  };

  # Gestion des crashs et redémarrage automatique en cas de panique noyau
  boot.kernel.sysctl = {
    "kernel.panic" = 10;
  };
}
