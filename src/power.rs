// Detección de capacidades del host que condicionan el menú de poder.
//
// «Hibernar» sólo tiene sentido si el kernel puede escribir la imagen en un medio
// que sobreviva al apagado y con espacio suficiente para ella. Ofrecerlo sin eso
// promete una restauración que no ocurre y arriesga el trabajo abierto.
//
// La condición es **una sola**: existe un swap que no sea zram y cuyo tamaño
// cubre la RAM. Lo de «dónde restaurar» no se comprueba aquí a propósito: el
// initrd de NixOS deriva `resumeDevices` de `swapDevices` y escribe el major:minor
// en `/sys/power/resume` antes de montar el sistema (`stage-1-init.sh`), sin
// necesidad de `resume=` en el cmdline ni de `boot.resumeDevice` para particiones.
// Exigir `resume=` en `/proc/cmdline` ocultaría la capacidad en hosts que sí
// pueden hibernar, que es justo el error que hay que evitar.

/// Detecta si este host puede hibernar leyendo `/proc`.
pub fn host_can_hibernate() -> bool {
    let swaps = std::fs::read_to_string("/proc/swaps").ok().unwrap_or_default();
    let mem_total = std::fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|m| mem_total_kib(&m))
        .unwrap_or(0);
    can_hibernate(&swaps, mem_total)
}

/// Decide con los datos ya leídos, para poder probarlo sin tocar `/proc`.
///
/// `mem_total_kib` en 0 (no medido) acepta cualquier swap de disco: no se puede
/// descartar por tamaño lo que no se conoce.
pub fn can_hibernate(swaps: &str, mem_total_kib: u64) -> bool {
    has_adequate_swap(mem_total_kib, swaps)
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
/// swap menor puede quedarse corto y abortar la operación a mitad. zram vive en
/// RAM: aunque sea enorme, su contenido desaparece al apagar y no sirve como
/// respaldo. Con `mem_total_kib == 0` sólo se exige que exista (no hay cota).
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
