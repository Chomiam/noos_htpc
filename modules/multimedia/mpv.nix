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

    # --- PROFILS AMD RYZEN & RADEON OPTIMISÉS ---

    # AMD APU Ryzen Éco (iGPU Vega / Radeon 680M/780M - Léger et fluide 60 FPS)
    [upscale-amd-apu]
    glsl-shaders-set="/etc/mpv/shaders/CAS-scaled.glsl"
    scale=spline36
    cscale=spline36
    dscale=mitchell
    correct-downscaling=yes
    linear-downscaling=yes
    deband=yes
    deband-iterations=2
    deband-threshold=35

    # AMD FSR Super Resolution (Algorithme spatial EASU + RCAS)
    [upscale-amd-fsr]
    glsl-shaders-set="/etc/mpv/shaders/FSR.glsl"
    scale=ewa_lanczossharp
    cscale=spline36
    deband=yes
    deband-iterations=2
    deband-threshold=35

    # AMD FidelityFX CAS Pur (Netteté chirurgicale adaptative au contraste)
    [upscale-amd-cas]
    glsl-shaders-set="/etc/mpv/shaders/CAS-scaled.glsl"
    scale=ewa_lanczossharp
    cscale=spline36
    deband=yes

    # AMD Radeon RX Dédié Ultra (FSR + KrigBilateral Chroma 4:4:4 + Deband haute qualité)
    [upscale-amd-high]
    glsl-shaders-set="/etc/mpv/shaders/FSR.glsl:/etc/mpv/shaders/KrigBilateral.glsl"
    scale=ewa_lanczossharp
    cscale=spline36
    deband=yes
    deband-iterations=4
    deband-threshold=48

    # --- PROFILS NVIDIA GEFORCE OPTIMISÉS ---

    # Nvidia GTX (GeForce GTX 10xx / 16xx - NVScaler + Spline36)
    [upscale-nvidia-gtx]
    glsl-shaders-set="/etc/mpv/shaders/NVScaler.glsl"
    scale=spline36
    cscale=spline36
    dscale=mitchell
    deband=yes
    deband-iterations=2
    deband-threshold=35

    # Nvidia Image Scaling Officiel (NIS / NVScaler + Lanczos)
    [upscale-nvidia-nis]
    glsl-shaders-set="/etc/mpv/shaders/NVScaler.glsl"
    scale=ewa_lanczossharp
    cscale=spline36
    deband=yes

    # Nvidia RTX Tensor Core (FSRCNNX IA 8 couches + KrigBilateral Chroma 4:4:4)
    [upscale-nvidia-rtx]
    glsl-shaders-set="/etc/mpv/shaders/FSRCNNX_x2_8-0-4-1.glsl:/etc/mpv/shaders/KrigBilateral.glsl"
    scale=ewa_lanczossharp
    cscale=spline36
    deband=yes
    deband-iterations=3
    deband-threshold=40

    # Nvidia RTX Ultra Studio (FSRCNNX IA 16 couches + SSimDownscaler + KrigBilateral)
    [upscale-nvidia-rtx-ultra]
    glsl-shaders-set="/etc/mpv/shaders/FSRCNNX_x2_16-0-4-1.glsl:/etc/mpv/shaders/SSimDownscaler.glsl:/etc/mpv/shaders/KrigBilateral.glsl"
    scale=ewa_lanczossharp
    cscale=spline36
    deband=yes
    deband-iterations=4
    deband-threshold=48

    # --- PROFILS SPÉCIALISÉS & RESTAURATION ---

    # Profil Animation & Manga (Reconstruction des contours et lignes nettes)
    [upscale-anime4k]
    glsl-shaders-set="/etc/mpv/shaders/Anime4K_Upscale_CNN_x2_M.glsl:/etc/mpv/shaders/Anime4K_Restore_CNN_M.glsl"
    scale=ewa_lanczossharp
    cscale=spline36

    # Restauration Vieux Films & Séries (SD / DVD 480p/576p vers HD/4K)
    [upscale-vintage-sd]
    glsl-shaders-set="/etc/mpv/shaders/nnedi3-nns64-win8x6.hook:/etc/mpv/shaders/KrigBilateral.glsl"
    scale=spline36
    cscale=spline36
    deband=yes
    deband-iterations=4
    deband-threshold=48
    deband-range=24

    # Profils génériques FSR / CAS / NNEDI3
    [upscale-fsr]
    profile=upscale-amd-fsr

    [upscale-cas]
    profile=upscale-amd-cas

    [upscale-fsrcnnx-8]
    profile=upscale-intel-xess-8

    [upscale-fsrcnnx-16]
    profile=upscale-intel-xess-16

    [upscale-nnedi3-64]
    glsl-shaders-set="/etc/mpv/shaders/nnedi3-nns64-win8x6.hook"
    scale=ewa_lanczossharp
    cscale=spline36

    [upscale-krig]
    glsl-shaders-append="/etc/mpv/shaders/KrigBilateral.glsl"

    [upscale-nvscaler]
    profile=upscale-nvidia-nis

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
      else if gpuCfg.profile == "amd" then "profile=upscale-amd-fsr"
      else if gpuCfg.profile == "nvidia" || gpuCfg.profile == "nvidia-legacy" then "profile=upscale-nvidia-gtx"
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
    # MENU UOSC PRINCIPAL HTPC (Accessible via START / Touche MENU)
    # ==============================================================================

    # --- 1. PISTES & FLUX ---
    # script-binding uosc/subtitles #! 1. Pistes & Flux > Sous-titres
    # script-binding uosc/audio #! 1. Pistes & Flux > Audio & Passthrough
    # script-binding uosc/stream-quality #! 1. Pistes & Flux > Qualite & Debit flux
    # no-op #! 1. Pistes & Flux > ---
    # cycle audio ; show-text "Piste audio suivante" #! 1. Pistes & Flux > Changer de piste audio
    # cycle sub ; show-text "Piste sous-titres suivante" #! 1. Pistes & Flux > Changer de piste sous-titres
    # cycle sub-visibility ; show-text "Sous-titres: ''\${sub-visibility}" #! 1. Pistes & Flux > Afficher / Masquer sous-titres

    # --- 2. MOTEURS D'UPSCALE (SCALERS GÉOMÉTRIQUES) ---
    # set scale spline36 ; show-text "Scaler: Spline36 (Recommande)" #! 2. Moteurs d'Upscale (Scalers) > Spline36 (Equilibre Intel UHD - Recommande)
    # set scale ewa_lanczossharp ; show-text "Scaler: EWA-Lanczos (Haute Precision)" #! 2. Moteurs d'Upscale (Scalers) > EWA-Lanczos / Jinc (Ultra Haute Precision)
    # set scale mitchell ; show-text "Scaler: Bicubique Mitchell" #! 2. Moteurs d'Upscale (Scalers) > Bicubique Mitchell (Doux & Cinematique)
    # set scale bilinear ; show-text "Scaler: Bilineaire (Eco)" #! 2. Moteurs d'Upscale (Scalers) > Bilineaire (Mode Eco iGPU)

    # --- 3. SHADERS GLSL (SUPER-RÉSOLUTION & NETTETÉ) ---
    # change-list glsl-shaders clr "" ; show-text "Shaders desactives (Rendu pur)" #! 3. Shaders & Ameliorations > Desactiver tous les shaders (Natif)
    # change-list glsl-shaders set "/etc/mpv/shaders/CAS-scaled.glsl" ; show-text "Shader: Intel / AMD CAS (Nettete)" #! 3. Shaders & Ameliorations > Intel / AMD CAS (Nettete intelligente - Recommande UHD 630)
    # change-list glsl-shaders set "/etc/mpv/shaders/FSR.glsl" ; show-text "Shader: AMD FSR (Super Resolution)" #! 3. Shaders & Ameliorations > AMD FSR (Super Resolution spatiale)
    # change-list glsl-shaders set "/etc/mpv/shaders/FSRCNNX_x2_8-0-4-1.glsl" ; show-text "Shader: Intel XeSS / FSRCNNX IA" #! 3. Shaders & Ameliorations > Intel XeSS / FSRCNNX (Super-resolution neuronale 8 couches)
    # change-list glsl-shaders set "/etc/mpv/shaders/FSRCNNX_x2_16-0-4-1.glsl" ; show-text "Shader: FSRCNNX 16 couches Ultra" #! 3. Shaders & Ameliorations > FSRCNNX IA Ultra (16 couches)
    # change-list glsl-shaders set "/etc/mpv/shaders/NVScaler.glsl" ; show-text "Shader: Nvidia NIS" #! 3. Shaders & Ameliorations > Nvidia NIS (Nvidia Image Scaling)
    # change-list glsl-shaders set "/etc/mpv/shaders/Anime4K_Upscale_CNN_x2_M.glsl:/etc/mpv/shaders/Anime4K_Restore_CNN_M.glsl" ; show-text "Shader: Anime4K" #! 3. Shaders & Ameliorations > Anime4K (Anime & Dessins Animes)
    # change-list glsl-shaders set "/etc/mpv/shaders/nnedi3-nns64-win8x6.hook" ; show-text "Shader: NNEDI3 (64 neurones)" #! 3. Shaders & Ameliorations > NNEDI3 (Interpolation Neuronale 64)

    # --- 4. PROFILS COMPLETS CLÉ EN MAIN (PAR GPU & CAS D'USAGE) ---
    # apply-profile upscale-intel-igpu ; show-text "Profil: Intel UHD 630 Eco" #! 4. Profils Complets GPU > Profils Intel > 1. Intel UHD/HD iGPU (Eco 60 FPS)
    # apply-profile upscale-intel-cas ; show-text "Profil: Intel Adaptive CAS" #! 4. Profils Complets GPU > Profils Intel > 2. Intel Adaptive CAS (Nettete UHD 630)
    # apply-profile upscale-intel-xess-8 ; show-text "Profil: Intel XeSS IA" #! 4. Profils Complets GPU > Profils Intel > 3. Intel XeSS IA Equilibre (Iris Xe / Arc)
    # apply-profile upscale-intel-xess-16 ; show-text "Profil: Intel XeSS Ultra" #! 4. Profils Complets GPU > Profils Intel > 4. Intel XeSS IA Ultra (Arc Dedie)
    # apply-profile upscale-amd-apu ; show-text "Profil: AMD Ryzen APU Eco" #! 4. Profils Complets GPU > Profils AMD > 1. AMD APU Ryzen (Vega / Radeon 600M-700M)
    # apply-profile upscale-amd-fsr ; show-text "Profil: AMD FSR Super Resolution" #! 4. Profils Complets GPU > Profils AMD > 2. AMD FSR Super Resolution (EASU + RCAS)
    # apply-profile upscale-amd-cas ; show-text "Profil: AMD FidelityFX CAS" #! 4. Profils Complets GPU > Profils AMD > 3. AMD FidelityFX CAS (Nettete chirurgicale)
    # apply-profile upscale-amd-high ; show-text "Profil: AMD Radeon RX Ultra" #! 4. Profils Complets GPU > Profils AMD > 4. AMD Radeon RX Ultra (FSR + Krig 4:4:4)
    # apply-profile upscale-nvidia-gtx ; show-text "Profil: Nvidia GTX NIS" #! 4. Profils Complets GPU > Profils Nvidia > 1. Nvidia GTX (GeForce 10xx / 16xx NIS)
    # apply-profile upscale-nvidia-nis ; show-text "Profil: Nvidia NIS Officiel" #! 4. Profils Complets GPU > Profils Nvidia > 2. Nvidia Image Scaling Officiel
    # apply-profile upscale-nvidia-rtx ; show-text "Profil: Nvidia RTX IA Tensor" #! 4. Profils Complets GPU > Profils Nvidia > 3. Nvidia RTX Tensor Core (FSRCNNX IA)
    # apply-profile upscale-nvidia-rtx-ultra ; show-text "Profil: Nvidia RTX Ultra" #! 4. Profils Complets GPU > Profils Nvidia > 4. Nvidia RTX Ultra Studio (16 couches + SSim)
    # apply-profile upscale-anime4k ; show-text "Profil: Anime4K" #! 4. Profils Complets GPU > Profils Specialises > 1. Mode Animation & Manga (Anime4K CNN)
    # apply-profile upscale-vintage-sd ; show-text "Profil: Restauration SD/DVD" #! 4. Profils Complets GPU > Profils Specialises > 2. Restauration Vieux Films (SD / DVD 576p)
    # apply-profile 4k-native ; show-text "Profil: 4K Natif Eco" #! 4. Profils Complets GPU > Profils Specialises > 3. Contenu 4K Natif (Eco GPU)
    # apply-profile upscale-off ; show-text "Profil: Rendu Natif Desactive" #! 4. Profils Complets GPU > Profils Specialises > 4. Desactive (Rendu Natif sans filtre)

    # --- 5. FORMAT D'IMAGE & ZOOM ---
    # set video-aspect-override "-1" ; show-text "Format: Auto / Original" #! 5. Format & Cadrage > Format Original (Auto)
    # set video-aspect-override "16:9" ; show-text "Format: 16:9 Plein ecran" #! 5. Format & Cadrage > Forcer 16:9 (Plein ecran standard)
    # set video-aspect-override "21:9" ; show-text "Format: 21:9 Cinemascope" #! 5. Format & Cadrage > Forcer 21:9 (Cinemascope sans bandes noires)
    # set video-aspect-override "4:3" ; show-text "Format: 4:3 Retro" #! 5. Format & Cadrage > Forcer 4:3 (Series & Films retro)
    # no-op #! 5. Format & Cadrage > ---
    # add video-zoom 0.05 ; show-text "Zoom: ''\${video-zoom}" #! 5. Format & Cadrage > Zoom Avant (+5%)
    # add video-zoom -0.05 ; show-text "Zoom: ''\${video-zoom}" #! 5. Format & Cadrage > Zoom Arriere (-5%)
    # set video-zoom 0 ; set video-pan-x 0 ; set video-pan-y 0 ; show-text "Zoom et Cadrage reinitialises" #! 5. Format & Cadrage > Reinitialiser Zoom & Cadrage

    # --- 6. VITESSE DE LECTURE ---
    # set speed 0.75 ; show-text "Vitesse: 0.75x" #! 6. Vitesse de lecture > 0.75x (Ralenti)
    # set speed 1.0 ; show-text "Vitesse: 1.0x (Normale)" #! 6. Vitesse de lecture > 1.0x (Vitesse Normale)
    # set speed 1.25 ; show-text "Vitesse: 1.25x" #! 6. Vitesse de lecture > 1.25x
    # set speed 1.5 ; show-text "Vitesse: 1.5x" #! 6. Vitesse de lecture > 1.5x
    # set speed 2.0 ; show-text "Vitesse: 2.0x" #! 6. Vitesse de lecture > 2.0x (Accelere)

    # --- 7. ÉTALONNAGE VIDÉO & COULEURS ---
    # add brightness 2 ; show-text "Luminosite: ''\${brightness}" #! 7. Image & Etalonnage > Luminosite +
    # add brightness -2 ; show-text "Luminosite: ''\${brightness}" #! 7. Image & Etalonnage > Luminosite -
    # add contrast 2 ; show-text "Contraste: ''\${contrast}" #! 7. Image & Etalonnage > Contraste +
    # add contrast -2 ; show-text "Contraste: ''\${contrast}" #! 7. Image & Etalonnage > Contraste -
    # add saturation 2 ; show-text "Saturation: ''\${saturation}" #! 7. Image & Etalonnage > Couleurs / Saturation +
    # add saturation -2 ; show-text "Saturation: ''\${saturation}" #! 7. Image & Etalonnage > Couleurs / Saturation -
    # add gamma 2 ; show-text "Gamma: ''\${gamma}" #! 7. Image & Etalonnage > Gamma +
    # add gamma -2 ; show-text "Gamma: ''\${gamma}" #! 7. Image & Etalonnage > Gamma -
    # set brightness 0 ; set contrast 0 ; set saturation 0 ; set gamma 0 ; show-text "Etalonnage Image reinitialise" #! 7. Image & Etalonnage > Reinitialiser Etalonnage (Zero)

    # --- 8. SYNCHRONISATION & DÉCALAGES ---
    # add audio-delay 0.1 ; show-text "Delai Audio: ''\${audio-delay}s" #! 8. Synchronisation (A/V Sync) > Retarder Audio (+100 ms)
    # add audio-delay -0.1 ; show-text "Delai Audio: ''\${audio-delay}s" #! 8. Synchronisation (A/V Sync) > Avancer Audio (-100 ms)
    # set audio-delay 0 ; show-text "Delai Audio reinitialise (0 ms)" #! 8. Synchronisation (A/V Sync) > Reinitialiser Audio (0 ms)
    # no-op #! 8. Synchronisation (A/V Sync) > ---
    # add sub-delay 0.1 ; show-text "Delai Sous-titres: ''\${sub-delay}s" #! 8. Synchronisation (A/V Sync) > Retarder Sous-titres (+100 ms)
    # add sub-delay -0.1 ; show-text "Delai Sous-titres: ''\${sub-delay}s" #! 8. Synchronisation (A/V Sync) > Avancer Sous-titres (-100 ms)
    # set sub-delay 0 ; show-text "Delai Sous-titres reinitialise (0 ms)" #! 8. Synchronisation (A/V Sync) > Reinitialiser Sous-titres (0 ms)
    # no-op #! 8. Synchronisation (A/V Sync) > ---
    # add sub-font-size 2 ; show-text "Taille Sous-titres: ''\${sub-font-size}" #! 8. Synchronisation (A/V Sync) > Agrandir Sous-titres (+2 pt)
    # add sub-font-size -2 ; show-text "Taille Sous-titres: ''\${sub-font-size}" #! 8. Synchronisation (A/V Sync) > Reduire Sous-titres (-2 pt)
    # add sub-pos -2 ; show-text "Position Sous-titres: ''\${sub-pos}" #! 8. Synchronisation (A/V Sync) > Monter Sous-titres
    # add sub-pos 2 ; show-text "Position Sous-titres: ''\${sub-pos}" #! 8. Synchronisation (A/V Sync) > Baisser Sous-titres

    # --- 9. TRAITEMENTS VIDÉO AVANCÉS ---
    # cycle deband ; show-text "Debanding: ''\${deband}" #! 9. Traitements Avances > Debanding (Anti-bandes de compression)
    # change-list glsl-shaders toggle "/etc/mpv/shaders/KrigBilateral.glsl" ; show-text "KrigBilateral Chroma bascule" #! 9. Traitements Avances > KrigBilateral (Chroma 4:4:4 haute fidelite)
    # cycle interpolation ; show-text "Interpolation: ''\${interpolation}" #! 9. Traitements Avances > Interpolation 60Hz (Anti-saccades 24p)

    # --- 10. DIAGNOSTICS & STATISTIQUES ---
    # script-binding stats/display-page-1-toggle #! Diagnostics > 1. Statistiques generales (Resolution, Codec, Debit)
    # script-binding stats/display-page-2-toggle #! Diagnostics > 2. Passes Shaders & Upscale GPU en direct

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
