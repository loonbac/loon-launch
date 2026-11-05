// Cabeceras del launcher: imágenes elegidas por color en cada apertura.
//
// El launcher ya sigue el acento del wallpaper. La cabecera hace lo mismo con
// las imágenes: cada apertura elige al azar entre las que tienen un tono
// cercano al del acento, así el header no desentona con el escritorio.
//
// El índice de color se calcula una sola vez (`tools/build_header_index.py`) y
// viaja con las imágenes, de modo que abrir el launcher no decodifica cientos de
// archivos: elige sobre una tabla ya calculada.

use std::path::{Path, PathBuf};

/// Diferencia máxima de tono, en grados, entre el acento y la cabecera.
pub const HUE_WINDOW: f64 = 30.0;

/// Una cabecera con su tono dominante, tal como viene del índice.
#[derive(Debug, Clone, PartialEq)]
pub struct Header {
    pub file: String,
    pub hue: f64,
}

/// Lee el índice `index.tsv` (archivo, tono, saturación, luminosidad).
///
/// Las líneas malformadas se descartan: el índice lo genera un script y una
/// entrada rota no debe impedir que el launcher abra.
pub fn parse_index(text: &str) -> Vec<Header> {
    text.lines()
        .filter(|line| !line.trim_start().starts_with('#') && !line.trim().is_empty())
        .filter_map(|line| {
            let fields: Vec<&str> = line.split('\t').map(str::trim).collect();
            // Cuatro columnas: archivo, tono, saturación, luminosidad.
            if fields.len() != 4 {
                return None;
            }
            let file = fields[0];
            let hue: f64 = fields[1].parse().ok()?;
            if file.is_empty() || !(0.0..=360.0).contains(&hue) {
                return None;
            }
            Some(Header {
                file: file.to_string(),
                hue,
            })
        })
        .collect()
}

/// Tono del acento en grados, o `None` si el hex no es válido.
pub fn hue_of(hex: &str) -> Option<f64> {
    let digits = hex.trim().strip_prefix('#')?;
    if digits.len() != 6 {
        return None;
    }
    let channel = |range: std::ops::Range<usize>| u8::from_str_radix(&digits[range], 16).ok();
    let (r, g, b) = (channel(0..2)?, channel(2..4)?, channel(4..6)?);
    let (r, g, b) = (
        f64::from(r) / 255.0,
        f64::from(g) / 255.0,
        f64::from(b) / 255.0,
    );
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    if delta.abs() < f64::EPSILON {
        // Gris: el tono no está definido y no sirve para emparejar.
        return None;
    }
    let hue = if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * ((b - r) / delta + 2.0)
    } else {
        60.0 * ((r - g) / delta + 4.0)
    };
    Some((hue + 360.0) % 360.0)
}

/// Distancia circular entre dos tonos, en grados.
fn hue_distance(a: f64, b: f64) -> f64 {
    let raw = (a - b).abs() % 360.0;
    raw.min(360.0 - raw)
}

/// Cabeceras cuyo tono cae dentro de la ventana del acento.
pub fn matching(headers: &[Header], accent_hue: f64) -> Vec<&Header> {
    headers
        .iter()
        .filter(|header| hue_distance(header.hue, accent_hue) <= HUE_WINDOW)
        .collect()
}

/// Elige una cabecera al azar entre las que combinan con el acento.
///
/// `roll` hace la elección reproducible en pruebas; en ejecución es un número
/// del reloj. Sin acento, o si ninguna combina, se usan todas antes que quedarse
/// sin cabecera.
pub fn pick<'a>(headers: &'a [Header], accent_hue: Option<f64>, roll: u64) -> Option<&'a Header> {
    if headers.is_empty() {
        return None;
    }
    let candidates = match accent_hue {
        Some(hue) => matching(headers, hue),
        None => Vec::new(),
    };
    let pool: Vec<&Header> = if candidates.is_empty() {
        headers.iter().collect()
    } else {
        candidates
    };
    Some(pool[(roll % pool.len() as u64) as usize])
}

/// Directorio con las cabeceras y su índice.
///
/// En producción llega por `XDG_DATA_DIRS` (`share/loon-launch/headers`, que
/// instala el paquete); en desarrollo se usan los assets del árbol.
pub fn headers_dir() -> Option<PathBuf> {
    if let Ok(dirs) = std::env::var("XDG_DATA_DIRS") {
        for dir in dirs.split(':').filter(|dir| !dir.is_empty()) {
            let candidate = PathBuf::from(dir).join("loon-launch/headers");
            if candidate.join("index.tsv").is_file() {
                return Some(candidate);
            }
        }
    }
    let local = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/headers");
    local.join("index.tsv").is_file().then_some(local)
}

/// Catálogo cargado del disco, junto con el directorio que lo contiene.
pub struct Catalog {
    pub dir: PathBuf,
    pub headers: Vec<Header>,
}

impl Catalog {
    /// Carga el índice del directorio de cabeceras, si existe.
    pub fn load() -> Option<Self> {
        let dir = headers_dir()?;
        let text = std::fs::read_to_string(dir.join("index.tsv")).ok()?;
        let headers = parse_index(&text);
        (!headers.is_empty()).then_some(Catalog { dir, headers })
    }

    /// Ruta completa de una cabecera del catálogo.
    pub fn path(&self, header: &Header) -> PathBuf {
        self.dir.join(&header.file)
    }
}

/// Cabeceras candidatas para esta apertura, en orden de preferencia.
///
/// Se devuelven varias, rotadas desde la elegida, porque una imagen puede no
/// poder decodificarse (falta un loader de formato en el sistema): el llamador
/// prueba la siguiente en vez de quedarse sin cabecera.
pub fn candidates_for_accent(accent: Option<&str>, roll: u64) -> Vec<PathBuf> {
    let Some(catalog) = Catalog::load() else {
        return Vec::new();
    };
    let hue = accent.and_then(hue_of);
    let Some(first) = pick(&catalog.headers, hue, roll) else {
        return Vec::new();
    };
    let start = catalog
        .headers
        .iter()
        .position(|header| header.file == first.file)
        .unwrap_or(0);
    let total = catalog.headers.len();
    (0..total)
        .map(|offset| catalog.path(&catalog.headers[(start + offset) % total]))
        .filter(|path| path.is_file())
        .collect()
}

/// Ruta de una cabecera concreta dentro del directorio dado (para pruebas).
pub fn path_in(dir: &Path, header: &Header) -> PathBuf {
    dir.join(&header.file)
}
