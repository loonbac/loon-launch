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
`KeyboardMode::Exclusive` retiene el foco de teclado y anula el
ocultar-al-perder-foco. El refuter los corroboró por inferencia, con
`probe unavailable on pi`: nadie lo midió.

**Medido en niri** (launch -> `niri msg action focus-window` a otra ventana):

```
antes:  focused = dev.loonbac.loonlaunch
después: focused = chrome-...-Default
superficies loon-launch mapeadas: 0
```

El compositor sí retira el foco y el launcher se desmapea: **falso positivo**.
La consecuencia práctica de aplicar la corrección propuesta: con `OnDemand` la
superficie no recibe el teclado al aparecer (el default del protocolo es `None`
y `OnDemand` enfoca sólo tras un clic), así que la entrada de búsqueda deja de
escribirse. Con `None`, lo mismo.

Lo que sí faltaba, y es el hueco real que R3-001 señala, es que la prueba
cubría sólo la función pura `use_overlay` y no la configuración que `apply`
aplica. Ahora `overlay::CONFIG` declara capa, modo de teclado, zona exclusiva y
namespace como dato, y `apply` la consume; la prueba la afirma y falla si el
modo cambia a `OnDemand`.

Evidencia: `cargo test` -> 27 pasan (RED observado al forzar `OnDemand` en
`CONFIG`, GREEN restaurado).
