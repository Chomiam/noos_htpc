{ pkgs ? import <nixpkgs> { } }:

pkgs.rustPlatform.buildRustPackage rec {
  pname = "noos-htpc-installer";
  version = "0.2.0";

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
    cairo
    pango
    libsoup_3
    openssl
  ];

  postInstall = ''
    mkdir -p $out/share/noos-htpc-installer
    cp -r frontend $out/share/noos-htpc-installer/
    install -Dm644 icons/icon.png $out/share/icons/hicolor/scalable/apps/noos-htpc-installer.png
  '';

  meta = with pkgs.lib; {
    description = "Installateur graphique TV pour Noos HTPC contrôlable à la manette";
    license = licenses.mit;
    platforms = platforms.linux;
  };
}
