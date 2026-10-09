# Fichier de configuration matérielle standard Noos HTPC.
# Peut être généré/remplacé spécifiquement pour votre machine avec la commande :
# sudo nixos-generate-config --dir /home/chomiam/Projets/noos_htpc/hosts/htpc/

{ config, lib, pkgs, modulesPath, ... }:

{
  imports = [
    (modulesPath + "/installer/scan/not-detected.nix")
  ];

  # Modules noyau essentiels pour mini-PC (NVMe, SATA, USB, stockage)
  boot.initrd.availableKernelModules = [
    "xhci_pci"
    "ahci"
    "nvme"
    "usb_storage"
    "usbhid"
    "sd_mod"
  ];
  boot.initrd.kernelModules = [ ];
  boot.kernelModules = [ "kvm-intel" "kvm-amd" ];
  boot.extraModulePackages = [ ];

  # Configuration générique des systèmes de fichiers (à adapter au partitionnement réel)
  # Par défaut, utilise la détection UEFI / GPT
  fileSystems."/" = lib.mkDefault {
    device = "/dev/disk/by-label/nixos";
    fsType = "ext4";
  };

  fileSystems."/boot" = lib.mkDefault {
    device = "/dev/disk/by-label/boot";
    fsType = "vfat";
    options = [ "fmask=0077" "dmask=0077" ];
  };

  # Détection automatique du nombre de cœurs CPU
  nixpkgs.hostPlatform = lib.mkDefault "x86_64-linux";
}
