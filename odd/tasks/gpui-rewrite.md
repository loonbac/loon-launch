# Migración del launcher de GTK4 a GPUI

Reescribir la capa de UI de `loon-launch` sobre GPUI (Zed), pineado a un rev del
monorepo que ya trae `wlr-layer-shell`, para que el launcher sea una superficie
overlay que no ocupa workspace en niri ni carga el stack GTK.

## Specs

- **S1** El launcher se presenta como superficie `wlr-layer-shell` en la capa
  overlay, con teclado exclusivo; abrirlo no debe crear ni cambiar de workspace.
  Evidencia: snapshot de `niri msg -j workspaces` antes/después, sin workspaces nuevos.
- **S2** Geometría y contenido conservan la paridad ya verificada: ventana 550x372,
  grilla de 3 columnas por 5 filas en orden column-major, ícono de 24 px, padding
  de celda 8, radio 4, selección invertida.
- **S3** La paleta sigue siendo una sola fuente de verdad y la selección usa el
  acento del wallpaper (`~/.config/mpvpaper/accent.txt`), con caída al valor por
  defecto cuando el token falta o es inválido.
- **S4** Cabecera: imagen escalada tipo cover, chip de glifo y entrada de búsqueda.
  Si la imagen de cabecera no existe, degradado integrado.
- **S5** El comportamiento del daemon se mantiene: instancia única, Super+Space
  alterna, ocultar al perder foco, navegación por teclado y prefijos `>` (poder)
  y `#`/`#!` (fondos).
- **S6** Presupuesto de recursos declarado y medido, no prometido: RSS del proceso
  residente y latencia de apertura, comparados contra la línea base GTK
  (RSS 224 MB medido el 2026-11-05).

## Tasks

- **T1** Pinear `gpui` al rev de Zed con layer-shell y dejar el build en el flake
  (deps nativas en devShell y en el paquete). Prueba: `cargo build` del crate.
- **T2** Ventana layer-shell + ciclo de vida (mostrar/ocultar, foco, Escape).
  Prueba: S1 con snapshots de workspaces en niri.
- **T3** Grilla 3x5 column-major con iconos y selección (S2, S3).
- **T4** Cabecera con imagen cover, chip y entrada (S4).
- **T5** Modo fondos y prefijos, paridad con la versión GTK (S5).
- **T6** Medición de RSS y latencia contra la línea base, y captura visual (S6).

## Log

- **L1** «ohh veo que estas usando gtk y esa vaina, quiero que uses gpui, migra a
  eso, asi ira muchisimo mas rapido y ahorraremos recursos»
- **L2** Antes de migrar se verificó el crate publicado: `gpui` 0.2.2 **no** expone
  layer-shell (`grep -rl layer_shell` vacío sobre su fuente; `WindowOptions.kind`
  solo tiene ventanas normales/popup). El soporte entró en Zed el 2025-10-29
  (PR #35610), posterior a ese release.
- **L3** Decisión del usuario: migrar igual, pineando el monorepo de Zed para
  tener layer-shell, en lugar de aceptar que el bug de workspace siguiera vivo.

## Evidencia T1-T2 (spike, 2026-11-05)

- **Build**: `gpui` pineado a `b92664c52d7c49bb49acd06d2d40eb23b4e8c601`
  compila con `rustc 1.95.0` del dev shell, tras parchear `xattr 0.2.3`
  (vía `async-tar` ← `http_client` de Zed): declaraba `ENOATTR` sin `cfg` y no
  compila en Linux. Fix vendorizado en `spike-gpui/vendor/xattr`.
- **Entorno**: GPUI carga `libwayland` y el loader de Vulkan con `dlopen`, así
  que el devShell necesita `LD_LIBRARY_PATH` real (`lib.makeLibraryPath`), no
  solo los `-dev` de `buildInputs`.
- **Layer-shell**: funciona. `WindowKind::LayerShell` (`Layer::Overlay`,
  `KeyboardInteractivity::Exclusive`) abre la superficie centrada sobre el
  wallpaper y **no crea workspace**:

      ANTES:   [(2,1),(4,2),(5,3),(13,4,focus)] en DP-2 + [(11,1)] en HDMI
      DESPUÉS: idéntico

  Con la versión GTK el mismo escenario creaba `(12,5)`. S1 queda demostrado
  como propiedad de la arquitectura, no como parche.
- **Recursos (dev build, 15 filas, sin cabecera ni iconos reales)**:
  RSS 241,5 MB · VSZ 6,8 GB · 29 hilos.
  Línea base GTK (app completa: iconos, cabecera, búsqueda, modo fondos):
  RSS 224,1 MB · 7 hilos.
- **Conclusión de S6**: la migración **no reduce** el consumo; lo aumenta con
  menos funcionalidad. La ganancia real de GPUI aquí es layer-shell y una API
  más moderna, no el rendimiento.

## Evidencia de latencia (2026-11-05)

Medición: comando → primer cuadro visible. Para GTK se usa el camino real del
usuario (daemon residente + toggle); para GPUI el spike no tiene daemon, así que
se instrumentó el primer `render()` y se mide proceso → primer cuadro.

- **GTK, toggle real (daemon instalado)**: 13, 30, 31 ms → mediana **30 ms**.
- **GPUI, arranque en frío hasta el primer cuadro**: 338, 356, 367 ms → mediana
  **356 ms**.
- No fue posible medir "GTK en frío con ventana visible": el launcher arranca
  oculto por diseño (daemon) y solo presenta tras la segunda invocación, así que
  la cifra comparable a GPUI en frío exigiría instrumentar la app igual.

Lectura: GPUI no es más rápido en el camino que el usuario percibe. Su arranque
en frío es ~12x la latencia del toggle actual, y para igualar los 30 ms
necesitaría la misma arquitectura de daemon residente, que es independiente del
toolkit. S6 queda respondido: sin mejora medible de recursos ni de latencia.

## Decisión (2026-11-05)

Migración **rechazada** con datos: no mejora recursos ni latencia, y la única
ganancia (layer-shell) se consigue en GTK con `gtk4-layer-shell`. La rama con el
spike se descartó; este documento queda como registro de la medición y de los
obstáculos encontrados (parche de `xattr`, `LD_LIBRARY_PATH` para las librerías
que GPUI carga con `dlopen`, pin del monorepo de Zed).

El fix que sí se implementa es S1 sobre GTK: superficie layer-shell en la capa
overlay, para que abrir el launcher no cree workspace en niri.
