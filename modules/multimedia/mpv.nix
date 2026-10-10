{ config, lib, pkgs, ... }:

let
  gpuCfg = config.hardware.noos-htpc.gpu;

  # mpv.conf complet et hautement optimisé pour l'affichage TV
  mpvConfig = ''
    # ==============================================================================
    # Configuration MPV Universelle - Noos HTPC
    # Optimisé pour grand écran TV, accélération matérielle et audio passthrough
    # ==============================================================================

    # 1. Rendu vidéo moderne et accélération matérielle
    vo=gpu-next
    gpu-api=vulkan,opengl
    hwdec=auto-safe

    # 1.1 Interface OSD UOSC (Désactivation de l'ancien OSC intégré mpv)
    osc=no
    osd-bar=no
    border=no

    # 1.2 Passthrough HDR dynamique & Tonemapping intelligent
    target-colorspace-hint=auto
    gamut-mapping-mode=auto

    # 2. Fluidité et élimination du judder (24Hz sur écran 60Hz/120Hz)
    video-sync=display-resample
    interpolation=yes
    tscale=oversample

    # 3. Audio Bitperfect Passthrough (Amplificateurs & Barres de son Home-Cinéma)
    ao=pipewire,alsa,pulse
    audio-spdif=ac3,dts,eac3,truehd,dts-hd
    audio-channels=auto
    audio-pitch-correction=no
    volume-max=100

    # 4. Sous-titres adaptés pour grand écran TV
    sub-auto=fuzzy
    sub-font='Sans-Serif'
    sub-font-size=48
    sub-border-size=3
    sub-color='#FFE500'
    sub-border-color='#000000'
    sub-shadow-offset=2
    sub-shadow-color='#101010'
    sub-pos=95

    # 5. Ergonomie TV et télécommande / manette
    fullscreen=yes
    keep-open=yes
    cursor-autohide=1000
    osd-font-size=36
    osd-duration=2000

    # ==============================================================================
    # PROFILS D'UPSCALING, TONE-MAPPING & AUDIO BITPERFECT
    # ==============================================================================

    # Profil Bitperfect Passthrough : activé dès qu'un flux surround Dolby/DTS est détecté
    [audio-bitperfect-passthrough]
    profile-cond=p["audio-codec-name"] == "ac3" or p["audio-codec-name"] == "eac3" or p["audio-codec-name"] == "truehd" or p["audio-codec-name"] == "dts" or p["audio-codec-name"] == "dts-hd" or p["audio-codec-name"] == "dca"
    profile-restore=copy
    video-sync=audio
    interpolation=no
    audio-pitch-correction=no

    # Profil 0 : Repli Tonemapping Automatique HDR vers SDR (libplacebo)
    [hdr-to-sdr-fallback]
    profile-cond=p["video-params/primaries"] == "bt.2020" or p["video-params/gamma"] == "pq" or p["video-params/gamma"] == "hlg"
    profile-restore=copy
    tone-mapping=spline
    hdr-compute-peak=yes
    hdr-contrast-recovery=0.5
    gamut-mapping-mode=perceptual
    target-peak=100

    # Profil 1 : Upscaling léger pour iGPU (Intel NUC, Beelink, AMD APU Vega/RDNA)
    [iGPU-light-upscale]
    scale=spline36
    cscale=spline36
    dscale=mitchell
    correct-downscaling=yes
    linear-downscaling=yes
    deband=yes
    deband-iterations=2
    deband-threshold=35
    deband-range=16
    deband-grain=5

    # Profil 2 : Upscaling lourd & Shaders pour GPU dédiés (AMD RX, Nvidia GTX/RTX)
    [dGPU-high-upscale]
    scale=ewa_lanczossharp
    scale-blur=0.981251
    cscale=spline36
    dscale=mitchell
    correct-downscaling=yes
    linear-downscaling=yes
    deband=yes
    deband-iterations=4
    deband-threshold=48
    deband-range=24
    deband-grain=16

    # Profil 3 : Contenu 4K Natif (Désactive les filtres inutiles pour économiser le GPU)
    [4k-native]
    profile-cond=width >= 3840 or height >= 2160
    scale=bilinear
    cscale=bilinear
    deband=no

    # Profil 4 : Détection automatique des contenus 720p / 1080p
    [auto-upscale]
    profile-cond=(width < 3840 and height < 2160) and (width >= 1280 or height >= 720)
    ${if (gpuCfg.profile == "intel" || gpuCfg.profile == "generic") then "profile=iGPU-light-upscale" else "profile=dGPU-high-upscale"}
  '';

  # Configuration Thème Catppuccin Mocha & Ergonomie TV 10-Foot pour UOSC
  uoscConfig = ''
    # Thème Catppuccin Mocha & Ergonomie TV 10-Foot pour Noos HTPC
    timeline_style=bar
    timeline_size=38
    timeline_line_width=3
    timeline_border=1
    timeline_cache=true

    # Barre de contrôles de lecture moderne
    controls=menu,gap,subtitles,audio,video,stream-quality,gap,space,speed,gap,prev,items,play-pause,next,gap,space,fullscreen
    controls_size=36
    controls_margin=16
    controls_spacing=10

    # Audio et volume
    volume=right
    volume_size=40
    volume_border=1

    # Menus et popups grand écran
    menu_item_height=42
    menu_min_width=340
    menu_padding=6
    menu_type_to_search=false

    # Barre supérieure de titre et infos
    top_bar=no-border
    top_bar_size=42
    top_bar_controls=right
    top_bar_title=yes

    # Échelle pour grand écran TV 4K / 1080p
    scale=1.1
    scale_fullscreen=1.4
    font_bold=yes
    border_radius=8

    # Couleurs Thème Catppuccin Mocha
    color=foreground=89b4fa,foreground_text=11111b,background=181825,background_text=cdd6f4,curtain=11111b,success=a6e3a1,error=f38ba8,match=89b4fa,heatmap=cba6f7
    opacity=timeline=0.95,position=1,chapters=0.8,slider=0.9,slider_gauge=1,controls=0.95,speed=0.6,menu=1,submenu=0.5,border=1,title=1,tooltip=1,thumbnail=1,curtain=0.85,idle_indicator=0.8,audio_indicator=0.8,buffering_indicator=0.8,playlist_position=0.8,heatmap=0.4

    # Indicateur de pause discret
    pause_indicator=flash
    destination_time=playtime-remaining
  '';

  # Fichier input.conf pour le contrôle à la manette et télécommande
  mpvInput = ''
    # ==============================================================================
    # BINDINGS MPV & CONTRÔLEUR TV / GAMEPAD - NOOS HTPC
    # ==============================================================================

    # Manette de jeu (Gamepad Xbox / PlayStation / Universel)
    GAMEPAD_DPAD_UP add volume 2
    GAMEPAD_DPAD_DOWN add volume -2
    GAMEPAD_DPAD_LEFT seek -10
    GAMEPAD_DPAD_RIGHT seek 10
    GAMEPAD_A cycle pause
    GAMEPAD_B script-binding uosc/menu-back
    GAMEPAD_X script-binding uosc/subtitles
    GAMEPAD_Y script-binding uosc/audio
    GAMEPAD_START script-binding uosc/menu
    GAMEPAD_BACK script-binding stats/display-stats-toggle
    GAMEPAD_LEFT_SHOULDER seek -60
    GAMEPAD_RIGHT_SHOULDER seek 60

    # Télécommande TV / Clavier standard
    SPACE cycle pause
    PLAYPAUSE cycle pause
    PLAY set pause no
    PAUSE set pause yes
    UP add volume 2
    DOWN add volume -2
    RIGHT seek 10
    LEFT seek -10
    Shift+RIGHT seek 60
    Shift+LEFT seek -60
    m cycle mute
    s script-binding uosc/subtitles
    a script-binding uosc/audio
    v script-binding uosc/stream-quality
    i script-binding stats/display-stats-toggle
    TAB script-binding uosc/toggle-ui
    MENU script-binding uosc/menu
    ENTER script-binding uosc/menu
    ESC script-binding uosc/menu-back
    q quit

    # ==============================================================================
    # MENUS UOSC DYNAMIQUES (Affichés dans le menu 'm' / Bouton START manette)
    # ==============================================================================

    #! Sous-titres : Pistes et Synchronisation
    s script-binding uosc/subtitles

    #! Audio : Pistes sonores & Passthrough Dolby/DTS
    a script-binding uosc/audio

    #! Qualité et Débit du flux vidéo
    v script-binding uosc/stream-quality

    #! Informations de lecture en direct (Codecs, HDR, Débit, Passthrough)
    i script-binding stats/display-stats-toggle

    #! Shaders & Technologies d'Upscaling > 1. Désactivé (Natif)
    _ change-list glsl-shaders clr all; set scale bilinear; set cscale bilinear; show-text "Upscaling : Désactivé (Natif)"

    #! Shaders & Technologies d'Upscaling > 2. AMD FSR (FidelityFX Super Resolution)
    _ change-list glsl-shaders clr all; change-list glsl-shaders append "/etc/mpv/shaders/FSR.glsl"; show-text "Upscaling : AMD FSR activé"

    #! Shaders & Technologies d'Upscaling > 3. AMD CAS (Contrast Adaptive Sharpening)
    _ change-list glsl-shaders clr all; change-list glsl-shaders append "/etc/mpv/shaders/CAS-scaled.glsl"; show-text "Upscaling : AMD CAS activé"

    #! Shaders & Technologies d'Upscaling > 4. FSRCNNX IA Léger (8-0-4-1)
    _ change-list glsl-shaders clr all; change-list glsl-shaders append "/etc/mpv/shaders/FSRCNNX_x2_8-0-4-1.glsl"; show-text "Upscaling : FSRCNNX IA Léger activé"

    #! Shaders & Technologies d'Upscaling > 5. FSRCNNX IA Ultra (16-0-4-1)
    _ change-list glsl-shaders clr all; change-list glsl-shaders append "/etc/mpv/shaders/FSRCNNX_x2_16-0-4-1.glsl"; show-text "Upscaling : FSRCNNX IA Ultra activé"

    #! Shaders & Technologies d'Upscaling > 6. Anime4K (Spécial Animation & Dessins Animés)
    _ change-list glsl-shaders clr all; change-list glsl-shaders append "/etc/mpv/shaders/Anime4K_Upscale_CNN_x2_M.glsl"; change-list glsl-shaders append "/etc/mpv/shaders/Anime4K_Restore_CNN_M.glsl"; show-text "Upscaling : Anime4K activé"

    #! Shaders & Technologies d'Upscaling > 7. NNEDI3 64 Neurones (Interpolation de contours)
    _ change-list glsl-shaders clr all; change-list glsl-shaders append "/etc/mpv/shaders/nnedi3-nns64-win8x6.hook"; show-text "Upscaling : NNEDI3 64 activé"

    #! Shaders & Technologies d'Upscaling > 8. KrigBilateral (Reconstruction Chroma 4:4:4)
    _ change-list glsl-shaders append "/etc/mpv/shaders/KrigBilateral.glsl"; show-text "Chroma : KrigBilateral activé"

    #! Shaders & Technologies d'Upscaling > 9. Nvidia Image Scaler (NVScaler)
    _ change-list glsl-shaders clr all; change-list glsl-shaders append "/etc/mpv/shaders/NVScaler.glsl"; show-text "Upscaling : Nvidia NVScaler activé"
  '';
in
{
  # 1. Installation de MPV avec support de tous les codecs et scripts
  environment.systemPackages = with pkgs; [
    (mpv.override {
      scripts = with pkgs.mpvScripts; [
        mpris
        uosc          # Interface OSD élégante pour TV
        thumbfast     # Vignettes de prévisualisation rapides
      ];
    })
    yt-dlp          # Support du streaming et liens web dans MPV
    ffmpeg-full     # Pile universelle de décodage/encodage (AV1, HEVC, DTS, Dolby)
    mpv-shim-default-shaders # Shaders FSR, CAS, FSRCNNX, NIS, NNEDI3, KrigBilateral
  ];

  # 2. Déploiement de la configuration déclarative globale /etc/mpv/
  environment.etc."mpv/mpv.conf".text = mpvConfig;
  environment.etc."mpv/input.conf".text = mpvInput;
  environment.etc."mpv/script-opts/uosc.conf".text = uoscConfig;
  environment.etc."mpv/shaders".source = "${pkgs.mpv-shim-default-shaders}/share/mpv-shim-default-shaders/shaders";

  # 3. Liens symboliques utilisateurs pour Jellyfin et MPV
  systemd.tmpfiles.rules = [
    "d /home/noos/.config/mpv 0755 noos users -"
    "d /home/noos/.config/mpv/script-opts 0755 noos users -"
    "L+ /home/noos/.config/mpv/script-opts/uosc.conf - - - - /etc/mpv/script-opts/uosc.conf"
    "L+ /home/noos/.config/mpv/input.conf - - - - /etc/mpv/input.conf"
    "L+ /home/noos/.config/mpv/mpv.conf - - - - /etc/mpv/mpv.conf"
    "L+ /home/noos/.config/mpv/shaders - - - - /etc/mpv/shaders"
    "d /home/noos/.config/jellyfin-media-player 0755 noos users -"
    "L+ /home/noos/.config/jellyfin-media-player/shaders - - - - /etc/mpv/shaders"
  ];
}
