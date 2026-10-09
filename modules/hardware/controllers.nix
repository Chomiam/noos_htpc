{ config, lib, pkgs, ... }:

{
  # 1. Activation des modules noyau et pilotes pour manettes
  boot.kernelModules = [ "uinput" "joydev" ];

  # Support étendu pour manettes Xbox One / Series en Bluetooth (gestion batterie, vibrations fines)
  hardware.xpadneo.enable = true;

  # Support Bluetooth haute performance pour manettes sans fil
  hardware.bluetooth = {
    enable = true;
    powerOnBoot = true;
    settings = {
      General = {
        Enable = "Source,Sink,Media,Socket";
        Experimental = true; # Active le niveau de batterie des manettes
        FastConnectable = true; # Reconnexion instantanée des manettes
      };
      Policy = {
        AutoEnable = true;
      };
    };
  };

  # 2. Règles udev universelles pour manettes (Steam, 8BitDo, PlayStation, Xbox, Switch)
  services.udev.packages = with pkgs; [
    game-devices-udev-rules
  ];

  # 3. Règles udev spécifiques Noos HTPC pour permissions /dev/uinput et manettes courantes
  services.udev.extraRules = ''
    # Accès direct à /dev/uinput pour le groupe input
    KERNEL=="uinput", MODE="0660", GROUP="input", OPTIONS+="static_node=uinput"

    # Sony PlayStation DualShock 4 / DualSense PS5
    KERNEL=="hidraw*", ATTRS{idVendor}=="054c", ATTRS{idProduct}=="05c4|09cc|0ce6|0df2", MODE="0666", GROUP="input"

    # Xbox 360 / One / Series (USB & Wireless Dongle)
    KERNEL=="event*", ATTRS{idVendor}=="045e", MODE="0666", GROUP="input"

    # 8BitDo Controllers (Bluetooth, 2.4G & USB - modes X-Input, D-Input, Switch)
    SUBSYSTEM=="input", ATTRS{name}=="*8BitDo*", MODE="0666", GROUP="input"
    KERNEL=="hidraw*", ATTRS{idVendor}=="2dc8", MODE="0666", GROUP="input"

    # Nintendo Switch Pro Controller
    KERNEL=="hidraw*", ATTRS{idVendor}=="057e", ATTRS{idProduct}=="2009", MODE="0666", GROUP="input"
  '';

  # 4. Utilitaires de test et calibration des manettes
  environment.systemPackages = with pkgs; [
    evtest
    jstest-gtk
    linuxConsoleTools # jstest, jscal
  ];
}
