{ config, pkgs, lib, self ? null, ... }:

let
  installerPkg = pkgs.callPackage ../installer/default.nix { };

  # Script de lancement du mode installateur TV plein écran
  installerSession = pkgs.writeShellScriptBin "noos-iso-session" ''
    export XDG_SESSION_TYPE=wayland
    export GDK_BACKEND=wayland,x11
    export WEBKIT_DISABLE_COMPOSITING_MODE=1

    echo "[Noos ISO] Lancement de l'installateur TV..."
    
    # 1. Tentative avec Gamescope (TV physique avec Vulkan matériel)
    # 2. Si échec (ex: Machine Virtuelle KVM/QEMU), bascule immédiate sur Cage
    if gamescope -W 1920 -H 1080 -r 60 --fullscreen -- ${installerPkg}/bin/noos-htpc-installer; then
      exit 0
    fi

    echo "[Noos ISO] Gamescope non supporté sur ce matériel, bascule sur Cage..."
    exec ${pkgs.cage}/bin/cage -s -- ${installerPkg}/bin/noos-htpc-installer
  '';
in
{
  # 1. Image ISO Live bootable
  image.fileName = lib.mkForce "noos-htpc-installer.iso";
  isoImage.volumeID = lib.mkForce "NOOS_HTPC";
  isoImage.makeEfiBootable = true;
  isoImage.makeUsbBootable = true;

  # 2. Démarrage silencieux et anti-veille console
  boot.kernelParams = [ "consoleblank=0" "quiet" "splash" ];

  # 3. Pilotes manettes & Wi-Fi / réseau dans l'ISO
  boot.kernelModules = [ "uinput" "joydev" "r8169" "igc" "e1000e" ];
  hardware.xpadneo.enable = true;
  hardware.enableRedistributableFirmware = true;
  services.udev.packages = with pkgs; [ game-devices-udev-rules ];

  # 4. Réseau avec NetworkManager pour le scan Wi-Fi dans l'installateur
  networking.hostName = "noos-installer";
  networking.networkmanager.enable = true;

  # 5. Autologin direct sur la session TV d'installation
  services.greetd = {
    enable = true;
    settings = {
      initial_session = {
        command = "${pkgs.bash}/bin/bash -l -c '${installerSession}/bin/noos-iso-session'";
        user = "root";
      };
      default_session = {
        command = "${pkgs.tuigreet}/bin/tuigreet --time --cmd '${installerSession}/bin/noos-iso-session'";
        user = "root";
      };
    };
  };

  # Recommandation ZFS pour éviter les avertissements d'importation
  boot.zfs.forceImportRoot = false;

  # 6. Accès SSH root automatique pour le contrôle et débogage distant
  services.openssh = {
    enable = true;
    settings.PermitRootLogin = "yes";
  };
  users.users.root = {
    initialHashedPassword = "";
    openssh.authorizedKeys.keys = [
      "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIAKTtcn0Ok3EGfiP0+00oknZI9SwGw7ael41PfizSeit chomiam@pop-os"
    ];
  };

  # 7. Outils d'installation et dépendances dans l'environnement Live
  environment.systemPackages = with pkgs; [
    installerPkg
    installerSession
    parted
    gptfdisk
    dosfstools
    e2fsprogs
    util-linux
    git
    networkmanager
    gamescope
    cage
  ];

  # 7. Activation de Flakes
  nix.settings.experimental-features = [ "nix-command" "flakes" ];
}
