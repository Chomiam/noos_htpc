{ config, pkgs, lib, self ? null, ... }:

let
  installerPkg = pkgs.callPackage ../installer/default.nix { };

  # Script de lancement du mode installateur TV plein écran
  installerSession = pkgs.writeShellScriptBin "noos-iso-session" ''
    export XDG_SESSION_TYPE=wayland
    export GDK_BACKEND=wayland,x11
    export WEBKIT_DISABLE_COMPOSITING_MODE=1
    export WLR_RENDERER_ALLOW_SOFTWARE=1
    export WLR_NO_HARDWARE_CURSORS=1
    export FONTCONFIG_FILE=/etc/fonts/fonts.conf

    echo "[Noos ISO] Démarrage de l'installateur TV Noos HTPC..."

    # Détection et application automatique de la résolution 1920x1080
    (
      for i in $(seq 1 12); do
        sleep 0.5
        if ${pkgs.wlr-randr}/bin/wlr-randr >/dev/null 2>&1; then
          for out in $(${pkgs.wlr-randr}/bin/wlr-randr | grep -E '^[a-zA-Z0-9-]+' | awk '{print $1}'); do
            ${pkgs.wlr-randr}/bin/wlr-randr --output "$out" --mode 1920x1080 || true
          done
          break
        fi
      done
    ) &

    # 1. Tentative avec Gamescope (TV physique avec Vulkan matériel)
    # 2. Si échec (ex: Machine Virtuelle KVM/QEMU), bascule immédiate sur Cage
    if ${pkgs.gamescope}/bin/gamescope -W 1920 -H 1080 -r 60 --fullscreen -- ${installerPkg}/bin/noos-htpc-installer; then
      exit 0
    fi

    echo "[Noos ISO] Gamescope non disponible sur ce matériel, bascule sur Cage (1080p)..."
    exec ${pkgs.cage}/bin/cage -s -- ${installerPkg}/bin/noos-htpc-installer
  '';
in
{
  # 1. Image ISO Live bootable
  image.fileName = lib.mkForce "noos-htpc-installer.iso";
  isoImage.isoName = lib.mkForce "noos-htpc-installer.iso";
  isoImage.volumeID = lib.mkForce "NOOS_HTPC";
  isoImage.makeEfiBootable = true;
  isoImage.makeUsbBootable = true;

  # Clavier AZERTY par défaut dans l'environnement Live
  services.xserver.xkb = {
    layout = "fr";
    variant = "";
  };
  console.keyMap = "fr";
  environment.sessionVariables = {
    XKB_DEFAULT_LAYOUT = "fr";
    XKB_DEFAULT_MODEL = "pc105";
  };

  # 2. Démarrage silencieux et résolution vidéo par défaut 1080p
  boot.kernelParams = [ "video=1920x1080@60" "consoleblank=0" "quiet" "splash" ];

  # 3. Pilotes manettes & Wi-Fi / réseau dans l'ISO
  boot.kernelModules = [ "uinput" "joydev" "r8169" "igc" "e1000e" "iwlwifi" ];
  hardware.xpadneo.enable = true;
  hardware.enableRedistributableFirmware = true;
  services.udev.packages = with pkgs; [ game-devices-udev-rules ];

  # 4. Réseau avec NetworkManager pour le scan Wi-Fi dans l'installateur
  networking.hostName = "noos-installer";
  networking.networkmanager.enable = true;

  # 5. Configuration des polices pour le rendu WebKit de l'installateur
  fonts.fontconfig.enable = true;

  # 6. Session d'installation dédiée avec autologin permanent sans invite de mot de passe
  services.greetd = {
    enable = true;
    settings = {
      initial_session = {
        command = "${pkgs.bash}/bin/bash -l -c '${installerSession}/bin/noos-iso-session'";
        user = "root";
      };
      default_session = {
        command = "${pkgs.bash}/bin/bash -l -c '${installerSession}/bin/noos-iso-session'";
        user = "root";
      };
    };
  };

  # Recommandation ZFS pour éviter les avertissements d'importation
  boot.zfs.forceImportRoot = false;

  # 7. Accès SSH et utilisateurs avec mot de passe simplifié "admin"
  services.openssh = {
    enable = true;
    settings = {
      PermitRootLogin = "yes";
      PasswordAuthentication = true;
    };
  };

  users.users.root = {
    # Mot de passe par défaut : "admin"
    initialHashedPassword = lib.mkForce "$6$B.lDUIXORsmlRbqu$dVZv8gC6LJM5rgi5S4xl5CR9XNnQqrBbXMuQL0uEduSyFg7kijZxYLVtG.fxYBqEOI/6i8dsAGMy7UeYurJvH0";
    openssh.authorizedKeys.keys = [
      "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIAKTtcn0Ok3EGfiP0+00oknZI9SwGw7ael41PfizSeit chomiam@pop-os"
    ];
  };

  users.users.noos = {
    isNormalUser = true;
    description = "Noos HTPC Live User";
    hashedPassword = "$6$B.lDUIXORsmlRbqu$dVZv8gC6LJM5rgi5S4xl5CR9XNnQqrBbXMuQL0uEduSyFg7kijZxYLVtG.fxYBqEOI/6i8dsAGMy7UeYurJvH0";
    extraGroups = [ "wheel" "video" "audio" "input" "networkmanager" ];
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
    wlr-randr
  ];

  # 8. Activation de Flakes
  nix.settings.experimental-features = [ "nix-command" "flakes" ];

  # 9. Intégration des sources Noos HTPC pour l'installation hors-ligne autonome
  environment.etc."noos-htpc-source" = lib.mkIf (self != null) {
    source = self;
  };
}
