// Lógica pura (testeable): filtrado de apps y movimiento de selección.
use crate::models::Item;

/// Milisegundos que hay que mantener `Enter` para que una acción de poder se
/// ejecute. Suficiente para que la pulsación sea deliberada y no un roce.
pub const HOLD_MS: u64 = 900;

/// Acciones que exigen mantener `Enter` antes de ejecutarse.
///
/// Sólo las que interrumpen la sesión o apagan la máquina. El selector de fondos
/// y las apps normales abren con un `Enter` suelto: no hay nada que confirmar.
pub fn requires_hold(exec: &str) -> bool {
    let exec = exec.trim();
    if exec.is_empty() || exec == "wallpaper-mode" {
        return false;
    }
    // Se compara por la acción real del sistema, no por el texto visible: el
    // nombre de la etiqueta puede cambiar sin alterar el comportamiento.
    const DISRUPTIVE: [&str; 5] = [
        "systemctl poweroff",
        "systemctl reboot",
        "systemctl suspend",
        "systemctl hibernate",
        "loginctl lock-session",
    ];
    DISRUPTIVE.contains(&exec)
}

/// Progreso de la pulsación, de 0.0 a 1.0, para la barra de la tarjeta.
pub fn hold_progress(elapsed_ms: u64) -> f64 {
    if HOLD_MS == 0 {
        return 1.0;
    }
    (elapsed_ms as f64 / HOLD_MS as f64).clamp(0.0, 1.0)
}

/// Si la pulsación ya cumplió el umbral y la acción debe ejecutarse.
pub fn hold_is_armed(elapsed_ms: u64) -> bool {
    elapsed_ms >= HOLD_MS
}

/// Qué lista se está mostrando. Es estado explícito de la UI, no algo que el
/// usuario teclea: el selector de fondos se abre con la acción `>` («Cambiar
/// fondo de pantalla») y `Tab` alterna entre animados y estáticos.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchMode {
    /// Lista de aplicaciones (y acciones de poder con el prefijo `>`).
    Apps,
    /// Selector de fondos: `static_only` muestra imágenes en vez de videos.
    Wallpapers { static_only: bool },
}

/// Filtra la lista según el modo y la query.
/// - En `Apps`, el prefijo `>` filtra acciones de poder.
/// - En `Wallpapers`, no hay prefijo: el texto filtra por nombre.
/// - `#` no es un prefijo de modo: se trata como texto normal.
pub fn filter_items(
    all_apps: &[Item],
    power: &[Item],
    wallpapers: &[Item],
    query: &str,
    mode: SearchMode,
) -> Vec<Item> {
    let q = query.to_lowercase();
    let mut shown: Vec<Item> = Vec::new();

    match mode {
        SearchMode::Wallpapers { static_only } => {
            let filter = q.trim().to_string();
            for w in wallpapers {
                if w.is_header
                    || !w.is_wallpaper
                    || w.is_animated_wallpaper == static_only
                {
                    continue;
                }
                if filter.is_empty() || w.name.to_lowercase().contains(&filter) {
                    shown.push(w.clone());
                }
            }
        }
        SearchMode::Apps => {
            if let Some(rest) = q.strip_prefix('>') {
                let filter = rest.trim();
                for p in power {
                    if filter.is_empty() || p.name.to_lowercase().contains(filter) {
                        shown.push(p.clone());
                    }
                }
            } else {
                for app in all_apps {
                    if app.name.to_lowercase().contains(&q) {
                        shown.push(app.clone());
                    }
                }
            }
        }
    }
    shown
}

/// Aplica un movimiento de selección (delta en celdas) y devuelve el nuevo índice.
/// - si no hay items devuelve -1
/// - si el índice actual es inválido, se ancla a 0
/// - clampa entre 0 y total-1
pub fn move_selection(sel: i32, delta: i32, total: usize) -> i32 {
    if total == 0 {
        return -1;
    }
    let mut idx = if sel < 0 { 0 } else { sel + delta };
    if idx < 0 {
        idx = 0;
    } else if idx >= total as i32 {
        idx = total as i32 - 1;
    }
    idx
}

/// Navegación en la lista de ROWS filas por columna:
/// - Abajo/Arriba se mueven +/-1 fila (dentro de la columna).
/// - Derecha/Izquierda se mueven en saltos de ROWS (siguiente/anterior columna).
pub fn move_sel_rowwise(sel: i32, delta: i32, total: usize) -> i32 {
    move_selection(sel, delta, total)
}

/// Avanza por un carrusel y envuelve en ambos extremos.
pub fn move_selection_wrap(sel: i32, delta: i32, total: usize) -> i32 {
    if total == 0 {
        return -1;
    }
    let current = if sel < 0 || sel >= total as i32 { 0 } else { sel };
    (current + delta).rem_euclid(total as i32)
}

/// Navegación 2D: `positions[i] = (row, col)` del item seleccionable i.
/// Abajo/arriba buscan la fila siguiente en la misma columna (o la más cercana).
/// Izquierda/derecha se mueven solo dentro de la fila. Si no hay celda, se queda.
pub fn move_sel_grid(sel: i32, drow: i32, dcol: i32, positions: &[(i32, i32)]) -> i32 {
    if positions.is_empty() {
        return -1;
    }
    let idx = if sel < 0 { 0 } else { sel as usize };
    if idx >= positions.len() {
        return 0;
    }
    if drow == 0 && dcol == 0 {
        return idx as i32;
    }
    let (r, c) = positions[idx];
    if dcol != 0 && drow == 0 {
        let target_c = c + dcol;
        return positions
            .iter()
            .enumerate()
            .find(|(_, &(rr, cc))| rr == r && cc == target_c)
            .map(|(i, _)| i as i32)
            .unwrap_or(idx as i32);
    }
    let mut best: Option<(i32, i32, i32)> = None;
    for (i, &(rr, cc)) in positions.iter().enumerate() {
        let rd = rr - r;
        let same_dir = if drow > 0 { rd > 0 } else { rd < 0 };
        if !same_dir {
            continue;
        }
        let key = (rd.abs(), (cc - c).abs(), i as i32);
        if best.is_none_or(|cur| key < cur) {
            best = Some(key);
        }
    }
    best.map(|(_, _, i)| i).unwrap_or(idx as i32)
}

/// Normaliza la selección tras un repopulate: si quedó fuera de rango, 0.
pub fn normalize_selection(sel: i32, total: usize) -> i32 {
    if total == 0 {
        -1
    } else if sel < 0 || sel >= total as i32 {
        0
    } else {
        sel
    }
}

/// Escribe un carácter en el texto de búsqueda.
pub fn apply_char(text: &str, c: char) -> String {
    let mut s = text.to_string();
    s.push(c);
    s
}

/// Borra el último carácter.
pub fn apply_backspace(text: &str) -> String {
    let mut s = text.to_string();
    s.pop();
    s
}
