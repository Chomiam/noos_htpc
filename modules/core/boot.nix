{ config, lib, pkgs, ... }:

{
  # 1. Chargeur d'amorçage UEFI moderne avec historique des générations (Rollback natif)
  boot.loader = {
    systemd-boot = {
      enable = true;
      configurationLimit = 15; # Conserve les 15 dernières générations pour un rollback instantané
      editor = false;          # Sécurisation contre l'édition manuelle sur TV
      consoleMode = "max";
    };
    efi = {
      canTouchEfiVariables = true;
    };
    timeout = 2; # 2 secondes d'affichage du menu boot avant démarrage automatique
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
