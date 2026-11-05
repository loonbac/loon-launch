// Cabecera del launcher: imagen de fondo + chip de glifo + entrada de búsqueda.
//
// Traslado de `style_1.rasi` (gh0stzk/dotfiles): el `inputbar` de rofi lleva la
// imagen a todo el ancho con padding lateral de 100 px, un chip con glifo y la
// entrada de búsqueda `expand`.
use gtk4::prelude::*;
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use crate::ui::layout::{BANNER_H, WIN_W};

/// Imagen de cabecera vigente. Vive aquí porque el `draw_func` la lee en cada
/// repintado y `show_header` la reemplaza entre aperturas del launcher.
type CurrentImage = Rc<RefCell<Option<gtk4::gdk_pixbuf::Pixbuf>>>;
/// Glifo del chip: hamburguesa (Nerd Font).
const PROMPT_GLYPH: char = '\u{f0c9}';
const PLACEHOLDER: &str = "Buscar app…  ('>' = poder)";
/// Padding lateral de la fila de búsqueda dentro de la cabecera.
const BAR_PAD_X: i32 = 100;

pub struct BannerRefs {
    pub root: gtk4::Overlay,
    pub viewport: gtk4::DrawingArea,
    pub entry: gtk4::Entry,
    image: CurrentImage,
}

pub fn build_banner() -> BannerRefs {
    let banner = gtk4::Overlay::new();
    banner.set_size_request(WIN_W, BANNER_H);
    banner.set_hexpand(true);
    banner.add_css_class("banner-viewport");

    let banner_viewport = gtk4::DrawingArea::new();
    banner_viewport.set_content_width(WIN_W);
    banner_viewport.set_content_height(BANNER_H);
    banner_viewport.set_size_request(WIN_W, BANNER_H);

    // La cabecera se elige al abrir el launcher (ver `ui::headers`); hasta que
    // haya una imagen, el fondo es un degradado integrado.
    let image: CurrentImage = Rc::new(RefCell::new(None));
    let draw_image = image.clone();
    banner_viewport.set_draw_func(move |_, cr, width, height| {
        match draw_image.borrow().as_ref() {
            Some(pixbuf) => {
                // Escalada tipo `cover`: cubre el ancho de la ventana y recorta
                // el alto sobrante por igual arriba y abajo, centrada.
                let image_w = f64::from(pixbuf.width());
                let image_h = f64::from(pixbuf.height());
                let scale = (f64::from(width) / image_w).max(f64::from(height) / image_h);
                let _ = cr.save();
                cr.translate(
                    (f64::from(width) - image_w * scale) / 2.0,
                    (f64::from(height) - image_h * scale) / 2.0,
                );
                cr.scale(scale, scale);
                let _ = cr.set_source_pixbuf(pixbuf, 0.0, 0.0);
                let _ = cr.paint();
                let _ = cr.restore();
            }
            None => {
                let gradient =
                    gtk4::cairo::LinearGradient::new(0.0, 0.0, f64::from(width), f64::from(height));
                gradient.add_color_stop_rgb(0.0, 0.12, 0.03, 0.24);
                gradient.add_color_stop_rgb(0.5, 0.42, 0.10, 0.58);
                gradient.add_color_stop_rgb(1.0, 0.08, 0.02, 0.18);
                let _ = cr.set_source(&gradient);
                let _ = cr.paint();
            }
        }
    });
    banner.set_child(Some(&banner_viewport));

    let entry = build_search_bar();
    let bar = entry.1;
    banner.add_overlay(&bar);
    banner.set_measure_overlay(&bar, false);

    BannerRefs {
        root: banner,
        viewport: banner_viewport,
        entry: entry.0,
        image,
    }
}

/// Fila de búsqueda: chip de glifo + entrada, con el padding lateral de rofi.
fn build_search_bar() -> (gtk4::Entry, gtk4::Box) {
    let bar = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
    bar.set_valign(gtk4::Align::Center);
    bar.set_halign(gtk4::Align::Fill);
    bar.set_margin_start(BAR_PAD_X);
    bar.set_margin_end(BAR_PAD_X);

    let chip = gtk4::Label::new(Some(&PROMPT_GLYPH.to_string()));
    chip.add_css_class("prompt-chip");
    chip.set_valign(gtk4::Align::Center);
    bar.append(&chip);

    let entry = gtk4::Entry::new();
    entry.set_placeholder_text(Some(PLACEHOLDER));
    entry.add_css_class("search-entry");
    entry.set_hexpand(true);
    bar.append(&entry);

    (entry, bar)
}

impl BannerRefs {
    /// Muestra `path` como cabecera. Devuelve si la imagen se pudo cargar; si
    /// no, se conserva la anterior (o el degradado).
    pub fn show_header(&self, path: &Path) -> bool {
        // Se decodifica al doble del ancho de la ventana: suficiente para
        // HiDPI y evita cargar en memoria un original de 3600 px de ancho.
        let Ok(pixbuf) =
            gtk4::gdk_pixbuf::Pixbuf::from_file_at_scale(path, WIN_W * 2, -1, true)
        else {
            return false;
        };
        *self.image.borrow_mut() = Some(pixbuf);
        self.viewport.queue_draw();
        true
    }

    pub fn apply_mode(&self, wallpaper: bool) {
        // El banner de apps no se toca. En fondos se oculta: no hay búsqueda.
        self.root.set_visible(!wallpaper);
        self.root.set_size_request(WIN_W, BANNER_H);
        self.viewport.set_content_width(WIN_W);
        self.viewport.set_content_height(BANNER_H);
        self.viewport.set_size_request(WIN_W, BANNER_H);
        self.entry.set_placeholder_text(Some(PLACEHOLDER));
    }
}
