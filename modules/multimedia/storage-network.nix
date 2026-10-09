{ config, lib, pkgs, ... }:

{
  # 1. Gestionnaire de fichiers Thunar adapté avec plugins de volumes et d'archives
  programs.thunar = {
    enable = true;
    plugins = with pkgs; [
      thunar-archive-plugin
      thunar-volman
    ];
  };

  # 2. Automontage USB / Disques durs externes et corbeille via GVFS et UDisks2
  services.gvfs.enable = true;
  services.udisks2.enable = true;
  services.devmon.enable = true; # Démon autonome montant les clés USB instantanément dans /media/

  # 3. Découverte automatique des périphériques réseau et NAS (mDNS / Zeroconf)
  services.avahi = {
    enable = true;
    nssmdns4 = true;
    openFirewall = true;
  };

  # 4. Support client Samba/CIFS et partage réseau Noos HTPC
  services.samba = {
    enable = true;
    openFirewall = true;
    settings = {
      global = {
        "workgroup" = "WORKGROUP";
        "server string" = "Noos HTPC Network Storage";
        "netbios name" = "NOOS-HTPC";
        "security" = "user";
        "map to guest" = "Bad User";
        "guest account" = "noos";
      };
      # Partage public du dossier ROMs pour injection directe depuis un PC distant
      "Noos-Roms" = {
        "path" = "/home/noos/Retro/ROMS";
        "browseable" = "yes";
        "read only" = "no";
        "guest ok" = "yes";
        "create mask" = "0644";
        "directory mask" = "0755";
      };
      # Partage public du dossier Vidéos
      "Noos-Videos" = {
        "path" = "/home/noos/Videos";
        "browseable" = "yes";
        "read only" = "no";
        "guest ok" = "yes";
        "create mask" = "0644";
        "directory mask" = "0755";
      };
    };
  };

  # 5. Serveur OpenSSH & SFTP pour télémaintenance et transfert de fichiers sécurisé
  services.openssh = {
    enable = true;
    settings = {
      PermitRootLogin = "prohibit-password";
      PasswordAuthentication = true;
    };
    openFirewall = true;
  };

  # 6. Paquets utilitaires pour les montages réseau et disques
  environment.systemPackages = with pkgs; [
    cifs-utils    # Outil de montage SMB/CIFS (mount.cifs)
    sshfs         # Montage distant sécurisé SFTP/SSHFS
    samba         # Client smbclient et outils NetBIOS
    file-roller   # Extraction d'archives ZIP/RAR de ROMs en un clic
    tumbler       # Générateur de miniatures d'images/vidéos pour Thunar
  ];
}
