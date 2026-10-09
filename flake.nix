{
  description = "Noos HTPC - Distribution NixOS dédiée aux mini PC multimédia et consoles de rétro-gaming sur TV";

  inputs = {
    # Synchronisation sur la branche standard de l'écosystème Noos (nixos-26.05)
    nixpkgs.url = "github:nixos/nixpkgs/nixos-26.05";

    # Gestion déclarative des paquets Flatpak
    nix-flatpak.url = "github:gmodena/nix-flatpak";
  };

  outputs = { self, nixpkgs, nix-flatpak, ... }@inputs:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
        config.allowUnfree = true;
      };

      # Définition du système principal Noos HTPC
      mkHtpcSystem = hostname: nixpkgs.lib.nixosSystem {
        inherit system;
        specialArgs = { inherit inputs self; };
        modules = [
          nix-flatpak.nixosModules.nix-flatpak
          {
            networking.hostName = hostname;
          }
          ./hosts/htpc/configuration.nix
        ];
      };
    in
    {
      # Configurations système prêtes au déploiement via nixos-rebuild
      nixosConfigurations = {
        htpc = mkHtpcSystem "noos-htpc";
        noos-htpc = self.nixosConfigurations.htpc;
        default = self.nixosConfigurations.htpc;

        # Image ISO d'installation autonome bootable sur TV
        iso = nixpkgs.lib.nixosSystem {
          inherit system;
          specialArgs = { inherit inputs self; };
          modules = [
            "${nixpkgs}/nixos/modules/installer/cd-dvd/installation-cd-minimal.nix"
            ./iso/installer-iso.nix
          ];
        };
      };

      # Module exportable pour inclusion dans d'autres dépôts Noos
      nixosModules = {
        default = {
          imports = [
            nix-flatpak.nixosModules.nix-flatpak
            ./modules
          ];
        };
        htpc = self.nixosModules.default;
      };

      # Applications graphiques Rust + Tauri & Cible ISO
      packages.${system} = rec {
        noos-htpc-installer = pkgs.callPackage ./installer/default.nix { };
        noos-tv-dashboard = pkgs.callPackage ./dashboard/default.nix { };
        noos-osk = pkgs.callPackage ./osk/default.nix { };
        iso = self.nixosConfigurations.iso.config.system.build.isoImage;
        default = iso;
      };

      apps.${system} = {
        default = {
          type = "app";
          program = "${self.packages.${system}.noos-htpc-installer}/bin/noos-htpc-installer";
          meta.description = "Installateur graphique TV Noos HTPC";
        };
      };

      # Shell de développement léger
      devShells.${system}.default = pkgs.mkShell {
        packages = with pkgs; [
          git
          nix-output-monitor
          nvd
        ];
      };
    };
}
