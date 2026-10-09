{ config, lib, pkgs, ... }:

let
  sessionCfg = config.services.noos-htpc.desktop;
in
{
  # 1. Gestionnaire de session Greetd configuré pour l'autologin direct TV
  services.greetd = {
    enable = true;
    settings = {
      initial_session = {
        # Démarre immédiatement la session TV sous l'utilisateur 'noos' sans mot de passe
        command = "${pkgs.bash}/bin/bash -l -c 'noos-tv-session'";
        user = "noos";
      };
      default_session = {
        # Relance continue du dashboard TV pour une expérience 100% Smart TV
        command = "${pkgs.bash}/bin/bash -l -c 'noos-tv-session'";
        user = "noos";
      };
    };
  };

  # 2. Sécurité & PAM pour autologin sans mot de passe
  security.pam.services.greetd.enableGnomeKeyring = true;

  # 3. Empêcher la mise en veille et l'extinction d'écran sur TV
  services.displayManager.gdm.autoSuspend = false;
  systemd.targets.sleep.enable = false;
  systemd.targets.suspend.enable = false;
  systemd.targets.hibernate.enable = false;
  systemd.targets.hybrid-sleep.enable = false;
}
