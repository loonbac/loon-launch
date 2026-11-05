# El launcher como superficie layer-shell

Convertir la ventana del launcher en superficie `wlr-layer-shell` para que abrirla
no ocupe ni cree workspace en niri, sin cambiar nada más de la app.

## Specs

- **S1** En Wayland con soporte del compositor, la ventana es superficie de capa
  (capa overlay, teclado exclusivo, namespace `loon-launch`). Abrir el launcher
  sobre la última workspace vacía no crea una workspace nueva.
- **S2** En X11 o sin soporte del protocolo, la app mantiene la ventana normal:
  layer-shell es una mejora, no un requisito.
- **S3** Nada más cambia: geometría 550x372, grilla 3x5, paleta con acento del
  wallpaper, cabecera, modo fondos, daemon de instancia única, ocultar al perder
  foco.

## Tasks

- **T1** Módulo `ui/overlay.rs`: decisión pura `use_overlay` + aplicación a la
  ventana. Prueba: tabla de verdad y namespace estable.
- **T2** Dependencia `gtk4-layer-shell` en Cargo y en el paquete Nix.
- **T3** Verificación en niri: snapshots de workspaces antes/después y captura.

## Log

- **L1** «cuando abro el launcher se abre pero tambien me crea otro workspace de
  niri aparte, no me gusta eso, revisa si lo puedes arreglar, deberia quedarse en
  un solo workspace no deberia crearse otro solo porque abri mi app launcher»
- **L2** Causa confirmada: niri mantiene una workspace vacía al final de cada
  output; una ventana normal que se abre ahí hace que el compositor cree otra.
  Evidencia GTK: `[(2,1),(4,2),(5,3),(10,4,focus)]` → `[...(12,5)]`.
- **L3** La alternativa GPUI se descartó con medición (ver
  `odd/tasks/gpui-rewrite.md`): no mejora recursos ni latencia, y layer-shell se
  consigue igual en GTK.

## Evidencia

- `cargo test` → 26 passed, 0 failed. RED observado al forzar `use_overlay` a
  falso: el test nuevo falla; restaurado, verde.
- Tres aperturas consecutivas sobre la workspace vacía del final de DP-2:
  workspaces antes 4, después 4 en las tres.
- Captura: `/tmp/run1.png` — la superficie flota sobre el wallpaper, con la UI
  completa y el acento teal del wallpaper.

## Corrección de la revisión nativa (R3-001 / R4-001)

Las lentes `reliability` y `resilience` reportaron como CRITICAL que
`KeyboardMode::Exclusive` retiene el foco y anula el ocultar-al-perder-foco.
**Los hallazgos eran correctos.**

Mi primera verificación los declaró falsos positivos y estaba mal hecha: contaba
ventanas con `niri msg -j windows` filtrando por `app_id`, pero una superficie
layer nunca aparece en esa lista. La prueba devolvía `0` tanto con la capa
abierta como cerrada: no podía fallar, y por eso no demostraba nada.

Con `EventControllerFocus::connect_leave` instrumentado y `niri msg layers`:

| modo | `leave` se emite | la capa se oculta |
| :--- | :--- | :--- |
| `Exclusive` | nunca | no; queda encima de todo |
| `OnDemand` | sí | sí (`hide_window` la desmapea) |

niri no retira el foco a una capa de overlay con teclado exclusivo. El launcher
queda pegado encima de la ventana activa y sólo se cierra con su propio toggle.
El dueño del repositorio lo confirmó en su sesión real: «actualmente se queda
abierto al dar click fuera».

La segunda afirmación mía, que `OnDemand` rompía la escritura, también era
falsa: tecleando con `wtype` sobre la superficie abierta el texto aparece en la
entrada.

Corrección aplicada: `CONFIG.keyboard = KeyboardMode::OnDemand`. La prueba
`overlay_applies_on_demand_keyboard_on_the_overlay_layer` lo fija y falla si
vuelve a `Exclusive` (RED observado).

Nota de método: al medir comportamiento de capas hay que usar `niri msg layers`,
no el listado de ventanas, y hay que comprobar que el servicio del sistema no
esté interceptando el binario de prueba por D-Bus.
