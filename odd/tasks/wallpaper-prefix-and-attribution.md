# Fondos sólo por `>` y atribución de las cabeceras de OmniStyle

## Specs

- **S1** Teclear `#` en el campo de búsqueda **no** abre el selector de fondos.
  `#` deja de ser prefijo de modo: se filtra como cualquier otro carácter, y como
  ninguna app se llama `#`, el grid queda vacío. Lo mismo para `#!`.

- **S2** El selector de fondos se abre **únicamente** con el prefijo `>` y su
  acción «Cambiar fondo de pantalla». Dentro del selector, escribir filtra por
  nombre; `Tab` alterna fondos animados y estáticos.

- **S3** Las cabeceras provienen de
  [OmniStyle](https://github.com/DerpFest-AOSP/packages_apps_OmniStyle/tree/15/res/drawable-nodpi)
  y **ese repositorio no incluye archivo de licencia**, así que el arte no está
  licenciado explícitamente por sus autores. El repo del launcher lo declara en
  `THIRD-PARTY.md` y en el `README`: origen, ausencia de licencia y política de
  retirada — si alguien es dueño del arte y no quiere que esté aquí, abre un
  issue y se borra.

## Tasks

| ID | Spec | Ruta | Commit |
| --- | --- | --- | --- |
| T1 | S1 | `src/filter.rs`, `src/ui/grid.rs`, `src/ui/keys.rs` | — |
| T2 | S2 | `src/filter.rs`, `src/ui/grid.rs`, `src/ui/mod.rs` | — |
| T3 | S3 | `THIRD-PARTY.md`, `README.md` | — |

## Log

**L1** (petición del dueño, verbatim):

> es verdad, cuando presiono # se cambia a los fondos, pero no me gusta eso, para
> cambiar fondos es unicamente con > y ya, lo de omnistyle arreglalo tu, pon que
> no tienes licencia y pon de donde vienen, si alguien es dueno del arte y no
> quiere que esten aqui abrir issue y lo borramos

**L2** Aclaración previa del dueño sobre el alcance de las cabeceras (verbatim):

> yo quiero mi set de imagenes de cabecera nada mas, asi esta planeado funcionar
