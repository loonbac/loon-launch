# Hibernar condicional a la capacidad del host

## Specs

- **S1** «Hibernar» aparece en el menú de poder **sólo si** existe al menos un
  swap que **no** sea zram y cuyo tamaño cubra la RAM (`MemTotal` de
  `/proc/meminfo`). zram no cuenta: vive en RAM y su contenido desaparece al
  apagar. Un swap menor que la RAM puede abortar la operación a mitad.

- **S2** El estado actual de las dos máquinas: `nixos-pc` sólo tiene zram
  (16.3 G contra 31 GiB de RAM) → sin entrada; laptop tiene 16.8 G de swap en
  disco contra 15.3 G de RAM → **con** entrada.

- **S2b** **No** se exige `resume=` en `/proc/cmdline`. El initrd de NixOS deriva
  `resumeDevices` de `swapDevices` y escribe el `major:minor` en
  `/sys/power/resume` antes de montar el sistema, así que el cmdline no lleva
  `resume=` en esta configuración. Igual para `boot.resumeDevice`, que según su
  propia descripción es para swap **files**, no para particiones. Exigirlo
  ocultaba la capacidad en la laptop, que sí puede hibernar hoy.

- **S3** La detección es tolerante a fallos: si `/proc/swaps` no se puede leer,
  no hay swap que cumpla y la entrada no aparece. Si `MemTotal` no se puede
  leer, no se descarta por tamaño (no se oculta lo que no se pudo medir).

- **S4** El resto del menú no cambia, y las acciones que exigen mantener `Enter`
  siguen exigiéndolo (Hibernar también, cuando aparezca: es disruptiva).

## Tasks

| ID | Spec | Ruta | Commit |
| --- | --- | --- | --- |
| T1 | S1, S3 | `src/power.rs`, `src/tests.rs` | `890cf84` |
| T2 | S2, S4 | `src/apps.rs`, `src/filter.rs` | `890cf84` |

## Log

**L1** (decisión del dueño, tras la corrección de la sesión `.nixos`):

> Condicional a la capacidad del host

**L2** Corrección que la motivó (sesión `.nixos`, verbatim):

**L3** Corrección sobre el mecanismo de restauración (sesión `.nixos`):

> NixOS deriva `resumeDevices` de `swapDevices` automáticamente, excluyendo zram
> y cifrado (`stage-1.nix`). La laptop califica, así que el initrd ya escribe en
> `/sys/power/resume`. Y `boot.resumeDevice` **no aplica**: es para swap files.

Verificado en el stage-1 real (`/nix/store/ajhl56x6rk7a6xa49fs09cysya36ghp3-stage-1-init.sh`),
que contiene el UUID de swap de la laptop y escribe `$resumeMajor:$resumeMinor`
en `/sys/power/resume`. Además `boot.resumeDevice` ya estaba declarado en
`hosts/loon-laptop/platform.nix`.

> La laptop **tiene swap de disco suficiente**. Hibernar sería viable si se
> añadiera `resume=`; hoy no funciona, pero por configuración, no por hardware —
> al revés que `nixos-pc`, que solo tiene zram.

Medido: laptop RAM 15.3 G, swap 16.8 G en `/dev/nvme0n1p3`, `resume=` ausente.
