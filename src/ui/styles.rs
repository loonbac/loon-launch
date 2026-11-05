// Hoja de estilos del launcher: paleta + reglas.
//
// Traslado de `style_1.rasi` (gh0stzk/dotfiles). Las reglas no llevan colores
// literales: todo color pasa por un token de `palette.rs`.
use gtk4::prelude::*;

use crate::ui::palette::Palette;
use crate::ui::layout::ELEMENT_PAD;

/// Hoja de estilos completa: tokens de la paleta, fuentes y reglas.
pub fn stylesheet() -> String {
    let palette = Palette::default();
    let mut css = palette.css_defines();
    css.push_str(&palette.font_rule());
    css.push_str(&rules(&palette));
    css
}
/// Reglas del launcher. Separadas de la paleta para que el test pueda
/// comprobar que ninguna declara un color literal.
fn rules(palette: &Palette) -> String {
    let entry_font = palette.entry_font_rule();
    // El padding de la celda viene de layout: una sola fuente de verdad.
    let element_pad = ELEMENT_PAD;
    format!(
        "
/* Ventana: radio 6 px y fondo de la paleta (rofi window). */
window.loon-launch {{
    background-color: @background;
    border-radius: 6px;
}}
.banner-viewport {{
    border-radius: 6px 6px 0 0;
}}

/* Cabecera: chip de glifo + entrada de búsqueda (rofi inputbar). */
.prompt-chip {{
    background-color: @background-alt;
    color: @foreground;
    border-radius: 8px;
    padding: 11px 17px;
}}
entry.search-entry {{
    min-height: 20px;
    padding: 12px;
    border-radius: 8px;
    background-color: @background-alt;
    color: @foreground;
    caret-color: @foreground;
    border: 1px solid transparent;
    outline: none;
    {entry_font}
    transition: border-color 160ms cubic-bezier(0.16, 1, 0.3, 1);
}}
entry.search-entry:focus {{
    border-color: @selected;
}}
entry.search-entry placeholder {{
    color: alpha(@foreground, 0.62);
}}
entry.search-entry selection {{
    background-color: @selected;
    color: @background;
}}

/* Celdas: padding de elemento, radio 4, texto a la izquierda (rofi element). */
row {{
    padding: {element_pad}px;
    border-radius: 4px;
    transition: background-color 160ms cubic-bezier(0.16, 1, 0.3, 1);
}}
row:hover {{
    background-color: @background-alt;
}}
row.selected {{
    background-color: @selected;
}}
label.app-name {{
    color: @foreground;
    font-weight: 500;
}}
row.selected label.app-name {{
    color: @background;
}}
label.section-header {{
    color: alpha(@foreground, 0.72);
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 1.5px;
}}

/* El carrusel de fondos se dibuja sobre el escritorio, sin cromo propio. */
window.loon-launch.wallpaper-mode {{
    background-color: transparent;
    box-shadow: none;
    border: none;
}}
window.loon-launch.wallpaper-mode > box,
window.loon-launch.wallpaper-mode scrolledwindow,
window.loon-launch.wallpaper-mode viewport {{
    background-color: transparent;
    box-shadow: none;
    border: none;
}}
.wallpaper-carousel {{
    background-color: transparent;
}}

/* Sin barras de scroll visibles en ningún modo. */
scrollbar,
scrollbar * {{
    min-width: 0;
    min-height: 0;
    opacity: 0;
    background: transparent;
    border: none;
}}
"
    )
}

pub fn setup_styles(window: &gtk4::ApplicationWindow) {
    let css = gtk4::CssProvider::new();
    css.load_from_data(&stylesheet());
    gtk4::style_context_add_provider_for_display(
        &WidgetExt::display(window),
        &css,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
