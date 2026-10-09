{ config, lib, pkgs, ... }:

let
  # Définition de RetroArch avec sa suite complète de cores émulés
  retroarchWithCores = pkgs.retroarch.override {
    cores = with pkgs.libretro; [
      # Nintendo
      nestopia        # NES
      snes9x          # Super Nintendo
      mgba            # Game Boy / Color / Advance
      mupen64plus     # Nintendo 64

      # Sega
      genesis-plus-gx # Master System / Mega Drive / Genesis / Game Gear
      beetle-saturn   # Sega Saturn

      # Sony
      beetle-psx-hw   # PlayStation 1 avec rendu matériel Vulkan/OpenGL
      pcsx-rearmed    # PlayStation 1 optimisé basse consommation

      # Arcade & Divers
      fbneo           # NeoGeo, CPS1/2/3, Arcade
      dosbox-pure     # Jeux rétro PC / DOS
    ];
  };

  # Configuration déclarative TV pour RetroArch
  retroarchConfig = ''
    # Dossiers système mappés sur /home/noos/Retro
    system_directory = "/home/noos/Retro/BIOS"
    rgui_browser_directory = "/home/noos/Retro/ROMS"
    savefile_directory = "/home/noos/Retro/Saves"
    savestate_directory = "/home/noos/Retro/States"
    screenshots_directory = "/home/noos/Retro/Screenshots"

    # Moteur vidéo moderne et affichage TV
    video_driver = "vulkan"
    video_fullscreen = "true"
    video_vsync = "true"
    video_hard_sync = "true"
    video_hard_sync_frames = "0"
    video_frame_delay = "0"

    # Moteur audio PipeWire
    audio_driver = "pipewire"
    audio_enable = "true"
    audio_sync = "true"

    # Interface graphique TV type console moderne
    menu_driver = "ozone"
    ozone_menu_color_theme = "1"
    menu_linear_filter = "true"
    menu_enable_widgets = "true"

    # Manettes et réactivité
    input_autodetect_enable = "true"
    input_enable_hotkey = "true"
    input_hotkey_block_keys = "true"
    # Combinaison Quitter jeu : Select + Start
    input_exit_emulator_btn = "7"
    input_enable_hotkey_btn = "6"
    # Sauvegarde / Chargement rapide
    input_save_state_btn = "10"
    input_load_state_btn = "9"
  '';
in
{
  # 1. Installation de RetroArch et des émulateurs 3D standalone lourds (PS2, GameCube)
  environment.systemPackages = [
    retroarchWithCores
    pkgs.pcsx2           # Émulateur PlayStation 2 standalone hautement performant
    pkgs.dolphin-emu     # Émulateur GameCube / Wii avec support Wiimote & manettes
  ];

  # 2. Injection du fichier de configuration global
  environment.etc."retroarch.cfg".text = retroarchConfig;
}
