# Hibernar condicional a la capacidad del host

## Specs

- **S1** «Hibernar» aparece en el menú de poder **sólo si** el host puede
  hibernar de verdad, y eso exige **las dos** condiciones:
  - `resume=` presente en `/proc/cmdline`, y
  - al menos un swap que **no** sea zram (en `/proc/swaps`), es decir respaldo
    en disco que sobrevive al apagado.

- **S2** Si falta cualquiera de las dos, la entrada no aparece. El estado actual de
  las dos máquinas es: `nixos-pc` sólo tiene zram → sin entrada; laptop tiene
  swap de disco pero no `resume=` → sin entrada hasta que se configure.

- **S3** La detección es tolerante a fallos: si `/proc/cmdline` o `/proc/swaps`
  no se pueden leer, se asume «no puede hibernar» y la entrada no aparece. Nunca
  se ofrece una acción que podría no completarse.

- **S4** El resto del menú no cambia, y las acciones que exigen mantener `Enter`
  siguen exigiéndolo (Hibernar también, cuando aparezca: es disruptiva).

## Tasks

| ID | Spec | Ruta | Commit |
| --- | --- | --- | --- |
| T1 | S1, S3 | `src/power.rs`, `src/tests.rs` | — |
| T2 | S2, S4 | `src/apps.rs`, `src/filter.rs` | — |

## Log

**L1** (decisión del dueño, tras la corrección de la sesión `.nixos`):

> Condicional a la capacidad del host

**L2** Corrección que la motivó (sesión `.nixos`, verbatim):

> La laptop **tiene swap de disco suficiente**. Hibernar sería viable si se
> añadiera `resume=`; hoy no funciona, pero por configuración, no por hardware —
> al revés que `nixos-pc`, que solo tiene zram.

Medido: laptop RAM 15.3 G, swap 16.8 G en `/dev/nvme0n1p3`, `resume=` ausente.
