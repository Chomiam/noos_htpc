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

  # 3. Découverte automatique des périphériques réseau et NAS (mDNS / Zeroconf / Avahi)
  services.avahi = {
    enable = true;
    nssmdns4 = true;
    openFirewall = true;
    publish = {
      enable = true;
      addresses = true;
      userServices = true;
      workstation = true;
    };
  };

  # 4. Découverte réseau Windows 10/11 native (WSDD - Web Services Dynamic Discovery)
  services.samba-wsdd = {
    enable = true;
    openFirewall = true;
  };

  # 5. Serveur et client Samba/CIFS : Partage complet de /home sur le réseau local
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
      # Partage de l'intégralité du /home sur le réseau local (accessible sans mot de passe ou via utilisateur noos)
      "Home" = {
        "path" = "/home";
        "browseable" = "yes";
        "read only" = "no";
        "guest ok" = "yes";
        "create mask" = "0775";
        "directory mask" = "0775";
        "force user" = "noos";
      };
      # Partage direct de l'espace utilisateur noos
      "noos" = {
        "path" = "/home/noos";
        "browseable" = "yes";
        "read only" = "no";
        "guest ok" = "yes";
        "create mask" = "0775";
        "directory mask" = "0775";
        "force user" = "noos";
      };
      # Partage public du dossier ROMs pour injection directe depuis un PC distant
      "Noos-Roms" = {
        "path" = "/home/noos/Retro/ROMS";
        "browseable" = "yes";
        "read only" = "no";
        "guest ok" = "yes";
        "create mask" = "0775";
        "directory mask" = "0775";
        "force user" = "noos";
      };
      # Partage public du dossier Vidéos
      "Noos-Videos" = {
        "path" = "/home/noos/Videos";
        "browseable" = "yes";
        "read only" = "no";
        "guest ok" = "yes";
        "create mask" = "0775";
        "directory mask" = "0775";
        "force user" = "noos";
      };
    };
  };

  # 6. Serveur OpenSSH & SFTP : Partage et télémaintenance sécurisée de /home
  services.openssh = {
    enable = true;
    settings = {
      PermitRootLogin = "yes";
      PasswordAuthentication = true;
      KbdInteractiveAuthentication = true;
    };
    openFirewall = true;
  };

  # 7. Paquets indispensables pour le partage réseau, Samba, SSH/SFTP et la découverte
  environment.systemPackages = with pkgs; [
    cifs-utils       # Outil de montage SMB/CIFS (mount.cifs)
    sshfs            # Montage distant sécurisé SFTP/SSHFS
    samba            # Client smbclient, nmblookup, testparm
    wsdd             # Découverte réseau Windows
    avahi            # mDNS, avahi-browse, avahi-resolve
    nfs-utils        # Montage et client NFS
    rsync            # Synchronisation rapide de données
    curl             # Transfert HTTP/FTP
    wget             # Téléchargement réseau
    inetutils        # Outils réseau (ping, traceroute, hostname)
    file-roller      # Extraction d'archives ZIP/RAR de ROMs en un clic
  ];
}
