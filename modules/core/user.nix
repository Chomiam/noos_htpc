{ config, lib, pkgs, ... }:

{
  # 1. Définition de l'utilisateur dédié "noos"
  users.users.noos = {
    isNormalUser = true;
    description = "Noos HTPC Console User";
    # Mot de passe par défaut chiffré : "admin" (généré via SHA-512 crypt)
    hashedPassword = "$6$B.lDUIXORsmlRbqu$dVZv8gC6LJM5rgi5S4xl5CR9XNnQqrBbXMuQL0uEduSyFg7kijZxYLVtG.fxYBqEOI/6i8dsAGMy7UeYurJvH0";
    extraGroups = [
      "wheel"          # Administration système (sudo)
      "video"          # Accès matériel direct GPU
      "audio"          # Accès audio PipeWire/ALSA
      "input"          # Manettes, télécommandes, périphériques d'entrée
      "render"         # Accès aux nœuds DRM de rendu matériel (DRI)
      "networkmanager" # Gestion réseau sans mot de passe
      "storage"        # Disques et stockage externe
      "gamemode"       # Priorité temps réel GameMode
      "disk"           # Accès stockage
      "cdrom"          # Accès direct aux lecteurs optiques DVD/Blu-ray (SATA & USB)
    ];
    home = "/home/noos";
    createHome = true;
    openssh.authorizedKeys.keys = [
      "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIAKTtcn0Ok3EGfiP0+00oknZI9SwGw7ael41PfizSeit chomiam@pop-os"
    ];
  };

  users.users.root = {
    initialHashedPassword = lib.mkDefault "$6$B.lDUIXORsmlRbqu$dVZv8gC6LJM5rgi5S4xl5CR9XNnQqrBbXMuQL0uEduSyFg7kijZxYLVtG.fxYBqEOI/6i8dsAGMy7UeYurJvH0";
    openssh.authorizedKeys.keys = [
      "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIAKTtcn0Ok3EGfiP0+00oknZI9SwGw7ael41PfizSeit chomiam@pop-os"
    ];
  };

  # 2. Droits sudo sans mot de passe pour l'utilisateur noos (usage TV / Console)
  security.sudo = {
    enable = true;
    wheelNeedsPassword = false;
    extraRules = [
      {
        users = [ "noos" ];
        commands = [
          {
            command = "ALL";
            options = [ "NOPASSWD" ];
          }
        ];
      }
    ];
  };

  # 3. Règles Polkit : Extinction, redémarrage et gestion système rapides sans mot de passe
  security.polkit.enable = true;
  security.polkit.extraConfig = ''
    polkit.addRule(function(action, subject) {
      if (
        subject.isInGroup("wheel") &&
        (
          action.id.indexOf("org.freedesktop.login1.") === 0 ||
          action.id.indexOf("org.freedesktop.udisks2.") === 0 ||
          action.id.indexOf("org.freedesktop.NetworkManager.") === 0 ||
          action.id.indexOf("org.freedesktop.systemd1.manage-units") === 0
        )
      ) {
        return polkit.Result.YES;
      }
    });
  '';

  # 4. Structure déclarative des dossiers utilisateur au premier démarrage
  systemd.tmpfiles.rules = [
    "d /home/noos/Retro 0755 noos users -"
    "d /home/noos/Retro/ROMS 0755 noos users -"
    "d /home/noos/Retro/ROMS/nes 0755 noos users -"
    "d /home/noos/Retro/ROMS/snes 0755 noos users -"
    "d /home/noos/Retro/ROMS/megadrive 0755 noos users -"
    "d /home/noos/Retro/ROMS/psx 0755 noos users -"
    "d /home/noos/Retro/ROMS/ps2 0755 noos users -"
    "d /home/noos/Retro/ROMS/gc 0755 noos users -"
    "d /home/noos/Retro/ROMS/arcade 0755 noos users -"
    "d /home/noos/Retro/BIOS 0755 noos users -"
    "d /home/noos/Videos 0755 noos users -"
    "d /home/noos/IPTV 0755 noos users -"
    "d /home/noos/Partages 0755 noos users -"
  ];
}
