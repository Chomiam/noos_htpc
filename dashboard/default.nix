{ pkgs ? import <nixpkgs> { } }:

pkgs.rustPlatform.buildRustPackage rec {
  pname = "noos-tv-dashboard";
  version = "0.3.3";

  src = ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  doCheck = false;

  nativeBuildInputs = with pkgs; [
    pkg-config
    wrapGAppsHook3
  ];

  buildInputs = with pkgs; [
    gtk3
    webkitgtk_4_1
    glib
    glib-networking
    cairo
    pango
    libsoup_3
    openssl
  ];

  postInstall = ''
    mkdir -p $out/share/noos-tv-dashboard
    cp -r frontend $out/share/noos-tv-dashboard/
    install -Dm644 icons/icon.png $out/share/icons/hicolor/scalable/apps/noos-tv-dashboard.png
  '';

  meta = with pkgs.lib; {
    description = "Tableau de bord TV Wayland immersif pour Noos HTPC";
    license = licenses.mit;
    platforms = platforms.linux;
  };
}
