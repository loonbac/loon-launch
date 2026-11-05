// Tests de la lógica pura del launcher (filtrado, navegación, edición).
use crate::apps::{data_dirs, power_actions};
use crate::filter::{
    apply_backspace, apply_char, filter_items, move_sel_grid, move_sel_rowwise, move_selection,
    move_selection_wrap, normalize_selection,
};
use crate::models::Item;
use crate::ui::layout::ROWS;
use crate::ui::parse_logical_size;

fn app(name: &str) -> Item {
    Item::app(name, "true", "x")
}

fn wallpaper(name: &str, animated: bool) -> Item {
    Item::wallpaper(name, "true", "/tmp/thumb.jpg").with_animated_wallpaper(animated)
}

fn power() -> Vec<Item> {
    power_actions()
}

#[test]
fn filter_matches_by_name_case_insensitive() {
    let apps = vec![app("Firefox"), app("Ghostty"), app("VS Code")];
    let got = filter_items(&apps, &power(), &[], "fire");
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].name, "Firefox");
}

#[test]
fn filter_empty_query_returns_all() {
    let apps = vec![app("A"), app("B")];
    let got = filter_items(&apps, &power(), &[], "");
    assert_eq!(got.len(), 2);
}

#[test]
fn filter_power_mode_prefix() {
    let apps = vec![app("Firefox")];
    let got = filter_items(&apps, &power(), &[], ">apag");
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].name, "Apagar");
}

#[test]
fn filter_power_empty_shows_all_power() {
    let got = filter_items(&[], &power(), &[], ">");
    assert_eq!(got.len(), power().len());
}

#[test]
fn filter_wallpaper_mode_prefix() {
    let wps = vec![wallpaper("▶ reze.mp4", true), wallpaper("🖼 Asa.png", false)];
    let got = filter_items(&[], &[], &wps, "#reze");
    assert_eq!(got.len(), 1);
    assert!(got[0].name.contains("reze"));
}

#[test]
fn wallpaper_tabs_separate_animated_and_static_lists() {
    let wps = vec![wallpaper("▶ a.mp4", true), wallpaper("🖼 b.png", false)];
    let animated = filter_items(&[], &[], &wps, "#");
    let static_images = filter_items(&[], &[], &wps, "#!");
    assert_eq!(animated.len(), 1);
    assert_eq!(animated[0].name, "▶ a.mp4");
    assert_eq!(static_images.len(), 1);
    assert_eq!(static_images[0].name, "🖼 b.png");
}

#[test]
fn move_sel_right_steps_one() {
    assert_eq!(move_selection(0, 1, 10), 1);
    assert_eq!(move_selection(9, 1, 10), 9); // clampa al final
}

#[test]
fn move_sel_left_clamps_at_zero() {
    assert_eq!(move_selection(0, -1, 10), 0);
    assert_eq!(move_selection(1, -1, 10), 0);
}

#[test]
fn move_sel_down_steps_rows() {
    // En la lista de ROWS filas, Derecha salta ROWS (siguiente columna).
    assert_eq!(move_sel_rowwise(0, ROWS as i32, 20), ROWS as i32);
    assert_eq!(move_sel_rowwise(18, ROWS as i32, 20), 19); // clampa
}

#[test]
fn move_sel_empty_returns_neg1() {
    assert_eq!(move_selection(0, 1, 0), -1);
}

#[test]
fn move_sel_from_invalid_anchors_to_zero() {
    // sel inválido (-1) se ancla a 0 (primera celda).
    assert_eq!(move_selection(-1, 1, 10), 0);
    assert_eq!(move_selection(-1, -1, 10), 0);
}

#[test]
fn normalize_sel_resets_out_of_range() {
    assert_eq!(normalize_selection(5, 3), 0);
    assert_eq!(normalize_selection(-2, 3), 0);
    assert_eq!(normalize_selection(2, 3), 2);
    assert_eq!(normalize_selection(0, 0), -1);
}

#[test]
fn char_and_backspace_edit_text() {
    assert_eq!(apply_char("gh", 'o'), "gho");
    assert_eq!(apply_backspace("gho"), "gh");
    assert_eq!(apply_backspace(""), "");
}

#[test]
fn dedup_prefers_waydroid_app_wrapper() {
    let mut apps = vec![
        Item::app(
            "Android Notes",
            "waydroid app launch com.example.notes",
            "x",
        ),
        Item::app("Android Notes", "waydroid-app com.example.notes", "x"),
    ];
    apps.sort_by(|a, b| {
        a.name
            .to_lowercase()
            .cmp(&b.name.to_lowercase())
            .then_with(|| b.exec.contains("waydroid-app").cmp(&a.exec.contains("waydroid-app")))
    });
    apps.dedup_by(|a, b| a.name.to_lowercase() == b.name.to_lowercase());
    assert_eq!(apps.len(), 1);
    assert_eq!(apps[0].exec, "waydroid-app com.example.notes");
}

#[test]
fn wallpaper_carousel_wraps_in_both_directions() {
    assert_eq!(move_selection_wrap(0, -1, 5), 4);
    assert_eq!(move_selection_wrap(4, 1, 5), 0);
    assert_eq!(move_selection_wrap(2, 1, 5), 3);
    assert_eq!(move_selection_wrap(0, 1, 0), -1);
}

#[test]
fn apps_grid_right_stays_on_same_row() {
    // Column-major 4-row grid: idx 0=(0,0), 1=(1,0), 4=(0,1)
    let pos: Vec<(i32, i32)> = (0..8).map(|i| ((i % 4) as i32, (i / 4) as i32)).collect();
    assert_eq!(move_sel_grid(0, 1, 0, &pos), 1);
    assert_eq!(move_sel_grid(0, 0, 1, &pos), 4);
    assert_eq!(move_sel_grid(3, 1, 0, &pos), 3);
    assert_eq!(move_sel_grid(0, -1, 0, &pos), 0);
}

#[test]
fn app_discovery_includes_dynamic_flatpak_exports() {
    let dirs = data_dirs();
    assert!(dirs
        .iter()
        .any(|dir| dir == std::path::Path::new("/var/lib/flatpak/exports/share")));

    if let Some(home) = std::env::var_os("HOME") {
        let user_exports =
            std::path::PathBuf::from(home).join(".local/share/flatpak/exports/share");
        assert!(dirs.contains(&user_exports));
    }
}

#[test]
fn grid_places_items_column_major_in_five_rows() {
    use crate::ui::layout::{cell_position, ROWS};

    // style_1.rasi: columns: 3, lines: 5. La primera columna se llena primero.
    assert_eq!(ROWS, 5);
    assert_eq!(cell_position(0), (0, 0));
    assert_eq!(cell_position(4), (0, 4));
    assert_eq!(cell_position(5), (1, 0));
    assert_eq!(cell_position(14), (2, 4));
    assert_eq!(cell_position(15), (3, 0));
}

#[test]
fn grid_fits_three_columns_within_the_window() {
    use crate::ui::layout::{cell_width, COLS, LIST_PAD, LIST_SPACING, VISIBLE_W, WIN_W};

    // style_1.rasi: width: 550px, listview spacing 3px, tres columnas.
    assert_eq!(WIN_W, 550);
    assert_eq!(COLS, 3);

    // Las tres columnas entran completas en el ancho visible.
    let columns_w = LIST_PAD + COLS as i32 * cell_width() + (COLS as i32 - 1) * LIST_SPACING;
    assert!(columns_w <= VISIBLE_W, "las columnas ({columns_w}) no caben");

    // El recorte cae exactamente donde empieza la columna siguiente: ninguna
    // fracción de la cuarta columna asoma en el borde.
    assert_eq!(
        VISIBLE_W,
        LIST_PAD + COLS as i32 * cell_width() + COLS as i32 * LIST_SPACING
    );
    assert!(VISIBLE_W < WIN_W, "falta el padding derecho de la lista");
}

#[test]
fn layout_height_is_banner_plus_five_rows() {
    use crate::ui::layout::{BANNER_H, ELEMENT_PAD, LIST_PAD, ROWS, ROW_H, WIN_H};

    // El paso de fila es el contenido más el padding del elemento: es el alto
    // que GTK termina asignando a cada celda.
    let row_pitch = ROW_H + 2 * ELEMENT_PAD;
    assert_eq!(row_pitch, 46);
    assert_eq!(WIN_H, BANNER_H + ROWS as i32 * row_pitch + 2 * LIST_PAD);
}

#[test]
fn palette_emits_every_color_token() {
    use crate::ui::palette::{Palette, COLOR_TOKENS};

    let defines = Palette::default().css_defines();
    for token in COLOR_TOKENS {
        assert!(
            defines.contains(&format!("@define-color {token} ")),
            "la paleta no define {token}:\n{defines}"
        );
    }
}

#[test]
fn stylesheet_uses_tokens_and_no_literal_colors_in_rules() {
    // S5: las reglas no llevan colores literales; sólo la paleta los declara.
    let css = crate::ui::styles::stylesheet();
    let rules: String = css
        .lines()
        .filter(|line| !line.trim_start().starts_with("@define-color"))
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        !rules.contains("rgba(") && !rules.contains("#"),
        "una regla de estilo lleva un color literal en vez de un token:\n{rules}"
    );
    assert!(rules.contains("@background") || rules.contains("@background-alt"));
}

#[test]
fn parses_niri_logical_output_size() {
    let output = "Output \"monitor\" (HDMI-A-1)\n  Logical size: 1080x1920\n  Scale: 1\n";
    assert_eq!(parse_logical_size(output), Some((1080, 1920)));
    assert_eq!(parse_logical_size("no output"), None);
}

#[test]
fn palette_uses_the_system_accent_when_available() {
    use crate::ui::palette::{normalize_accent, Palette};

    // `accent-wallpaper` publica el acento del wallpaper como hex de seis digitos.
    assert_eq!(normalize_accent("#217168\n"), Some("#217168".to_string()));
    assert_eq!(normalize_accent("  #7aa2f7  "), Some("#7AA2F7".to_string()));

    let palette = Palette::with_accent(Some("#217168".to_string()));
    assert_eq!(palette.selected, "#217168");
    assert!(palette
        .css_defines()
        .contains("@define-color selected #217168;"));
}

#[test]
fn palette_keeps_its_default_accent_without_a_valid_token() {
    use crate::ui::palette::{normalize_accent, Palette};

    // Archivo ausente, truncado o basura: el launcher no puede quedarse sin color.
    assert_eq!(normalize_accent(""), None);
    assert_eq!(normalize_accent("#21716"), None);
    assert_eq!(normalize_accent("#ZZZZZZ"), None);
    assert_eq!(normalize_accent("teal"), None);

    let palette = Palette::with_accent(None);
    assert_eq!(palette.selected, Palette::default().selected);
}
