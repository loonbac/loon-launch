// Métricas y colocación de la grilla del launcher.
//
// Traslado de `style_1.rasi` (gh0stzk/dotfiles): ventana de 550 px, grilla
// column-major de 3 columnas por 5 filas visibles, celdas con padding 8,
// listbox con padding 15 y spacing 3. Las cotas horizontales se derivan del
// ancho de la ventana: cambiar WIN_W o COLS mantiene la grilla dentro del
// ancho útil sin tocar la aritmética de cada celda.

/// Columnas visibles de la grilla de apps.
pub const COLS: usize = 3;
/// Filas de la grilla: una columna se llena antes de pasar a la siguiente.
pub const ROWS: usize = 5;
/// Padding del área de lista (rofi `listbox padding: 15px`).
pub const LIST_PAD: i32 = 15;
/// Separación entre celdas, horizontal y vertical (rofi `listview spacing: 3px`).
pub const LIST_SPACING: i32 = 3;
/// Padding interno de cada celda (rofi `element padding: 8px`). Lo aplica el
/// CSS; la aritmética lo cuenta para que el alto de la ventana sea real.
pub const ELEMENT_PAD: i32 = 8;
/// Alto del contenido de cada celda; con ELEMENT_PAD da el paso de fila.
pub const ROW_H: i32 = 30;
/// Alto de la cabecera: imagen de fondo + entrada de búsqueda.
pub const BANNER_H: i32 = 112;
/// Tamaño del ícono de cada celda (rofi `element-icon size: 24px`).
pub const ICON_SIZE: i32 = 24;

/// Ancho del launcher en modo apps (rofi `window width: 550px`).
pub const WIN_W: i32 = 550;
/// Ancho del bloque de COLS columnas visibles: padding izquierdo de la lista,
/// las columnas y la separación que precede a la columna siguiente. El
/// ScrolledWindow se recorta aquí, así no asoma una fracción de la siguiente.
pub const VISIBLE_W: i32 = LIST_PAD + COLS as i32 * cell_width() + COLS as i32 * LIST_SPACING;
/// Área que ocupa la lista de apps: ROWS filas más el padding de la lista.
pub const WELL_H: i32 = ROWS as i32 * (ROW_H + 2 * ELEMENT_PAD) + 2 * LIST_PAD;
/// Alto del launcher: cabecera + área de lista.
pub const WIN_H: i32 = BANNER_H + WELL_H;

/// Modo fondos: carrusel ancho inspirado en el image-picker de Omarchy.
/// La preview central conserva sus proporciones originales y las laterales
/// aparecen como cortes estrechos superpuestos.
pub const WP_WIN_W: i32 = 1400;
pub const WP_WIN_H: i32 = 590;

/// Ancho de celda que reparte el ancho útil de la lista entre COLS columnas.
/// La división es entera: la grilla puede quedar hasta COLS px más angosta
/// que la ventana, nunca más ancha.
pub const fn cell_width() -> i32 {
    (WIN_W - 2 * LIST_PAD - LIST_SPACING * (COLS as i32 - 1)) / COLS as i32
}

/// Posición (columna, fila) de un item, en orden column-major: los primeros
/// ROWS items llenan la primera columna.
pub fn cell_position(index: usize) -> (i32, i32) {
    ((index / ROWS) as i32, (index % ROWS) as i32)
}

/// Filas que ocupa una sección de `items` celdas dentro de una columna.
pub fn section_rows(items: usize) -> i32 {
    items.div_ceil(ROWS) as i32
}
