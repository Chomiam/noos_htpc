# 🎮 Noos HTPC — Distribution NixOS Console & Media Center TV

Distribution NixOS personnalisée, légère, hautement optimisée et 100% reproductible, conçue spécifiquement pour transformer les **mini-PC de salon** (Intel NUC, Beelink, AMD Ryzen Mini, etc.) en **lecteurs multimédias 4K HDR** et **consoles de rétro-gaming** pour la télévision.

---

## 🌟 Points Forts & Spécifications

* **Expérience TV "Console-like" & Sans clavier :** Démarrage direct au boot sur grand écran via **Gamescope** (micro-compositeur de Valve) avec zéro tearing, latence ultra-faible et support expérimental **HDR**.
* **Profils GPU Modulaires :** Prise en charge native d'**AMD** (amdgpu + RADV + HDR), **Intel** (QuickSync VA-API iHD), **Nvidia Moderne** (GTX 1650+ / RTX propriétaires) et **Nvidia Legacy** (470xx).
* **Autologin & Sécurité :** Compte utilisateur `noos` (mot de passe par défaut : `admin` hashé de manière déclarative), avec extinction et redémarrage rapides sans mot de passe.
* **Multimédia Universel & 4K :** Lecteur **MPV** avec accélération matérielle complète (`hwdec`), profils d'upscaling automatiques légers (iGPU) et lourds (dGPU), passthrough audio bitstream (Dolby Atmos, DTS-HD, TrueHD) et lecteur **IPTV** (Hypnotix / M3U).
* **Rétro-Gaming & Émulation :** Interface **ES-DE (EmulationStation Desktop Edition)** couplée à **RetroArch** et tous les cores majeurs (NES, SNES, Megadrive, PS1, PS2, GameCube, Arcade). Dossiers de ROMs et BIOS préconfigurés dans `/home/noos/Retro/`.
* **Manettes "Plug & Play" :** Optimisations udev et Bluetooth pour manettes **Xbox** (xpadneo), **PlayStation** (DualShock 4 / DualSense PS5), **8BitDo** et **Nintendo Switch**.
* **Stockage & Réseau Transparent :** Partage **Samba (SMB)** activé pour déposer des ROMs directement depuis un PC distant, montage automatique des clés USB et disques durs via **UDisks2/GVFS**, et explorateur **Thunar** aux icônes adaptées à la TV.
* **Mises à jour & Robustesse NixOS :** Script déclaratif `noos-update` avec retour arrière instantané en cas d'imprévu via `noos-rollback` et le menu de démarrage UEFI.

---

## 📁 Architecture du Dépôt

```
/home/chomiam/Projets/noos_htpc/
├── flake.nix                                # Point d'entrée Flake (nixosConfigurations.htpc)
├── logo.png                                 # Logo officiel Noos HTPC (Plymouth & OSD)
├── hosts/
│   └── htpc/
│       ├── configuration.nix                # Configuration hôte (sélecteur GPU, nom, réseau)
│       └── hardware-configuration.nix       # Détection matérielle & partitions
├── modules/
│   ├── default.nix                          # Importeur racine de tous les sous-modules
│   ├── core/                                # Cœur du système
│   │   ├── boot.nix                         # Boot silencieux, Plymouth splash, UEFI rollback
│   │   ├── user.nix                         # Compte 'noos', autologin, sudoers NOPASSWD, dossiers
│   │   ├── audio.nix                        # PipeWire, WirePlumber, basse latence, Bluetooth
│   │   └── system.nix                       # Fuseau, ZRAM, paquets de base, noos-update
│   ├── hardware/                            # Matériel & Profils GPU
│   │   ├── gpu-profiles.nix                 # Sélecteur central déclaratif de profil GPU
│   │   ├── amd.nix                          # Pilote amdgpu, RADV Vulkan, HDR Wayland
│   │   ├── intel.nix                        # QuickSync VA-API (iHD), Intel Media Driver
│   │   ├── nvidia.nix                       # Pilotes propriétaires modernes (GTX 1650+)
│   │   ├── nvidia-legacy.nix                # Pilotes propriétaires legacy (470xx)
│   │   └── controllers.nix                  # Udev rules Xbox, PS4/PS5, 8BitDo, Switch Pro
│   ├── desktop/                             # Environnement TV
│   │   ├── gamescope-session.nix            # Session Gamescope, GameMode, FSR upscaling
│   │   └── display-manager.nix              # Autologin direct via Greetd sans mot de passe
│   ├── multimedia/                          # Lecteurs multimédias
│   │   ├── mpv.nix                          # MPV 4K, shaders upscaling iGPU vs dGPU, passthrough
│   │   ├── iptv.nix                         # Hypnotix, script noos-iptv
│   │   ├── flatpak.nix                      # nix-flatpak, VacuumTube (YouTube TV) Flatpak déclaratif
│   │   ├── jellyfin.nix                     # Jellyfin Media Player (moteur MPV 4K HDR & mode TV 10-foot)
│   │   └── storage-network.nix              # Samba (partage /home), SFTP/SSHFS, UDisks2, Thunar
│   └── gaming/                              # Émulation
│       ├── emulationstation.nix             # ES-DE préconfiguré pour la TV
│       └── retroarch.nix                    # RetroArch avec cores et mapping /home/noos/Retro
└── scripts/
    ├── noos-update.sh                       # Script de mise à jour sécurisée avec rollback
    └── noos-htpc-launcher.sh                # Lanceur universel TV / manette
```

---

## ⚡ Sélecteur Rapide de GPU

Dans le fichier [hosts/htpc/configuration.nix](file:///home/chomiam/Projets/noos_htpc/hosts/htpc/configuration.nix), configurez votre profil en une ligne :

```nix
hardware.noos-htpc.gpu = {
  profile = "amd";    # Options disponibles : "amd" | "intel" | "nvidia" | "nvidia-legacy" | "generic"
  enableHDR = true;   # Active le support HDR sur TV compatible (AMD / Nvidia)
};
```

| Profil | Cible matérielle | Accélération & Spécificités |
| :--- | :--- | :--- |
| **`amd`** | Beelink SER, Minisforum, APU Ryzen, GPU Radeon | Pilote `amdgpu` open-source, Vulkan RADV, FreeSync, HDR via Gamescope. |
| **`intel`** | Intel NUC, Celeron N100, Core i3/i5/i7, Iris Xe | QuickSync VA-API moderne (`intel-media-driver` iHD), faible consommation. |
| **`nvidia`** | Mini PC & tours avec GTX 1650, RTX 2060/3060/4060+ | Pilote propriétaire stable, NVDEC VA-API, modesetting Wayland. |
| **`nvidia-legacy`** | Anciens PC avec GeForce séries 600/700/800 | Pilote propriétaire legacy 470xx. |
| **`generic`** | Tout autre matériel ou machine virtuelle | Pilote générique Mesa/modesetting. |

---

## 🎬 Profils d'Upscaling MPV

La configuration MPV ([modules/multimedia/mpv.nix](file:///home/chomiam/Projets/noos_htpc/modules/multimedia/mpv.nix)) détecte la résolution du fichier lu :

1. **`[iGPU-light-upscale]` (Intel NUC / AMD Vega APU) :** Algorithmes `spline36` + `mitchell`, debanding léger. Assure un affichage 1080p -> 4K sans perte de trames (*0 dropped frames*).
2. **`[dGPU-high-upscale]` (GPU dédiés) :** Algorithmes `ewa_lanczossharp` haute fidélité, debanding poussé, tone-mapping HDR automatique et correction colorimétrique.
3. **`[4k-native]` :** Désactive les filtres gourmands pour préserver les ressources lorsque la vidéo source est déjà en 2160p natif.

---

## 🕹️ Emplacement des ROMs & Partage Réseau

Les dossiers de jeux sont automatiquement créés dans le répertoire utilisateur :
* **ROMs :** `/home/noos/Retro/ROMS/` (sous-dossiers : `nes`, `snes`, `megadrive`, `psx`, `ps2`, `gc`, `arcade`)
* **BIOS :** `/home/noos/Retro/BIOS/`
* **Vidéos :** `/home/noos/Videos/`
* **Playlists IPTV :** `/home/noos/IPTV/`

### Dépôt de fichiers par le réseau local (Samba) :
Depuis un PC sous Windows, Mac ou Linux connecté au même réseau :
* Tapez `\\noos-htpc\Noos-Roms` (ou `smb://noos-htpc.local/Noos-Roms`) pour déposer directement vos jeux sans clé USB.

---

## 🔄 Mise à jour et Maintenance

Pour mettre à jour la machine de salon :
```bash
noos-update
```

En cas de problème après une mise à jour :
```bash
noos-rollback
# Ou sélectionnez directement la version précédente dans le menu de démarrage UEFI
```

---

## 🚀 Installateur Graphique TV (Rust + Tauri v2)

L'installation de **Noos HTPC** est conçue pour s'effectuer intégralement depuis le canapé, **à la manette**, sans clavier ni souris physiques :

* **Moteur :** Application native en **Rust** propulsée par **Tauri v2**, avec détection matérielle directe des disques (`lsblk`), des puces graphiques (`lspci`), et des réseaux Wi-Fi (`nmcli`).
* **Navigation Manette :** Moteur de navigation spatiale 60 FPS supportant les manettes Xbox, PlayStation (DualShock / DualSense), 8BitDo et Switch Pro.
* **Clavier Virtuel Intégré (OSK) :** Saisie des mots de passe Wi-Fi et noms d'hôte via un dock virtuel pilotable au D-Pad avec raccourcis directs :
  * **(A)** : Valider / Écrire la touche sélectionnée
  * **(B)** : Fermer le clavier virtuel / Étape précédente
  * **(X)** : Ouvrir le clavier virtuel / Touche Effacer (Backspace)
  * **(Y)** : Touche Espace
  * **(Start)** : Confirmer et passer à l'étape suivante
* **Image ISO Bootable :** Démarrage direct sur clé USB en plein écran avec micro-compositeur Gamescope pour une expérience zéro tearing dès le premier boot.

