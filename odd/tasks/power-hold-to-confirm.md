# Menú de poder: sin hibernar y con confirmación por mantener Enter

## Specs

- **S1** El menú de poder (`>`) ya no ofrece «Hibernar». Esta máquina no puede
  hibernar: `/proc/cmdline` no lleva `resume=` y el único swap es `zram0`
  (volátil). Salen: Cambiar fondo de pantalla, Apagar, Reiniciar, Suspender,
  Bloquear.

- **S2** Las acciones de poder destructivas o disruptivas —Apagar, Reiniciar,
  Suspender, Bloquear— **no se ejecutan con un Enter suelto**. Hay que mantener
  `Enter` presionado ~900 ms; mientras tanto la tarjeta seleccionada muestra una
  animación de progreso. Al soltar antes, se cancela y no pasa nada.

- **S3** «Cambiar fondo de pantalla» **no** pide confirmación: sigue abriendo el
  selector con un Enter normal. Tampoco la piden las apps normales.

- **S4** La animación es visible y comprensible: la tarjeta seleccionada gana una
  barra/clase de progreso que crece con el tiempo de pulsación y vuelve a cero al
  cancelar.

## Tasks

| ID | Spec | Ruta | Commit |
| --- | --- | --- | --- |
| T1 | S1 | `src/apps.rs`, `src/tests.rs` | `ecdc755` |
| T2 | S2, S3 | `src/ui/keys.rs`, `src/ui/mod.rs`, `src/filter.rs` | `ecdc755` |
| T3 | S4 | `src/ui/grid.rs`, `src/ui/styles.rs` | `ecdc755` |

## Log

**L1** (petición del dueño, verbatim):

> tengo otro problema mas tambien, creo que hay algunas opciones de loon launch
> de apagado reinicio que no deberia tener una pc o no tiene sentido que lo
> tenga, ademas que quiero que esas opciones para que funcione tenga que mantener
> presionado enter y salga una animacion ahi de que lo estoy manteniendo
> presionado y ahi recien funcione lo que iba a hacer, menos lo de cambiar de
> fondo de pantalla obvio

**L2** Decisión del dueño sobre hibernar (verbatim de la opción elegida):

> Quitarla — no funciona en tu PC

Evidencia: `/sys/power/state` = `freeze mem disk`; `/proc/cmdline` sin `resume=`;
`swapon` sólo `/dev/zram0`.
