{
  description = "loon-launch — app launcher Wayland para niri (Rust/GTK4)";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
      loon-launch = pkgs.callPackage ./default.nix { };
    in
    {
      packages.${system} = {
        inherit loon-launch;
        default = loon-launch;
      };

      # Entorno con las librerías nativas que `pkg-config` necesita, para que
      # `nix develop --command cargo test` funcione sin montar el nix-shell a
      # mano. `inputsFrom` replica buildInputs y nativeBuildInputs del paquete.
      devShells.${system}.default = pkgs.mkShell {
        inputsFrom = [ loon-launch ];
      };

      # Instalación por nombre en NixOS, sin overlay ni specialArgs:
      # `imports = [ inputs.loon-launch.nixosModules.default ];`
      # `programs.loon-launch.enable = true;`
      nixosModules.default = import ./nix/module.nix;
    };
}
