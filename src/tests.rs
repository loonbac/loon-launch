// Tests de la lógica pura del launcher (filtrado, navegación, edición).
use crate::apps::{data_dirs, power_actions};
use crate::filter::{
    apply_backspace, apply_char, filter_items, move_sel_grid, move_sel_rowwise, move_selection,
    move_selection_wrap, normalize_selection, hold_is_armed, hold_progress, requires_hold,
    SearchMode, HOLD_MS,
};
use crate::models::Item;
use crate::power::{
    can_hibernate, can_hibernate_with_swap_size, has_adequate_swap, has_disk_swap,
};
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
    let got = filter_items(&apps, &power(), &[], "fire", SearchMode::Apps);
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].name, "Firefox");
}

#[test]
fn filter_empty_query_returns_all() {
    let apps = vec![app("A"), app("B")];
    let got = filter_items(&apps, &power(), &[], "", SearchMode::Apps);
    assert_eq!(got.len(), 2);
}

#[test]
fn filter_power_mode_prefix() {
    let apps = vec![app("Firefox")];
    let got = filter_items(&apps, &power(), &[], ">apag", SearchMode::Apps);
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].name, "Apagar");
}

#[test]
fn filter_power_empty_shows_all_power() {
    let got = filter_items(&[], &power(), &[], ">", SearchMode::Apps);
    assert_eq!(got.len(), power().len());
}

#[test]
fn wallpaper_mode_filters_by_name_without_a_prefix() {
    let wps = vec![wallpaper("▶ reze.mp4", true), wallpaper("🖼 Asa.png", false)];
    let got = filter_items(
        &[],
        &[],
        &wps,
        "reze",
        SearchMode::Wallpapers { static_only: false },
    );
    assert_eq!(got.len(), 1);
    assert!(got[0].name.contains("reze"));
}

#[test]
fn wallpaper_tabs_separate_animated_and_static_lists() {
    let wps = vec![wallpaper("▶ a.mp4", true), wallpaper("🖼 b.png", false)];
    let animated = filter_items(
        &[],
        &[],
        &wps,
        "",
        SearchMode::Wallpapers { static_only: false },
    );
    let static_images = filter_items(
        &[],
        &[],
        &wps,
        "",
        SearchMode::Wallpapers { static_only: true },
    );
    assert_eq!(animated.len(), 1);
    assert_eq!(animated[0].name, "▶ a.mp4");
    assert_eq!(static_images.len(), 1);
    assert_eq!(static_images[0].name, "🖼 b.png");
}

#[test]
fn typing_hash_does_not_open_the_wallpaper_picker() {
    // `#` es un carácter normal: filtra apps por nombre y, como ninguna app se
    // llama `#`, el resultado queda vacío. Nunca devuelve fondos.
    let apps = vec![app("Firefox"), app("Ghostty")];
    let wps = vec![wallpaper("▶ reze.mp4", true), wallpaper("🖼 Asa.png", false)];
    for q in ["#", "#!", "#reze"] {
        let got = filter_items(&apps, &power(), &wps, q, SearchMode::Apps);
        assert!(
            got.iter().all(|it| !it.is_wallpaper),
            "la query {q:?} devolvió fondos desde el modo apps"
        );
        assert!(got.is_empty(), "la query {q:?} debía quedar vacía");
    }
}

#[test]
fn power_prefix_still_opens_power_actions() {
    let got = filter_items(&[], &power(), &[], ">", SearchMode::Apps);
    assert_eq!(got.len(), power().len());
    assert!(got.iter().all(|it| !it.is_wallpaper));
}

#[test]
fn power_menu_does_not_offer_hibernate() {
    // Esta máquina no puede hibernar: no hay `resume=` en el cmdline y el único
    // swap es zram (volátil). Ofrecerlo sería prometer algo que no ocurre.
    let names: Vec<String> = power().iter().map(|it| it.name.clone()).collect();
    assert!(
        !names.iter().any(|n| n.contains("Hibernar")),
        "el menú de poder no debe ofrecer Hibernar: {names:?}"
    );
}

#[test]
fn power_menu_keeps_the_actions_that_make_sense_on_a_desktop() {
    let names: Vec<String> = power().iter().map(|it| it.name.clone()).collect();
    for expected in ["Apagar", "Reiniciar", "Suspender", "Bloquear"] {
        assert!(
            names.iter().any(|n| n == expected),
            "falta la acción {expected}: {names:?}"
        );
    }
    assert!(
        names.iter().any(|n| n.contains("Cambiar fondo")),
        "falta cambiar fondo de pantalla: {names:?}"
    );
}

#[test]
fn only_disruptive_power_actions_require_a_hold() {
    // Mantener Enter sólo para lo que interrumpe la sesión o la apaga.
    assert!(requires_hold("systemctl poweroff"));
    assert!(requires_hold("systemctl reboot"));
    assert!(requires_hold("systemctl suspend"));
    assert!(requires_hold("loginctl lock-session"));
    // El selector de fondos y las apps normales abren con un Enter suelto.
    assert!(!requires_hold("wallpaper-mode"));
    assert!(!requires_hold("firefox"));
    assert!(!requires_hold(""));
}

#[test]
fn hold_progress_grows_and_completes_at_the_threshold() {
    assert_eq!(hold_progress(0), 0.0);
    assert!((hold_progress(HOLD_MS / 2) - 0.5).abs() < 0.01);
    assert_eq!(hold_progress(HOLD_MS), 1.0);
    // Nunca pasa de 1: la barra no puede desbordar.
    assert_eq!(hold_progress(HOLD_MS * 3), 1.0);
}

#[test]
fn hold_is_not_armed_below_the_threshold() {
    assert!(!hold_is_armed(HOLD_MS - 1));
    assert!(hold_is_armed(HOLD_MS));
    assert!(hold_is_armed(HOLD_MS + 500));
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

#[test]
fn overlay_is_used_only_on_wayland_with_layer_shell_support() {
    use crate::ui::overlay::{use_overlay, NAMESPACE};

    // Solo con Wayland Y soporte del compositor; en el resto de casos la app
    // tiene que seguir funcionando con una ventana normal.
    assert!(use_overlay(true, true));
    assert!(!use_overlay(true, false));
    assert!(!use_overlay(false, true));
    assert!(!use_overlay(false, false));

    // El namespace es el identificador visible para el compositor.
    assert_eq!(NAMESPACE, "loon-launch");
    assert!(!NAMESPACE.contains(char::is_whitespace));
}

#[test]
fn overlay_applies_on_demand_keyboard_on_the_overlay_layer() {
    use gtk4_layer_shell::{KeyboardMode, Layer};
    use crate::ui::overlay::{CONFIG, NAMESPACE};

    // R3-001/R4-001 eran correctos: con `Exclusive` el compositor no retira el
    // foco, `EventControllerFocus` nunca emite `leave` y la capa queda encima de
    // todo. Medido en niri: `leave` no se emite y `niri msg layers` sigue
    // contando la capa. Con `OnDemand` sí se emite y la capa desaparece, y la
    // entrada sigue recibiendo teclado (verificado con wtype).
    assert_eq!(CONFIG.keyboard, KeyboardMode::OnDemand);
    assert_eq!(CONFIG.layer, Layer::Overlay);
    assert_eq!(CONFIG.exclusive_zone, 0);
    assert_eq!(CONFIG.namespace, NAMESPACE);
}

#[test]
fn headers_parse_skips_malformed_lines() {
    use crate::ui::headers::parse_index;

    // El índice lo genera un script: una línea rota no debe tumbar el catálogo.
    let text = "# cabeceras\nok.webp\t120.0\t0.4\t0.5\nsin-tono.webp\tx\t0.4\t0.5\n\n\t90.0\t0.2\t0.3\nfuera.webp\t420.0\t0.2\t0.3\nvacio.webp\t90.0\n";
    let headers = parse_index(text);
    assert_eq!(headers.len(), 1);
    assert_eq!(headers[0].file, "ok.webp");
    assert_eq!(headers[0].hue, 120.0);
}

#[test]
fn accent_hue_is_computed_circularly() {
    use crate::ui::headers::hue_of;

    // El acento del sistema llega como hex de seis dígitos.
    let teal = hue_of("#217168").expect("teal tiene tono");
    assert!((teal - 173.0).abs() < 3.0, "teal ~173°, fue {teal}");
    let red = hue_of("#FF0000").expect("rojo tiene tono");
    assert!(red.abs() < 0.01 || (red - 360.0).abs() < 0.01);
    // Sin tono definido: gris o hex inválido.
    assert_eq!(hue_of("#808080"), None);
    assert_eq!(hue_of("217168"), None);
}

#[test]
fn headers_are_picked_only_from_the_accent_window() {
    use crate::ui::headers::{matching, pick, Header, HUE_WINDOW};

    let headers = vec![
        Header { file: "teal.webp".into(), hue: 175.0 },
        Header { file: "azul.webp".into(), hue: 230.0 },
        Header { file: "rojo.webp".into(), hue: 355.0 },
        Header { file: "rojo-cerca.webp".into(), hue: 5.0 },
    ];

    // La ventana es de ±HUE_WINDOW grados y cruza el 0/360 sin romperse.
    let near_teal = matching(&headers, 173.0);
    assert_eq!(near_teal.len(), 1);
    assert_eq!(near_teal[0].file, "teal.webp");

    let near_red = matching(&headers, 0.0);
    assert_eq!(near_red.len(), 2, "rojo 355 y 5 deben entrar");

    // La elección es reproducible con el mismo roll y cambia con otro.
    let first = pick(&headers, Some(0.0), 0).map(|h| h.file.clone());
    let repeat = pick(&headers, Some(0.0), 2).map(|h| h.file.clone());
    assert_eq!(first, repeat);
    let other = pick(&headers, Some(0.0), 1).map(|h| h.file.clone());
    assert_ne!(first, other);

    // Sin acento, o sin coincidencias, se usa el catálogo completo.
    assert!(pick(&headers, None, 0).is_some());
    assert!(pick(&headers, Some(120.0), 0).is_some());
}

#[test]
fn header_catalog_loads_and_matches_the_system_accent() {
    use crate::ui::headers::{matching, hue_of, Catalog};

    let catalog = Catalog::load().expect("el índice de cabeceras debe existir");
    assert!(catalog.headers.len() > 100, "catálogo vacío");

    // Con el acento actual del escritorio debe haber candidatas.
    let accent = crate::ui::palette::system_accent();
    if let Some(hue) = accent.as_deref().and_then(hue_of) {
        let candidates = matching(&catalog.headers, hue);
        assert!(
            !candidates.is_empty(),
            "ninguna cabecera combina con el acento {accent:?} (tono {hue})"
        );
        // Y la ruta resuelta tiene que existir en disco.
        let path = catalog.path(candidates[0]);
        assert!(path.is_file(), "no existe {}", path.display());
        // El llamador recibe varias: si una no decodifica, prueba la siguiente.
        let ordered = crate::ui::headers::candidates_for_accent(accent.as_deref(), 0);
        assert!(ordered.len() > 1, "sin alternativas si la primera falla");
    }
}

#[test]
fn hibernate_is_offered_only_with_resume_and_disk_swap() {
    // Las dos condiciones juntas: sin una de ellas no hay imagen que restaurar.
    assert!(can_hibernate(Some("resume=/dev/nvme0n1p3"), "/dev/nvme0n1p3 partition"));
    assert!(!can_hibernate(None, "/dev/nvme0n1p3 partition")); // sin resume=
    assert!(!can_hibernate(Some("resume=/dev/sda2"), "/dev/zram0 partition")); // sólo zram
    assert!(!can_hibernate(None, "/dev/zram0 partition"));
}

#[test]
fn hibernate_is_offered_never_when_proc_is_unreadable() {
    // Si no se puede leer, se asume que no puede: nunca ofrecer algo que podría
    // dejar la sesión a medias.
    assert!(!can_hibernate(None, ""));
}

#[test]
fn zram_does_not_count_as_a_hibernate_backing_store() {
    let swaps = "/dev/zram0 partition 16393724 7722392 100";
    assert!(!has_disk_swap(swaps));
    let mixed = "/dev/zram0 partition 16393724 7722392 100\n/dev/nvme0n1p3 partition 16777216 0 -2";
    assert!(has_disk_swap(mixed));
    assert!(!has_disk_swap(""));
}

#[test]
fn hibernate_requires_a_hold_like_the_other_disruptive_actions() {
    // Cuando la entrada aparece, es tan disruptiva como apagar o suspender.
    assert!(requires_hold("systemctl hibernate"));
}

#[test]
fn hibernate_needs_a_swap_at_least_as_large_as_ram() {
    // Una imagen de hibernación ocupa como la RAM en uso: un swap menor puede
    // quedarse corto y abortar la operación a mitad.
    let mem = 16_000_000; // KiB
    assert!(has_adequate_swap(mem, "/dev/nvme0n1p3 partition 16777216 0 -2"));
    assert!(!has_adequate_swap(mem, "/dev/nvme0n1p3 partition 4000000 0 -2"));
    // Justo igual basta.
    assert!(has_adequate_swap(mem, "/dev/sda2 partition 16000000 0 -2"));
}

#[test]
fn zram_never_counts_even_when_it_is_large() {
    // zram vive en RAM: aunque sea enorme, no sobrevive al apagado.
    assert!(!has_adequate_swap(
        1_000,
        "/dev/zram0 partition 999999999 0 100"
    ));
}

#[test]
fn hibernate_is_hidden_when_the_only_swap_is_zram() {
    // Este es el caso real de nixos-pc.
    let cmdline = "root=/dev/nvme0n1p2 quiet splash";
    let swaps = "/dev/zram0 partition 16393724 7578244 100";
    assert!(!can_hibernate_with_swap_size(Some(cmdline), swaps, 32788184));
}
