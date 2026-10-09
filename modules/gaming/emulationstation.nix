{ config, lib, pkgs, ... }:

let
  # Configuration par défaut d'EmulationStation-DE (ES-DE)
  esSettingsXml = ''
    <?xml version="1.0"?>
    <config>
        <string name="ROMDirectory" value="/home/noos/Retro/ROMS" />
        <bool name="Fullscreen" value="true" />
        <bool name="HideTaskbar" value="true" />
        <bool name="ShowHelpPrompts" value="true" />
        <string name="UIMode" value="Full" />
        <string name="TransitionStyle" value="fade" />
        <string name="GamelistViewStyle" value="automatic" />
        <bool name="ScrapeRatings" value="true" />
        <bool name="DrawFramerate" value="false" />
        <int name="ScreenSaverTime" value="300000" />
    </config>
  '';
in
{
  # 1. Installation du paquet officiel ES-DE
  environment.systemPackages = with pkgs; [
    es-de
  ];

  # 2. Déploiement automatique du fichier de paramètres ES-DE pour l'utilisateur noos
  systemd.tmpfiles.rules = [
    "d /home/noos/.emulationstation 0755 noos users -"
    "C+ /home/noos/.emulationstation/es_settings.xml 0644 noos users - ${pkgs.writeText "es_settings.xml" esSettingsXml}"
  ];

  # 3. Raccourci .desktop pour lancer ES-DE
  environment.etc."xdg/autostart/es-de.desktop".text = ''
    [Desktop Entry]
    Type=Application
    Name=EmulationStation DE
    Comment=Frontend TV pour jeux rétro et émulateurs
    Exec=es-de
    Icon=es-de
    Categories=Game;Emulator;
  '';
}
