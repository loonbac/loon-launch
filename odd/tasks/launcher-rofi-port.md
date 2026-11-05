# Launcher de apps: port del layout rofi de gh0stzk (style_1)

Rama: `feat/launcher-rofi-port`. Referencia: `gh0stzk/dotfiles@master`,
`config/bspwm/config/rofi-themes/style_1.rasi` (el del screenshot) y `shared.rasi`.

## Specs

S1. La ventana del launcher de apps mide 550 px de ancho, con esquinas de 6 px
    y fondo `@background`.
S2. La grilla visible es de 3 columnas por 5 filas, en orden column-major; los
    items restantes continúan en columnas a la derecha.
S3. Cada celda lleva padding 8, spacing 10, radio 4, ícono de 24 px y texto
    alineado a la izquierda, centrado vertical.
S4. La celda seleccionada usa fondo `@selected` y texto `@background`.
S5. Colores y tipografía son tokens de una paleta única —equivalente a
    `shared.rasi`: `background`, `background-alt`, `foreground`, `selected`,
    `active`, `urgent`, más la fuente— y ninguna regla de estilo contiene
    colores literales.
S6. La cabecera dibuja la imagen escalada al ancho de la ventana y recortada al
    alto de la cabecera, centrada; con el chip de glifo y la entrada de búsqueda
    sobre `@background-alt` y radio 8.
S7. Se conserva el comportamiento existente: prefijo `>` para acciones de poder,
    `#` y `#!` para fondos, daemon de instancia única, ocultar al perder el foco,
    navegación por teclado.

## Tasks

- T1 — `src/ui/layout.rs` (métricas y colocación column-major) y
  `src/ui/palette.rs` (tokens). Cubre S1, S2, S3, S5.
- T2 — Estilos y celdas del grid con los tokens. Cubre S3, S4, S5.
- T3 — Cabecera escalada con chip y entrada. Cubre S6.
- T4 — Verificación: tests, `cargo check`, captura visual. Cubre S1–S7.

## Log

L1. «https://github.com/gh0stzk/dotfiles busca donde esta el codigo para su app
launcher ya que quiero portearlo a mi codigo, ya no quiero tener mi app
launcher, quiero migrar su codigo del app launcher a mi app launcher»

L2. «pues estamos en este proyecto no?, loon launch, eso quiero»

L3. «y si vas a programar usa agents.md para todo»

## Evidencia

- Commit `6e6e0b1` (T1-T3): layout, paleta, estilos, grilla y cabecera.
- `nix develop --command cargo test` → 23 passed, 0 failed.
- `nix develop --command cargo build` → sin errores.
- Captura en sesión real (niri, `dbus-run-session` + `grim`): ventana 550x372,
  grilla 3x5 column-major, selección invertida, cabecera con imagen.
- No verificado: modo fondos (`#`/`#!`) no se abrió en esta pasada; su cromo
  quedó con las mismas reglas que antes.
