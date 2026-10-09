{ pkgs, ... }:

{
  imports = [
    ./mpv.nix
    ./iptv.nix
    ./flatpak.nix
    ./storage-network.nix
  ];

  # Applications multimédia supplémentaires Noos HTPC
  environment.systemPackages = with pkgs; [
    pear-desktop      # Client YouTube Music avec bloqueur de pubs et extensions audio
  ];
}
