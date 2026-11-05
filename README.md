# loon-launch

App launcher Wayland para [niri](https://github.com/YaLTeR/niri), escrito en Rust
con GTK4 + libadwaita. Lista las aplicaciones instaladas, filtra al escribir,
navega con el teclado y ofrece acciones de poder con el prefijo `>`. Incluye un
selector de fondos de pantalla con miniaturas reales.

Vive en su propio repositorio y se consume desde
[`loon-flakes`](https://github.com/loonbac/loon-flakes) como input de flake.

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

El paquete llega a los módulos por `specialArgs` y el binario se ejecuta como
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
