// Modelo de una app/acción. Las cotas de la grilla viven en ui/layout.rs.
#[derive(Clone)]
pub struct Item {
    pub name: String,
    pub exec: String, // comando a ejecutar al activar
    pub icon: String,
    /// Si es un wallpaper: el ícono es la miniatura de la imagen/video.
    pub is_wallpaper: bool,
    /// Distingue videos animados de imágenes estáticas dentro del selector.
    pub is_animated_wallpaper: bool,
    /// Cabecera de sección (no ejecutable, no seleccionable).
    pub is_header: bool,
    /// Ruta del media real (video o imagen) para el preview en vivo.
    pub media_path: String,
}

impl Item {
    pub fn app(name: impl Into<String>, exec: impl Into<String>, icon: impl Into<String>) -> Self {
        Item {
            name: name.into(),
            exec: exec.into(),
            icon: icon.into(),
            is_wallpaper: false,
            is_animated_wallpaper: false,
            is_header: false,
            media_path: String::new(),
        }
    }

    pub fn wallpaper(name: impl Into<String>, exec: impl Into<String>, thumb: impl Into<String>) -> Self {
        Item {
            name: name.into(),
            exec: exec.into(),
            icon: thumb.into(),
            is_wallpaper: true,
            is_animated_wallpaper: false,
            is_header: false,
            media_path: String::new(),
        }
    }

    pub fn header(name: impl Into<String>) -> Self {
        Item {
            name: name.into(),
            exec: String::new(),
            icon: String::new(),
            is_wallpaper: false,
            is_animated_wallpaper: false,
            is_header: true,
            media_path: String::new(),
        }
    }

    pub fn with_media(mut self, path: impl Into<String>) -> Self {
        self.media_path = path.into();
        self
    }

    pub fn with_animated_wallpaper(mut self, animated: bool) -> Self {
        self.is_animated_wallpaper = animated;
        self
    }
}

