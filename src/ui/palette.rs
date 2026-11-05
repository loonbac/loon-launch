// Paleta del launcher: los tokens de `shared.rasi` (gh0stzk/dotfiles).
//
// Una sola fuente de verdad para color y tipografía. Las reglas de `styles.rs`
// referencian estos tokens como `@nombre`; ningún color literal vive fuera de
// este archivo, así que cambiar el tema es editar esta paleta.

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
    pub background: &'static str,
    pub background_alt: &'static str,
    pub foreground: &'static str,
    pub selected: &'static str,
    pub active: &'static str,
    pub urgent: &'static str,
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            font_family: "JetBrainsMono Nerd Font",
            font_size: 12,
            entry_size: 13,
            background: "#1a1b26",
            background_alt: "rgba(26, 27, 38, 0.86)",
            foreground: "#c0caf5",
            selected: "#ee3d96",
            active: "#9ece6a",
            urgent: "#f7768e",
        }
    }
}

impl Palette {
    /// Bloque `@define-color` que consumen las reglas de estilo.
    pub fn css_defines(&self) -> String {
        let mut css = String::from("/* Paleta: tokens de shared.rasi. */\n");
        for (token, value) in [
            ("background", self.background),
            ("background-alt", self.background_alt),
            ("foreground", self.foreground),
            ("selected", self.selected),
            ("active", self.active),
            ("urgent", self.urgent),
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
