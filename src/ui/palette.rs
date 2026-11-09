// Paleta del launcher: los tokens de `shared.rasi` (gh0stzk/dotfiles) más el
// acento vivo del sistema.
//
// Una sola fuente de verdad para color y tipografía. Las reglas de `styles.rs`
// referencian estos tokens como `@nombre`; ningún color literal vive fuera de
// este archivo, así que cambiar el tema es editar esta paleta.
//
// El acento no se hereda del `.rasi`: el sistema lo deriva del wallpaper activo
// (`accent-wallpaper` del repo de NixOS) y lo publica en
// `~/.config/mpvpaper/accent.txt`. Cuando ese archivo existe y es válido, la
// selección del launcher lo sigue; si no, queda el acento por defecto.

use std::path::{Path, PathBuf};

/// Archivo donde `accent-wallpaper` publica el acento del wallpaper activo.
pub const ACCENT_FILE: &str = ".config/mpvpaper/accent.txt";

/// Acento fijado por el launcher al aplicar un fondo estático: la capa
/// externa (`None`) dice si hay fijado; la interna, el valor fijado.
/// Regla del dueño: solo los animados mueven el acento. Como el demonio
/// reescribe el archivo con cualquier fondo, el launcher fija el vigente
/// al aplicar un estático e ignora el archivo hasta que un animado lo
/// libere. Lo que se cambia fuera del launcher con un fijo activo queda
/// fuera de esta regla (limitación conocida).

/// Nombres de los tokens de color expuestos a la hoja de estilos.
pub const COLOR_TOKENS: [&str; 6] = [
    "background",
    "background-alt",
    "foreground",
    "selected",
    "active",
    "urgent",
];

/// Tokens visuales del launcher.
pub struct Palette {
    pub font_family: &'static str,
    /// Tamaño del texto de las celdas, en px.
    pub font_size: i32,
    /// Tamaño del texto de la entrada de búsqueda, en px.
    pub entry_size: i32,
    pub background: String,
    pub background_alt: String,
    pub foreground: String,
    pub selected: String,
    pub active: String,
    pub urgent: String,
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            font_family: "JetBrainsMono Nerd Font",
            font_size: 12,
            entry_size: 13,
            background: "#1A1B26".to_string(),
            background_alt: "#1A1B26E0".to_string(),
            foreground: "#c0caf5".to_string(),
            selected: "#7aa2f7".to_string(),
            active: "#9ece6a".to_string(),
            urgent: "#f7768e".to_string(),
        }
    }
}

impl Palette {
    /// Paleta base con el acento indicado. `None` deja el acento por defecto.
    pub fn with_accent(accent: Option<String>) -> Self {
        let mut palette = Self::default();
        if let Some(accent) = accent {
            palette.selected = accent;
        }
        palette
    }

    /// Paleta del launcher en ejecución: acento efectivo (fijado o sistema).
    pub fn current() -> Self {
        Self::with_accent(effective_accent())
    }

    /// Bloque `@define-color` que consumen las reglas de estilo.
    pub fn css_defines(&self) -> String {
        let mut css = String::from("/* Paleta: tokens de shared.rasi + acento del sistema. */\n");
        for (token, value) in [
            ("background", &self.background),
            ("background-alt", &self.background_alt),
            ("foreground", &self.foreground),
            ("selected", &self.selected),
            ("active", &self.active),
            ("urgent", &self.urgent),
        ] {
            css.push_str(&format!("@define-color {token} {value};\n"));
        }
        css
    }

    /// Fuente base. GTK no tiene tokens de tipografía, así que la regla se
    /// deriva de esta misma paleta en vez de repetirse en cada selector.
    pub fn font_rule(&self) -> String {
        format!(
            "* {{\n    font-family: \"{}\";\n    font-size: {}px;\n}}\n",
            self.font_family, self.font_size
        )
    }

    /// Fuente de la entrada de búsqueda, algo mayor que la de las celdas.
    pub fn entry_font_rule(&self) -> String {
        format!("font-size: {}px;", self.entry_size)
    }
}

/// Normaliza un acento a `#RRGGBB`, o `None` si no es un hex de seis dígitos.
///
/// El archivo lo escribe otro programa: llega con salto de línea y puede estar
/// corrupto, así que se valida antes de inyectarlo en la hoja de estilos.
pub fn normalize_accent(raw: &str) -> Option<String> {
    let hex = raw.trim().strip_prefix('#')?;
    if hex.len() != 6 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    Some(format!("#{}", hex.to_ascii_uppercase()))
}

/// Acento publicado en `path`, si el archivo existe y es válido.
pub fn accent_from_file(path: &Path) -> Option<String> {
    normalize_accent(&std::fs::read_to_string(path).ok()?)
}

/// Acento del sistema derivado del wallpaper activo.
pub fn system_accent() -> Option<String> {
    let home = std::env::var_os("HOME")?;
    accent_from_file(&PathBuf::from(home).join(ACCENT_FILE))
}

/// Núcleo puro: el fijado manda; sin fijar, manda el archivo.
pub(crate) fn resolve_accent(
    pinned: Option<Option<String>>,
    file: Option<String>,
) -> Option<String> {
    pinned.unwrap_or(file)
}

static PINNED_ACCENT: std::sync::Mutex<Option<Option<String>>> = std::sync::Mutex::new(None);

/// Fija el acento vigente en este instante. Llamar ANTES de aplicar el
/// fondo estático: el demonio aún no reescribió el archivo y el vigente
/// es el anterior (del último animado o el defecto).
pub fn pin_accent() {
    let current = effective_accent();
    *PINNED_ACCENT.lock().unwrap() = Some(current);
}

/// Libera el fijado: el acento vuelve a seguir al archivo. Al aplicar un
/// fondo animado desde el launcher.
pub fn unpin_accent() {
    *PINNED_ACCENT.lock().unwrap() = None;
}

/// Acento efectivo: el fijado si lo hay; si no, el publicado por el sistema.
pub fn effective_accent() -> Option<String> {
    let pinned = PINNED_ACCENT.lock().unwrap().clone();
    resolve_accent(pinned, system_accent())
}
