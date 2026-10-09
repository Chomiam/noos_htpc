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

    # Configuration optimisée pour HTPC / TV (Qualité audiophile, fréquences cinéma et Bitstream Passthrough)
    extraConfig.pipewire = {
      "10-clock-rates" = {
        "context.properties" = {
          "default.clock.rate" = 48000;
          "default.clock.allowed-rates" = [ 44100 48000 88200 96000 176400 192000 ];
        };
      };
    };

    # Support du Passthrough numérique brut (IEC61937) pour amplificateurs Home-Cinéma
    extraConfig.pipewire-pulse = {
      "10-passthrough" = {
        "pulse.properties" = {
          "pulse.default.format" = "F32";
          "pulse.default.position" = [ "FL" "FR" ];
        };
        "pulse.rules" = [
          {
            matches = [ { "application.process.binary" = "mpv"; } ];
            actions = {
              update-props = {
                "audio.format" = "passthrough";
              };
            };
          }
        ];
      };
    };

    # Configuration WirePlumber : maintien du lien HDMI actif (zéro craquement / resynchronisation ampli)
    wireplumber.extraConfig = {
      "10-disable-suspension" = {
        "monitor.alsa.rules" = [
          {
            matches = [
              { "device.name" = "~alsa_card.*"; }
              { "node.name" = "~alsa_output.*"; }
            ];
            actions = {
              update-props = {
                "audio.allowed-rates" = "44100,48000,88200,96000,176400,192000";
                "session.suspend-timeout-seconds" = 0; # Maintient l'amplificateur synchronisé sans coupure
              };
            };
          }
        ];
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
