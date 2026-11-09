# Módulo NixOS de loon-launch: instalar poniendo el nombre.
#
# Uso en el flake del sistema:
#   inputs.loon-launch.url = "github:loonbac/loon-launch";
#   # en modules:
#   imports = [ inputs.loon-launch.nixosModules.default ];
#   programs.loon-launch.enable = true;
#
# Sin overlay ni `specialArgs`: el paquete se evalúa con el nixpkgs del
# sistema que importa el módulo (`pkgs.callPackage ../default.nix`).
{ config, lib, pkgs, ... }:

let
  cfg = config.programs."loon-launch";
in
{
  options.programs."loon-launch" = {
    enable = lib.mkEnableOption "loon-launch, app launcher Wayland para niri (Super+Space, Super+B)";

    package = lib.mkOption {
      type = lib.types.package;
      default = pkgs.callPackage ../default.nix { };
      description = "Paquete loon-launch a instalar y ejecutar como servicio.";
    };

    enableService = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Levantar el daemon persistente como servicio de usuario (arranca oculto, el atajo alterna su visibilidad).";
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];

    systemd.user.services.loon-launch = lib.mkIf cfg.enableService {
      description = "loon-launch: app launcher Wayland para niri";
      wantedBy = [ "graphical-session.target" ];
      partOf = [ "graphical-session.target" ];
      after = [ "graphical-session.target" ];
      serviceConfig = {
        ExecStart = "${cfg.package}/bin/loon-launch";
        Restart = "on-failure";
      };
    };
  };
}
