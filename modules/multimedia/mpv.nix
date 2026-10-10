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

    # 5. Contrôles à la manette et télécommande (SDL2 Gamepad activé)
    input-gamepad=yes
    input-default-bindings=yes
    sub-shadow-offset=2
    sub-shadow-color='#101010'
    sub-pos=95

    # 5. Ergonomie TV et télécommande / manette
    fullscreen=yes
    keep-open=yes
    cursor-autohide=1000
    osd-font-size=36
    osd-duration=2000
    input-ipc-server=/tmp/noos-mpv.sock
    osd-playing-msg="🎮 (A) Pause  (B) Quitter  (X) Sous-titres  (Y) Audio  (START) Menu  (SELECT) Infos"

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

    # Profil 0 : Désactivé (Natif)
    [upscale-off]
    glsl-shaders-clr
    scale=spline36
    cscale=spline36
    deband=no

    # --- PROFILS INTEL OPTIMISÉS (LENOVO M720q, NUC, IRIS XE & ARC) ---

    # Intel iGPU Éco (QuickSync & UHD Graphics 630 / HD - Zéro saccade, 60 FPS fluide, très faible charge)
    [upscale-intel-igpu]
    glsl-shaders-clr
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

    # Intel Adaptive CAS (Netteté intelligente pour Intel UHD 630 & Iris Xe)
    [upscale-intel-cas]
    glsl-shaders-set="/etc/mpv/shaders/CAS-scaled.glsl"
    scale=spline36
    cscale=spline36
    dscale=mitchell
    correct-downscaling=yes
    linear-downscaling=yes
    deband=yes
    deband-iterations=2
    deband-threshold=35

    # Intel XeSS Équilibré (Super-résolution neuronale 8 couches - Iris Xe / Intel Arc A380)
    [upscale-intel-xess-8]
    glsl-shaders-set="/etc/mpv/shaders/FSRCNNX_x2_8-0-4-1.glsl"
    scale=ewa_lanczossharp
    cscale=spline36

    # Intel XeSS Ultra (Réseau neuronal 16 couches + SSim - Intel Arc A750/A770)
    [upscale-intel-xess-16]
    glsl-shaders-set="/etc/mpv/shaders/FSRCNNX_x2_16-0-4-1.glsl:/etc/mpv/shaders/SSimDownscaler.glsl"
    scale=ewa_lanczossharp
    cscale=spline36

    # --- PROFILS AMD & NVIDIA & SPÉCIALISÉS ---

    [upscale-fsr]
    glsl-shaders-set="/etc/mpv/shaders/FSR.glsl"
    scale=ewa_lanczossharp
    cscale=spline36

    [upscale-cas]
    glsl-shaders-set="/etc/mpv/shaders/CAS-scaled.glsl"
    scale=spline36
    cscale=spline36

    [upscale-fsrcnnx-8]
    glsl-shaders-set="/etc/mpv/shaders/FSRCNNX_x2_8-0-4-1.glsl"
    scale=ewa_lanczossharp
    cscale=spline36

    [upscale-fsrcnnx-16]
    glsl-shaders-set="/etc/mpv/shaders/FSRCNNX_x2_16-0-4-1.glsl"
    scale=ewa_lanczossharp
    cscale=spline36

    [upscale-anime4k]
    glsl-shaders-set="/etc/mpv/shaders/Anime4K_Upscale_CNN_x2_M.glsl:/etc/mpv/shaders/Anime4K_Restore_CNN_M.glsl"
    scale=ewa_lanczossharp
    cscale=spline36

    [upscale-nnedi3-64]
    glsl-shaders-set="/etc/mpv/shaders/nnedi3-nns64-win8x6.hook"
    scale=ewa_lanczossharp
    cscale=spline36

    [upscale-krig]
    glsl-shaders-append="/etc/mpv/shaders/KrigBilateral.glsl"

    [upscale-nvscaler]
    glsl-shaders-set="/etc/mpv/shaders/NVScaler.glsl"
    scale=spline36
    cscale=spline36

    # Rétrocompatibilité profils iGPU/dGPU
    [iGPU-light-upscale]
    profile=upscale-intel-igpu

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

    # Profil Contenu 4K Natif (Désactive les filtres inutiles pour économiser le GPU)
    [4k-native]
    profile-cond=width >= 3840 or height >= 2160
    scale=bilinear
    cscale=bilinear
    deband=no

    # Profil Détection automatique des contenus 720p / 1080p selon le GPU
    [auto-upscale]
    profile-cond=(width < 3840 and height < 2160) and (width >= 1280 or height >= 720)
    ${if gpuCfg.profile == "intel" then "profile=upscale-intel-cas"
      else if gpuCfg.profile == "amd" then "profile=upscale-fsr"
      else if gpuCfg.profile == "nvidia" || gpuCfg.profile == "nvidia-legacy" then "profile=upscale-nvscaler"
      else "profile=upscale-intel-igpu"}
  '';

  # Configuration Thème Catppuccin Mocha & Ergonomie TV 10-Foot pour UOSC
  uoscConfig = ''
    # Configuration Thème Catppuccin Mocha & Ergonomie TV 10-Foot pour UOSC
    timeline_style=bar
    timeline_size=38
    timeline_line_width=3
    timeline_border=1
    timeline_cache=yes

    # Contrôles de lecture & timeline persistants en pause
    controls_persistency=paused
    timeline_persistency=paused

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
    menu_item_height=48
    menu_min_width=400
    menu_padding=8
    menu_type_to_search=no

    # Barre supérieure de titre et infos
    top_bar=no-border
    top_bar_size=42
    top_bar_controls=right
    top_bar_title=yes

    # Échelle pour grand écran TV 4K / 1080p
    scale=1.2
    scale_fullscreen=1.5
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

    # Contrôleur / Gamepad (Xbox, PlayStation, Manette Universelle)
    # Les commandes keypress permettent de piloter nativement les menus UOSC tout en contrôlant la lecture hors menu
    GAMEPAD_DPAD_UP keypress up
    GAMEPAD_DPAD_DOWN keypress down
    GAMEPAD_DPAD_LEFT keypress left
    GAMEPAD_DPAD_RIGHT keypress right
    GAMEPAD_LEFT_STICK_UP keypress up
    GAMEPAD_LEFT_STICK_DOWN keypress down
    GAMEPAD_LEFT_STICK_LEFT keypress left
    GAMEPAD_LEFT_STICK_RIGHT keypress right
    GAMEPAD_ACTION_DOWN keypress enter
    GAMEPAD_ACTION_RIGHT keypress esc
    GAMEPAD_ACTION_LEFT script-binding uosc/subtitles
    GAMEPAD_ACTION_UP script-binding uosc/audio
    GAMEPAD_START script-binding uosc/menu
    GAMEPAD_MENU script-binding uosc/menu
    GAMEPAD_BACK script-binding stats/display-stats-toggle
    GAMEPAD_RIGHT_TRIGGER script-binding uosc/stream-quality
    GAMEPAD_LEFT_SHOULDER seek -60
    GAMEPAD_RIGHT_SHOULDER seek 60
    GAMEPAD_LEFT_TRIGGER seek -10
    GAMEPAD_RIGHT_STICK_UP add volume 5
    GAMEPAD_RIGHT_STICK_DOWN add volume -5

    # Télécommande TV / Clavier standard & Commandes simulées par keypress
    UP add volume 2
    DOWN add volume -2
    LEFT seek -10
    RIGHT seek 10
    ENTER cycle pause
    ESC quit
    SPACE cycle pause
    PLAYPAUSE cycle pause
    PLAY set pause no
    PAUSE set pause yes
    Shift+RIGHT seek 60
    Shift+LEFT seek -60
    m cycle mute
    s script-binding uosc/subtitles
    a script-binding uosc/audio
    v script-binding uosc/stream-quality
    i script-binding stats/display-stats-toggle
    TAB script-binding uosc/toggle-ui
    MENU script-binding uosc/menu
    q quit

    # ==============================================================================
    # MENU UOSC DYNAMIQUE EN FRANÇAIS (Accessible via START / Touche MENU)
    # ==============================================================================
    # script-binding uosc/subtitles #! Sous-titres
    # script-binding uosc/audio #! Audio & Passthrough
    # script-binding uosc/stream-quality #! Qualite & Debit flux
    # script-binding stats/display-page-1-toggle #! Diagnostics > 1. Statistiques generales (Resolution & Codec)
    # script-binding stats/display-page-2-toggle #! Diagnostics > 2. Passes Shaders & Upscale GPU en direct
    # no-op #! ---
    # apply-profile upscale-intel-igpu #! Upscaling Intel > 1. Intel UHD/HD iGPU (Eco 60 FPS)
    # apply-profile upscale-intel-cas #! Upscaling Intel > 2. Intel Adaptive CAS (Nettete)
    # apply-profile upscale-intel-xess-8 #! Upscaling Intel > 3. Intel XeSS IA Equilibre (Iris/Arc)
    # apply-profile upscale-intel-xess-16 #! Upscaling Intel > 4. Intel XeSS IA Ultra (Arc Dedie)
    # apply-profile upscale-fsr #! Autres Shaders > AMD FSR (Super Resolution)
    # apply-profile upscale-cas #! Autres Shaders > AMD CAS
    # apply-profile upscale-nvscaler #! Autres Shaders > Nvidia NIS
    # apply-profile upscale-anime4k #! Autres Shaders > Anime4K (Dessins Animes)
    # apply-profile upscale-nnedi3-64 #! Autres Shaders > NNEDI3 64 Neurones
    # apply-profile upscale-krig #! Autres Shaders > KrigBilateral Chroma 4:4:4
    # apply-profile upscale-off #! Autres Shaders > Desactive (Natif)
    # no-op #! ---
    # quit #! Quitter le lecteur
  '';
in
{
  # 1. Installation de MPV avec support de tous les codecs et scripts
  environment.systemPackages = with pkgs; [
    (mpv.override {
      mpv-unwrapped = pkgs.mpv-unwrapped.override {
        sdl2Support = true;
      };
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
    "L+ /home/noos/.config/mpv/shaders - - - - /etc/mpv/shaders"
    "d /home/noos/.config/jellyfin-media-player 0755 noos users -"
    "L+ /home/noos/.config/jellyfin-media-player/shaders - - - - /etc/mpv/shaders"
  ];
}
