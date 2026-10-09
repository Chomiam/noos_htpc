{
  description = "Noos HTPC - Distribution NixOS dédiée aux mini PC multimédia et consoles de rétro-gaming sur TV";

  inputs = {
    # Synchronisation sur la branche standard de l'écosystème Noos (nixos-26.05)
    nixpkgs.url = "github:nixos/nixpkgs/nixos-26.05";
  };

  outputs = { self, nixpkgs, ... }@inputs:
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
        default = ./modules;
        htpc = ./modules;
      };

      # Application d'installation graphique Rust + Tauri
      packages.${system} = rec {
        noos-htpc-installer = pkgs.callPackage ./installer/default.nix { };
        default = noos-htpc-installer;
      };

      apps.${system} = {
        default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/noos-htpc-installer";
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
