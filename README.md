# loon-launch

App launcher Wayland para [niri](https://github.com/YaLTeR/niri), escrito en Rust
con GTK4 + libadwaita. Lista las aplicaciones instaladas, filtra al escribir,
navega con el teclado y ofrece acciones de poder con el prefijo `>`. Incluye un
selector de fondos de pantalla con miniaturas reales.

Vive en su propio repositorio y se consume desde
[`loon-flakes`](https://github.com/loonbac/loon-flakes) como input de flake.

## Instalación

Dependencias del sistema: GTK4 y libadwaita. La `gtk4-layer-shell` viaja
vendida dentro de los paquetes, así que no hace falta instalarla.

### NixOS (recomendado)

El módulo declara el paquete y el servicio de usuario: basta con el nombre, sin
overlay ni `specialArgs`.

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
    loon-launch.url = "github:loonbac/loon-launch";
  };

  outputs = { nixpkgs, loon-launch, ... }: {
    nixosConfigurations.mi-maquina = nixpkgs.lib.nixosSystem {
      modules = [
        loon-launch.nixosModules.default
        {
          programs.loon-launch.enable = true;
        }
      ];
    };
  };
}
```

En cualquier otra distro con Nix:
`nix profile install github:loonbac/loon-launch`.

### Arch y otras distribuciones (tarball portable)

Descarga `loon-launch-<version>-x86_64-linux.tar.gz` de la última
[release](https://github.com/loonbac/loon-launch/releases), descomprime y sigue
las instrucciones de `INSTALL.txt`:

```bash
sudo pacman -S gtk4 libadwaita     # opcional: ffmpeg, para miniaturas de vídeo
tar -xzf loon-launch-*-x86_64-linux.tar.gz
less loon-launch-*/INSTALL.txt
sudo cp -a loon-launch-*/bin loon-launch-*/lib loon-launch-*/share /usr/local/
loon-launch
```

### Debian / Ubuntu (.deb)

```bash
sudo apt install ./loon-launch-<version>.deb
```

### Fedora (.rpm)

```bash
sudo dnf install ./loon-launch-<version>.rpm
```

Ambos paquetes dejan el binario en `/usr/bin`, la entrada `.desktop` y las
cabeceras en las rutas del sistema. Se compilan contra glibc 2.36, así que
funcionan en Debian 12+, Ubuntu 22.04+ y Fedora 36+.

## Uso

| Atajo | Acción |
| --- | --- |
| `Super+Space` | Abrir el launcher |
| `Super+B` (o `loon-launch wallpapers`) | Selector de fondos |

Navegación: `←→↑↓` mueven la selección por el grid, `Enter` ejecuta, `Escape`
cierra y `Backspace` borra. Los caracteres imprimibles filtran la lista aunque
el campo de búsqueda no tenga el foco.

El prefijo `>` entra en modo poder (apagar, reiniciar, suspender, cerrar sesión,
cambiar fondo de pantalla...). El selector de fondos se abre **sólo** con esa
acción; dentro de él, escribir filtra por nombre y `Tab` alterna fondos animados
y estáticos.

Tus propios fondos van en `~/.loon-launch/wallpapers/`: se crea solo con las
imágenes que trae la app (sección "Mis fondos"); suelta ahí imágenes o videos.

## Arquitectura

El proceso es un daemon persistente: arranca oculto con la sesión y el atajo
alterna su visibilidad mediante la instancia única de `GtkApplication`, así que
abre al instante. La lista de apps se recarga en cada apertura, de modo que una
app instalada aparece sin reiniciar nada.

| Archivo | Responsabilidad |
| --- | --- |
| `src/main.rs` | Arranque, instancia única, ciclo de presentación |
| `src/apps.rs` | Descubrimiento de `.desktop` |
| `src/filter.rs` | Filtrado y modos de búsqueda (`>`, fondos) |
| `src/wallpapers.rs` | Listado de fondos y miniaturas |
| `src/models.rs` | Modelo de ítem y acciones |
| `src/ui/` | Ventana, banner, grid, carrusel y estilos |

## Desarrollo

```bash
nix develop --command cargo test     # tests
nix develop --command cargo check    # compilación
nix build                            # paquete completo (incluye checkPhase)
nix run                              # ejecutar
```

`cargo test` necesita las librerías nativas en `PKG_CONFIG_PATH`; por eso se usa
`nix develop` y no el `cargo` del PATH.

## Integración

En `loon-flakes`:

```nix
inputs.loon-launch = {
  url = "github:loonbac/loon-launch";
  inputs.nixpkgs.follows = "nixpkgs";
};
```

El paquete llega a los módulos por el módulo `programs.loon-launch.enable` (sin `specialArgs`) y el binario se ejecuta como
servicio de usuario `loon-launch.service`. Actualizar la versión del sistema es
mover el input (`nix flake update loon-launch`), nunca editar un pin a mano.

## Créditos y licencias de terceros

Las imágenes de cabecera (`assets/headers/`, 201 archivos) provienen de
[`DerpFest-AOSP/packages_apps_OmniStyle`](https://github.com/DerpFest-AOSP/packages_apps_OmniStyle/tree/15/res/drawable-nodpi),
que **no incluye archivo de licencia**: el arte no está licenciado explícitamente
por sus autores. Se usan aquí para uso personal, sin reclamar propiedad, y con
política de retirada: si eres dueño del arte y no quieres que esté en este
repositorio, abre un
[issue](https://github.com/loonbac/loon-launch/issues) y se borra. Detalles y
modificaciones locales en [`THIRD-PARTY.md`](THIRD-PARTY.md).
