# Cabeceras elegidas por el color del acento

La cabecera del launcher deja de ser una imagen fija: cada apertura elige una
imagen del catálogo cuyo color dominante combina con el acento del wallpaper.

## Specs

- **S1** El catálogo vive en `assets/headers/`, versionado con el repo e instalado
  en `share/loon-launch/headers` (buscado por `XDG_DATA_DIRS`).
- **S2** Cada apertura del launcher elige al azar entre las cabeceras cuyo tono
  cae a ±30° del acento del wallpaper (`~/.config/mpvpaper/accent.txt`).
- **S3** Si el acento falta, es gris o ninguna imagen combina, se usa el catálogo
  completo: el launcher nunca queda sin cabecera.
- **S4** La imagen se escala tipo cover: cubre la cabecera y recorta a los lados.
- **S5** Una imagen que no se pueda decodificar se salta y se prueba la siguiente.
- **S6** Formatos: solo los que el sistema decodifica sin plugins extra (JPEG, PNG
  con transparencia). Nada de WebP.

## Tasks

- **T1** `tools/build_header_index.py`: índice de tono/saturación/luminosidad.
- **T2** `src/ui/headers.rs`: parser del índice, tono del acento, ventana de
  selección y elección reproducible.
- **T3** `src/ui/banner.rs`: cabecera reemplazable por apertura, dibujada cover.
- **T4** Empaquetado en `default.nix` + pruebas y verificación visual.

## Log

- **L1** «seria bueno que ahora configuremos tambien la imagen de fondo en ves de
  usar la que pusiste de mi wallpaper, https://github.com/DerpFest-AOSP/packages_apps_OmniStyle/blob/15/res/drawable-nodpi clona todo lo que esta aqui en esta carpeta, esta llena de imagenes [...] todas estas imagenes de aca se usaran, se ajustara a la pantalla verticalmente no importa que tamano tenga [...] ademas saldran imagenes aleatorias cada que se abre el app launcher pero solo segun el color del accent del fondo de pantalla»
- **L2** «hay que convertir los webp a otro formato compatible» — el sistema no
  tiene loader de WebP: 234 de 400 imágenes eran WebP.
- **L3** Poda pedida por el usuario, en este orden: Naruto (27), `bootleg_*` (14),
  `cl_nicehp_19` (1), `comic_*` (18), `darko_blur_*` (6), `derp_header_*` (51),
  `du_crot_*` (5), `space_*` (26), `uns_aqua_*` (25), `uns_holo_*` (14),
  `uns_pattern(s)_*` (24). «solo esos creo, continua».

## Evidencia

- `cargo test` → 29 passed, 0 failed (5 tests nuevos: parser del índice, tono
  circular, ventana de acento, fallback de candidatas, catálogo real contra el
  acento del sistema).
- RED observado en el test de la ventana de acento al neutralizar el filtro.
- Captura en niri: la cabecera sale de las imágenes y combina con el acento teal.
- Catálogo final: 201 imágenes, 12 MB, todas JPEG salvo tres PNG con transparencia.
