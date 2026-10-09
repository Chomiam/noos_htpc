{ config, lib, pkgs, ... }:

let
  # Configuration par défaut pour EmulationStation-DE (ES-DE)
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

  # Wrapper ES-DE polyvalent & résilient
  esDeWrapper = pkgs.writeShellScriptBin "es-de" ''
    set -euo pipefail

    # 1. Détection prioritaire d'une AppImage ES-DE présente dans /home/noos/Retro/
    USER_APPIMAGE=$(find /home/noos/Retro/ -maxdepth 2 -name "*ES-DE*.AppImage" 2>/dev/null | head -n 1 || true)
    if [ -n "$USER_APPIMAGE" ]; then
      chmod +x "$USER_APPIMAGE" || true
      echo "[Noos Gaming] Lancement de l'AppImage officielle ES-DE : $USER_APPIMAGE"
      exec "$USER_APPIMAGE" "$@"
    fi

    # 2. Utilisation de Pegasus Frontend TV (inclus et packagé nativement dans NixOS)
    if command -v pegasus-frontend >/dev/null 2>&1; then
      echo "[Noos Gaming] Lancement de Pegasus Frontend TV..."
      exec pegasus-frontend "$@"
    fi

    # 3. Secours : RetroArch avec menu TV Ozone
    exec retroarch "$@"
  '';
in
{
  # 1. Installation de Pegasus Frontend et du wrapper ES-DE
  environment.systemPackages = with pkgs; [
    pegasus-frontend
    esDeWrapper
  ];

  # 2. Déploiement automatique du fichier de paramètres ES-DE pour l'utilisateur noos
  systemd.tmpfiles.rules = [
    "d /home/noos/.emulationstation 0755 noos users -"
    "C+ /home/noos/.emulationstation/es_settings.xml 0644 noos users - ${pkgs.writeText "es_settings.xml" esSettingsXml}"
  ];

  # 3. Raccourci .desktop pour lancer l'interface TV
  environment.etc."xdg/autostart/es-de.desktop".text = ''
    [Desktop Entry]
    Type=Application
    Name=Frontend Rétro-Gaming TV
    Comment=Interface TV pour jeux rétro et émulateurs
    Exec=es-de
    Icon=es-de
    Categories=Game;Emulator;
  '';
}
