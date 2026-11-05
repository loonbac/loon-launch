// Detección de capacidades del host que condicionan el menú de poder.
//
// «Hibernar» sólo tiene sentido si el kernel puede escribir la imagen en un medio
// que sobreviva al apagado: `resume=` en el cmdline Y un swap que no sea zram.
// Ofrecerlo sin eso promete una restauración que no ocurre y arriesga el trabajo
// abierto del usuario.

/// Detecta si este host puede hibernar leyendo `/proc`.
///
/// Las tres condiciones juntas: `resume=` en el cmdline (sin él el kernel no
/// sabe de dónde restaurar), un swap que no sea zram (zram vive en RAM y no
/// sobrevive al apagado) y que ese swap sea al menos tan grande como la RAM
/// (una imagen menor puede abortar la operación a mitad).
pub fn host_can_hibernate() -> bool {
    let cmdline = std::fs::read_to_string("/proc/cmdline").ok();
    let swaps = std::fs::read_to_string("/proc/swaps").ok().unwrap_or_default();
    let mem_total = std::fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|m| mem_total_kib(&m))
        .unwrap_or(0);
    can_hibernate_with_swap_size(cmdline.as_deref(), &swaps, mem_total)
}

/// Decide con los datos ya leídos, para poder probarlo sin tocar `/proc`.
///
/// `cmdline` es `None` si no se pudo leer, y entonces se asume que no puede
/// hibernar: ante la duda, no se ofrece la acción.
pub fn can_hibernate(cmdline: Option<&str>, swaps: &str) -> bool {
    let Some(cmdline) = cmdline else {
        return false;
    };
    has_resume_arg(cmdline) && has_disk_swap(swaps)
}

/// Como [`can_hibernate`], exigiendo además que el swap cubra la RAM.
/// `mem_total_kib` en 0 (no medido) acepta cualquier swap de disco: no se puede
/// descartar por tamaño lo que no se conoce.
pub fn can_hibernate_with_swap_size(
    cmdline: Option<&str>,
    swaps: &str,
    mem_total_kib: u64,
) -> bool {
    let Some(cmdline) = cmdline else {
        return false;
    };
    has_resume_arg(cmdline) && has_adequate_swap(mem_total_kib, swaps)
}

/// `MemTotal` de `/proc/meminfo`, en KiB.
pub fn mem_total_kib(meminfo: &str) -> Option<u64> {
    meminfo.lines().find_map(|line| {
        let rest = line.strip_prefix("MemTotal:")?;
        rest.split_whitespace().next()?.parse().ok()
    })
}

/// Algún swap que no sea zram y que sea al menos tan grande como `mem_total_kib`.
///
/// Una imagen de hibernación ocupa aproximadamente la RAM en uso, así que un
/// swap menor puede quedarse corto y abortar la operación a mitad. Con
/// `mem_total_kib == 0` sólo se exige que exista (no se conoce la cota).
pub fn has_adequate_swap(mem_total_kib: u64, swaps: &str) -> bool {
    swap_entries(swaps).any(|entry| {
        let is_disk = !entry.device.starts_with("/dev/zram");
        let big_enough = mem_total_kib == 0 || entry.size_kib >= mem_total_kib;
        is_disk && big_enough
    })
}

/// Una entrada de `/proc/swaps` ya parseada.
struct SwapEntry {
    device: String,
    size_kib: u64,
}

/// Entradas de `/proc/swaps`, saltando la cabecera por nombre y no por posición.
/// El tamaño está en la tercera columna y se expresa en KiB.
fn swap_entries(swaps: &str) -> impl Iterator<Item = SwapEntry> + '_ {
    swaps.lines().filter_map(|line| {
        let mut fields = line.split_whitespace();
        let device = fields.next()?;
        if device == "Filename" {
            return None;
        }
        let kind = fields.next()?;
        if !matches!(kind, "partition" | "file") {
            return None;
        }
        let size_kib = fields.next().and_then(|s| s.parse().ok()).unwrap_or(0);
        Some(SwapEntry {
            device: device.to_string(),
            size_kib,
        })
    })
}

/// `resume=` seguido de un valor no vacío. `resume` suelto no habilita nada.
pub fn has_resume_arg(cmdline: &str) -> bool {
    cmdline.split_whitespace().any(|arg| {
        arg.strip_prefix("resume=")
            .is_some_and(|value| !value.is_empty())
    })
}

/// Algún swap que no sea zram.
///
/// Se usa para la comprobación sin tamaño; con tamaño disponible preferir
/// [`has_adequate_swap`], que además exige cubrir la RAM.
pub fn has_disk_swap(swaps: &str) -> bool {
    swap_entries(swaps).any(|entry| !entry.device.starts_with("/dev/zram"))
}
