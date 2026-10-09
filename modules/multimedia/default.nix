{ pkgs, ... }:

{
  imports = [
    ./mpv.nix
    ./iptv.nix
    ./flatpak.nix
    ./storage-network.nix
  ];

  # Applications et bibliothèques multimédias Noos HTPC
  environment.systemPackages = with pkgs; [

    # Décodage, déchiffrement et contrôle pour disques DVD & Blu-ray (SATA et USB)
    libdvdcss         # Déchiffrement CSS indispensable pour DVD-Vidéo du commerce
    libdvdread        # Lecture des blocs et structures IFO/VOB DVD
    libdvdnav         # Menus interactifs et navigation DVD
    libbluray         # Décodage et structure BDMV pour disques Blu-ray
    eject             # Commande d'ouverture / éjection du tiroir optique
  ];
}
