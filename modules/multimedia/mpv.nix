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
    reset-on-clear=yes

    # 1.1 Passthrough HDR dynamique & Tonemapping intelligent
    target-colorspace-hint=auto
    gamut-mapping-mode=auto

    # 2. Fluidité et élimination du judder (24Hz sur écran 60Hz/120Hz)
    video-sync=display-resample
    interpolation=yes
    tscale=oversample

    # 3. Audio Passthrough haute fidélité (barres de son et amplis home-cinéma)
    ao=pipewire,pulse,alsa
    audio-spdif=ac3,dts,eac3,truehd,dts-hd
    audio-channels=auto
    audio-pitch-correction=yes
    volume-max=150

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
    # PROFILS D'UPSCALING, TONE-MAPPING & AFFICHAGE
    # ==============================================================================

    # Profil 0 : Repli Tonemapping Automatique HDR vers SDR (libplacebo)
    # Activé automatiquement dès qu'un flux HDR (BT.2020 / PQ / HLG) est détecté sur écran SDR
    [hdr-to-sdr-fallback]
    profile-cond=p["video-params/primaries"] == "bt.2020" or p["video-params/gamma"] == "pq" or p["video-params/gamma"] == "hlg"
    profile-restore=copy
    tone-mapping=spline
    tone-mapping-mode=auto
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
    cscale=ewa_lanczossoft
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

  # Fichier input.conf pour le contrôle à la manette et télécommande
  mpvInput = ''
    # Contrôles pour Gamepad / Télécommande
    PLAYPAUSE cycle pause
    SPACE cycle pause
    UP add volume 2
    DOWN add volume -2
    RIGHT seek 10
    LEFT seek -10
    Shift+RIGHT seek 60
    Shift+LEFT seek -60
    m cycle mute
    s cycle sub
    a cycle audio
    f cycle fullscreen
    q quit
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
  ];

  # 2. Déploiement de la configuration déclarative globale /etc/mpv/
  environment.etc."mpv/mpv.conf".text = mpvConfig;
  environment.etc."mpv/input.conf".text = mpvInput;
}
