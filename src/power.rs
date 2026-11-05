// Detección de capacidades del host que condicionan el menú de poder.
//
// «Hibernar» sólo tiene sentido si el kernel puede escribir la imagen en un medio
// que sobreviva al apagado: `resume=` en el cmdline Y un swap que no sea zram.
// Ofrecerlo sin eso promete una restauración que no ocurre y arriesga el trabajo
// abierto del usuario.

/// Detecta si este host puede hibernar leyendo `/proc`.
pub fn host_can_hibernate() -> bool {
    let cmdline = std::fs::read_to_string("/proc/cmdline").ok();
    let swaps = std::fs::read_to_string("/proc/swaps").ok().unwrap_or_default();
    can_hibernate(cmdline.as_deref(), &swaps)
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

/// `resume=` seguido de un valor no vacío. `resume` suelto no habilita nada.
pub fn has_resume_arg(cmdline: &str) -> bool {
    cmdline.split_whitespace().any(|arg| {
        arg.strip_prefix("resume=")
            .is_some_and(|value| !value.is_empty())
    })
}

/// Algún swap que no sea zram.
///
/// zram vive en RAM: su contenido desaparece al apagar, así que no sirve como
/// respaldo de una imagen de hibernación. La cabecera de `/proc/swaps` empieza
/// por `Filename`, así que se descarta por nombre y no por posición: así la
/// función también acepta contenido sin cabecera.
pub fn has_disk_swap(swaps: &str) -> bool {
    swaps.lines().any(|line| {
        let mut fields = line.split_whitespace();
        let Some(device) = fields.next() else {
            return false;
        };
        if device == "Filename" {
            return false;
        }
        // Un swap de disco declara su tamaño; zram y las líneas vacías no pasan.
        let is_swap = fields.next().is_some_and(|kind| {
            matches!(kind, "partition" | "file")
        });
        is_swap && !device.starts_with("/dev/zram") && device != "/dev/zram0"
    })
}
