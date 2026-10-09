{ config, lib, pkgs, ... }:

{
  # 1. Désactivation de PulseAudio classique en faveur de PipeWire
  services.pulseaudio.enable = false;

  # 2. PipeWire comme serveur audio temps réel basse latence
  security.rtkit.enable = true; # Priorité temps réel pour éviter tout stuttering audio
  services.pipewire = {
    enable = true;
    alsa.enable = true;
    alsa.support32Bit = true;
    pulse.enable = true;
    jack.enable = true;
    wireplumber.enable = true;

    # Configuration optimisée pour HTPC / TV (qualité et bitstream)
    extraConfig.pipewire = {
      "10-clock-rates" = {
        "context.properties" = {
          "default.clock.rate" = 48000;
          "default.clock.allowed-rates" = [ 44100 48000 88200 96000 192000 ];
        };
      };
    };
  };

  # 3. Outils audio et gestion de mixage
  environment.systemPackages = with pkgs; [
    pavucontrol
    alsa-utils
    wireplumber
  ];
}
