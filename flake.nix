{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-22.11";

  outputs = { self, nixpkgs, ... }:
    let
      version = "0.1.0";
      supportedSystems = [ "x86_64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
      nixpkgsFor = forAllSystems (system:
        import nixpkgs {
          inherit system;
          overlays = [ self.overlays.default ];
        });
    in
    {
      overlays.default = final: prev: {
        tiny-azagent = final.callPackage ./nix/package.nix {
          inherit version;
        };
      };

      packages = forAllSystems (system:
        let pkgs = nixpkgsFor.${system}; in
        {
          default = pkgs.tiny-azagent;
        });
      
      nixosModules.default = import ./nix/nixos-module.nix;

      nixosConfigurations.test-vm = nixpkgs.lib.nixosSystem {
        system = "x86_64-linux";
        pkgs = nixpkgsFor.x86_64-linux;
        modules = [
          ./nix/nixos-module.nix
          ./nix/test-vm.nix
        ];
      };
    };
}
