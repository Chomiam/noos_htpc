{ pkgs ? import <nixpkgs> { } }:

pkgs.stdenv.mkDerivation {
  pname = "noos-osk";
  version = "0.1.0";

  src = ./.;

  nativeBuildInputs = with pkgs; [
    pkg-config
  ];

  buildInputs = with pkgs; [
    gtk3
    gtk-layer-shell
    glib
  ];

  buildPhase = ''
    gcc -O2 -Wall main.c -o noos-osk $(pkg-config --cflags --libs gtk+-3.0 gtk-layer-shell-0 gio-2.0) -lpthread
  '';

  installPhase = ''
    install -Dm755 noos-osk $out/bin/noos-osk
  '';

  meta = with pkgs.lib; {
    description = "Clavier virtuel Wayland pilotable à la manette pour Noos HTPC";
    license = licenses.mit;
    platforms = platforms.linux;
  };
}
