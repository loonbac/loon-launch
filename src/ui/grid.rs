// Grid de apps + carrusel de wallpapers.
use gtk4::prelude::*;
use gtk4::{Image, Label, ListBoxRow, Orientation, Widget};
use libadwaita as adw;
use libadwaita::prelude::AnimationExt;
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use std::time::Duration;

use crate::filter::{filter_items, normalize_selection, SearchMode};
use crate::icons::resolve_icon;
use crate::models::Item;
use crate::ui::carousel::{build_carousel, CarouselRefs};
use crate::ui::layout::{
    cell_position, cell_width, section_rows, ICON_SIZE, LIST_PAD, LIST_SPACING, ROW_H, VISIBLE_W,
    WELL_H, WIN_W, WP_WIN_H, WP_WIN_W,
};

/// Referencias a los widgets del grid que usa el resto de la UI.
#[derive(Clone)]
pub struct GridRefs {
    pub grid: gtk4::Grid,
    pub carousel: CarouselRefs,
    pub scrolled: gtk4::ScrolledWindow,
    pub wallpaper_mode: Rc<RefCell<bool>>,
    pub wallpaper_static: Rc<RefCell<bool>>,
    pub media: Rc<RefCell<Vec<gtk4::MediaFile>>>,
    pub activate: Rc<RefCell<Option<Rc<dyn Fn()>>>>,
    pub cards: Rc<RefCell<Vec<Widget>>>,
    /// Barras de confirmación, en paralelo a `cards`. `Overlay` no expone sus
    /// overlays, así que se guardan aquí para no recrearlas en cada tick.
    hold_bars: Rc<RefCell<Vec<gtk4::Box>>>,
    pub positions: Rc<RefCell<Vec<(i32, i32)>>>,
    enter_anims: Rc<RefCell<Vec<adw::TimedAnimation>>>,
    scroll_anims: Rc<RefCell<Vec<adw::TimedAnimation>>>,
}

impl GridRefs {
    /// Cambia entre apps y selector de fondos. Es la única vía de entrada al
    /// selector: no se deduce de la query tecleada.
    pub fn set_wallpaper_mode(&self, on: bool, static_only: bool) {
        *self.wallpaper_mode.borrow_mut() = on;
        *self.wallpaper_static.borrow_mut() = on && static_only;
    }

    /// Limpia el grid y lo vuelve a poblar con los items que matchean `query`.
    /// `animate` solo en present/cambio de modo: no en cada tecla.
    pub fn repopulate(
        &self,
        all_apps: &[Item],
        power: &[Item],
        wallpapers: &[Item],
        query: &str,
        sel: &Rc<RefCell<i32>>,
        animate: bool,
    ) -> Vec<Item> {
        // El modo es estado de la UI, nunca se deduce de lo tecleado: `#` es
        // texto normal y no abre el selector de fondos.
        let mode = if *self.wallpaper_mode.borrow() {
            SearchMode::Wallpapers {
                static_only: *self.wallpaper_static.borrow(),
            }
        } else {
            SearchMode::Apps
        };
        let wallpaper_mode = mode != SearchMode::Apps;
        self.pause_media();
        self.media.borrow_mut().clear();
        self.cards.borrow_mut().clear();
        self.hold_bars.borrow_mut().clear();
        self.positions.borrow_mut().clear();
        for animation in self.enter_anims.borrow_mut().drain(..) {
            animation.pause();
        }
        for animation in self.scroll_anims.borrow_mut().drain(..) {
            animation.pause();
        }

        while let Some(child) = self.grid.first_child() {
            self.grid.remove(&child);
        }
        let shown = filter_items(all_apps, power, wallpapers, query, mode);
        self.apply_chrome(wallpaper_mode);

        let selectable: Vec<usize> = shown
            .iter()
            .enumerate()
            .filter(|(_, it)| !it.is_header)
            .map(|(i, _)| i)
            .collect();
        let new_sel = normalize_selection(*sel.borrow(), selectable.len());
        *sel.borrow_mut() = new_sel;
        let sel_item = new_sel
            .try_into()
            .ok()
            .and_then(|n: usize| selectable.get(n).copied());

        if wallpaper_mode {
            self.fill_gallery(&shown, sel_item, sel, animate);
        } else {
            self.fill_apps(&shown, sel_item, sel, animate);
        }

        shown
    }

    fn fill_gallery(
        &self,
        shown: &[Item],
        sel_item: Option<usize>,
        sel: &Rc<RefCell<i32>>,
        animate: bool,
    ) {
        let wallpaper_items = shown
            .iter()
            .filter(|item| !item.is_header)
            .cloned()
            .collect::<Vec<_>>();
        let selected = sel_item
            .map(|real_index| shown[..real_index].iter().filter(|item| !item.is_header).count() as i32)
            .unwrap_or(-1);

        self.positions
            .borrow_mut()
            .extend((0..wallpaper_items.len()).map(|index| (0, index as i32)));
        self.carousel.set_items(&wallpaper_items, selected.max(0));
        let carousel = self.carousel.clone();
        let selection = sel.clone();
        let activate = self.activate.clone();
        self.carousel.set_on_click(Rc::new(move |index, confirm| {
            *selection.borrow_mut() = index;
            carousel.set_selected(index, true);
            if confirm {
                if let Some(callback) = activate.borrow().as_ref() {
                    callback();
                }
            }
        }));

        if animate {
            animate_enter(&self.carousel.area, 20, &self.enter_anims);
        }
    }

    fn fill_apps(
        &self,
        shown: &[Item],
        sel_item: Option<usize>,
        sel: &Rc<RefCell<i32>>,
        animate: bool,
    ) {
        let mut item_idx = 0usize;
        let mut row = 0i32;
        let mut header_row = 0i32;
        let mut items_in_section = 0usize;
        let mut selectable_idx = 0i32;
        for (i, item) in shown.iter().enumerate() {
            let (cell, bar) = make_app_cell(item);
            if item.is_header {
                if items_in_section > 0 {
                    row = header_row + 1 + section_rows(items_in_section);
                }
                self.grid.attach(&cell, 0, row, 100, 1);
                header_row = row;
                row += 1;
                items_in_section = 0;
            } else {
                let (col, line) = cell_position(item_idx);
                self.grid
                    .attach(&cell, col, header_row + 1 + line, 1, 1);
                if Some(i) == sel_item {
                    cell.add_css_class("selected");
                }
                bind_click(&cell, selectable_idx, sel, self);
                // `positions` guarda (fila, columna): así lo recorre la
                // navegación de teclado en filter::move_sel_grid.
                self.positions.borrow_mut().push((line, col));
                self.cards.borrow_mut().push(cell.clone().upcast());
                // Sólo las tarjetas seleccionables entran en `cards`, y sus
                // barras entran aquí: ambos vectores quedan alineados por índice.
                if let Some(bar) = bar {
                    self.hold_bars.borrow_mut().push(bar);
                }
                if animate {
                    animate_enter(
                        &cell,
                        ((item_idx as u32) * 24).min(220),
                        &self.enter_anims,
                    );
                }
                item_idx += 1;
                items_in_section += 1;
                selectable_idx += 1;
            }
        }
    }

    /// Pinta el avance de una pulsación sostenida sobre la tarjeta seleccionada.
    /// `pct` va de 0.0 a 1.0; 0 quita la marca y deja la fila como estaba.
    /// La barra es un hijo real de la fila: crecerla no cambia la altura ni
    /// requiere inyectar CSS en cada tick.
    pub fn set_hold_progress(&self, idx: i32, pct: f64) {
        if *self.wallpaper_mode.borrow() {
            return;
        }
        let pct = pct.clamp(0.0, 1.0);
        if idx < 0 {
            return;
        }
        let Some(card) = self.cards.borrow().get(idx as usize).cloned() else {
            return;
        };
        let Some(bar) = self.hold_bar(idx as usize) else {
            return;
        };
        if pct <= 0.0 {
            card.remove_css_class("holding");
            bar.set_size_request(0, 3);
            bar.set_visible(false);
            return;
        }
        card.add_css_class("holding");
        bar.set_visible(true);
        // El ancho se interpola sobre el ancho útil de la tarjeta.
        let width = (card.width().max(1) as f64 * pct) as i32;
        bar.set_size_request(width.max(1), 3);
    }

    /// Devuelve la barra de progreso de una tarjeta por índice. Se guarda en
    /// una lista paralela a `cards` porque `Overlay` no expone sus overlays.
    fn hold_bar(&self, idx: usize) -> Option<gtk4::Box> {
        self.hold_bars.borrow().get(idx).cloned()
    }

    pub fn clear_hold(&self) {
        if *self.wallpaper_mode.borrow() {
            return;
        }
        let cards = self.cards.borrow().clone();
        for card in cards.iter() {
            card.remove_css_class("holding");
        }
        for bar in self.hold_bars.borrow().iter() {
            bar.set_size_request(0, 3);
            bar.set_visible(false);
        }
    }

    pub fn apply_sel(&self, idx: i32) {
        if *self.wallpaper_mode.borrow() {
            self.carousel.set_selected(idx, true);
            return;
        }
        let cards = self.cards.borrow();
        for (i, card) in cards.iter().enumerate() {
            if i as i32 == idx {
                card.add_css_class("selected");
            } else {
                card.remove_css_class("selected");
            }
        }
        if idx >= 0 {
            if let Some(card) = cards.get(idx as usize).cloned() {
                drop(cards);
                self.scroll_card(&card, true);
                let this = self.clone();
                let card = card.clone();
                glib::idle_add_local_once(move || {
                    this.scroll_card(&card, true);
                });
            }
        }
    }

    fn scroll_card(&self, card: &Widget, animate: bool) {
        // En fondos el scroll real es el strip de la fila, no el ScrolledWindow de afuera.
        let scrolled = nearest_scrolled(card).unwrap_or_else(|| self.scrolled.clone());
        let Some((x, y)) = card.translate_coordinates(&scrolled, 0.0, 0.0) else {
            return;
        };
        let w = f64::from(card.width().max(1));
        let ht = f64::from(card.height().max(1));
        let h = scrolled.hadjustment();
        let v = scrolled.vadjustment();
        // Dejar aire para el borde de la card: si no, al volver al primero
        // el scroll lo pega al recorte de la ventana.
        const PAD: f64 = 22.0;
        let mut hx = h.value();
        if x < PAD {
            hx = (h.value() + x - PAD).max(h.lower());
        } else if x + w > h.page_size() - PAD {
            hx = (h.value() + x + w - (h.page_size() - PAD))
                .min(h.upper() - h.page_size())
                .max(h.lower());
        }
        let mut vy = v.value();
        if y < PAD {
            vy = (v.value() + y - PAD).max(v.lower());
        } else if y + ht > v.page_size() - PAD {
            vy = (v.value() + y + ht - (v.page_size() - PAD))
                .min(v.upper() - v.page_size())
                .max(v.lower());
        }
        if animate {
            animate_adjustment(&scrolled, &h, hx, &self.scroll_anims);
            animate_adjustment(&scrolled, &v, vy, &self.scroll_anims);
        } else {
            h.set_value(hx);
            v.set_value(vy);
        }
    }

    pub fn pause_media(&self) {
        for media in self.media.borrow().iter() {
            media.pause();
        }
    }

    pub fn play_media(&self) {
        for media in self.media.borrow().iter() {
            media.play();
        }
    }

    fn apply_chrome(&self, wallpaper_mode: bool) {
        let well_h = if wallpaper_mode { WP_WIN_H } else { WELL_H };
        if wallpaper_mode {
            self.scrolled.set_child(Some(&self.carousel.area));
            self.scrolled.set_vscrollbar_policy(gtk4::PolicyType::Never);
            self.scrolled.set_hscrollbar_policy(gtk4::PolicyType::Never);
            // El ancho real lo decide la ventana según el output enfocado.
            // No imponer 1400 aquí permite que el carrusel quepa en el
            // monitor vertical de 1080 px.
            self.scrolled.set_size_request(-1, well_h);
            self.scrolled.set_margin_end(0);
            self.scrolled.set_min_content_width(0);
            self.scrolled.set_max_content_width(WP_WIN_W);
            self.scrolled.set_hexpand(true);
        } else {
            self.scrolled.set_child(Some(&self.grid));
            self.scrolled.set_vscrollbar_policy(gtk4::PolicyType::Never);
            // Automatic: si es Never, GTK ensancha la ventana para mostrar
            // todas las columnas. La barra se oculta por CSS.
            self.scrolled.set_hscrollbar_policy(gtk4::PolicyType::Automatic);
            self.scrolled.set_size_request(VISIBLE_W, well_h);
            self.scrolled.set_margin_end(WIN_W - VISIBLE_W);
            self.scrolled.set_min_content_width(VISIBLE_W);
            self.scrolled.set_max_content_width(VISIBLE_W);
            self.scrolled.set_hexpand(false);
        }
        // Liberar primero el máximo evita una aserción de GTK al pasar de
        // los 170 px del launcher normal a los 590 px del carrusel.
        self.scrolled.set_max_content_height(-1);
        self.scrolled.set_min_content_height(well_h);
        self.scrolled.set_max_content_height(well_h);
    }
}

/// Construye el grid + galería + scrolled window.
pub fn build_grid() -> GridRefs {
    let grid = gtk4::Grid::new();
    grid.set_row_spacing(LIST_SPACING as u32);
    grid.set_column_spacing(LIST_SPACING as u32);
    // El padding de la lista (rofi `listbox padding: 15px`) se aplica como
    // margen del grid; el derecho lo aporta la ventana, fuera del scroller.
    grid.set_margin_start(LIST_PAD);
    grid.set_margin_top(LIST_PAD);
    grid.set_margin_bottom(LIST_PAD);
    grid.set_halign(gtk4::Align::Center);
    grid.set_valign(gtk4::Align::Start);
    grid.set_focusable(true);

    let carousel = build_carousel();

    let well_h = WELL_H; // apps; el modo fondos lo cambia en apply_chrome
    let scrolled = gtk4::ScrolledWindow::builder()
        .child(&grid)
        .hexpand(false)
        .vexpand(false)
        .propagate_natural_width(false)
        .propagate_natural_height(false)
        .min_content_width(VISIBLE_W)
        .max_content_width(VISIBLE_W)
        .min_content_height(well_h)
        .max_content_height(well_h)
        .width_request(VISIBLE_W)
        .height_request(well_h)
        .hscrollbar_policy(gtk4::PolicyType::Automatic)
        .vscrollbar_policy(gtk4::PolicyType::Never)
        .focusable(true)
        .build();
    // El resto del ancho de la ventana es el padding derecho de la lista: vive
    // fuera del scroller para que la columna siguiente no se vea recortada.
    scrolled.set_margin_end(WIN_W - VISIBLE_W);

    GridRefs {
        grid,
        carousel,
        scrolled,
        wallpaper_mode: Rc::new(RefCell::new(false)),
        wallpaper_static: Rc::new(RefCell::new(false)),
        media: Rc::new(RefCell::new(Vec::new())),
        activate: Rc::new(RefCell::new(None)),
        cards: Rc::new(RefCell::new(Vec::new())),
        hold_bars: Rc::new(RefCell::new(Vec::new())),
        positions: Rc::new(RefCell::new(Vec::new())),
        enter_anims: Rc::new(RefCell::new(Vec::new())),
        scroll_anims: Rc::new(RefCell::new(Vec::new())),
    }
}

fn animate_adjustment(
    widget: &impl IsA<Widget>,
    adj: &gtk4::Adjustment,
    to: f64,
    hold: &Rc<RefCell<Vec<adw::TimedAnimation>>>,
) {
    let from = adj.value();
    if (from - to).abs() < 0.5 {
        return;
    }
    let adj = adj.clone();
    let target = adw::CallbackAnimationTarget::new(glib::clone!(
        #[strong]
        adj,
        move |v| {
            adj.set_value(v);
        }
    ));
    let anim = adw::TimedAnimation::new(widget, from, to, 260, target);
    anim.set_easing(adw::Easing::EaseOutCubic);
    anim.play();
    hold.borrow_mut().push(anim);
}

fn nearest_scrolled(widget: &Widget) -> Option<gtk4::ScrolledWindow> {
    let mut current = widget.parent();
    while let Some(p) = current {
        if let Ok(sw) = p.clone().downcast::<gtk4::ScrolledWindow>() {
            return Some(sw);
        }
        current = p.parent();
    }
    None
}

fn bind_click(
    cell: &impl IsA<Widget>,
    selectable_idx: i32,
    sel: &Rc<RefCell<i32>>,
    refs: &GridRefs,
) {
    let gesture = gtk4::GestureClick::new();
    gesture.connect_released(glib::clone!(
        #[strong]
        sel,
        #[strong]
        refs,
        move |_, _, _, _| {
            *sel.borrow_mut() = selectable_idx;
            refs.apply_sel(selectable_idx);
            if let Some(cb) = refs.activate.borrow().as_ref() {
                cb();
            }
        }
    ));
    cell.add_controller(gesture);
}

fn animate_enter(
    widget: &impl IsA<Widget>,
    delay_ms: u32,
    hold: &Rc<RefCell<Vec<adw::TimedAnimation>>>,
) {
    // Solo opacidad: animar márgenes cambia el tamaño de la ventana en cada frame.
    let w = widget.clone().upcast::<Widget>();
    w.set_opacity(0.0);
    let hold = hold.clone();
    glib::timeout_add_local_once(Duration::from_millis(delay_ms as u64), move || {
        let ww = w.clone();
        let target = adw::CallbackAnimationTarget::new(move |v| {
            ww.set_opacity(v);
        });
        let anim = adw::TimedAnimation::new(&w, 0.0, 1.0, 340, target);
        anim.set_easing(adw::Easing::EaseOutCubic);
        anim.play();
        hold.borrow_mut().push(anim);
    });
}

fn make_app_cell(item: &Item) -> (ListBoxRow, Option<gtk4::Box>) {
    let cell = ListBoxRow::new();

    if item.is_header {
        cell.set_size_request(-1, 30);
        cell.add_css_class("section-header-row");
        let label = Label::new(Some(&item.name));
        label.set_xalign(0.0);
        label.set_margin_start(12);
        label.add_css_class("section-header");
        cell.set_child(Some(&label));
        return (cell, None);
    }

    cell.set_size_request(cell_width(), ROW_H);

    // El padding de la celda (rofi `element padding: 8px`) lo pone el CSS
    // sobre `row`; aquí sólo el espaciado entre ícono y texto.
    let hbox = gtk4::Box::new(Orientation::Horizontal, 10);
    hbox.set_valign(gtk4::Align::Center);

    let image = Image::new();
    if item.is_wallpaper {
        let thumb_path = Path::new(&item.icon);
        if thumb_path.is_file() {
            if let Ok(pixbuf) =
                gtk4::gdk_pixbuf::Pixbuf::from_file_at_scale(thumb_path, 64, 40, true)
            {
                image.set_from_pixbuf(Some(&pixbuf));
            }
        }
    } else if let Some(icon) = resolve_icon(&item.icon) {
        image.set_paintable(Some(&icon));
    }
    image.set_pixel_size(if item.is_wallpaper { 40 } else { ICON_SIZE });
    if item.is_wallpaper {
        image.set_size_request(64, 40);
    } else {
        image.set_size_request(ICON_SIZE, ICON_SIZE);
    }
    image.set_valign(gtk4::Align::Center);
    hbox.append(&image);

    let label = Label::new(Some(&item.name));
    label.set_xalign(0.0);
    label.set_halign(gtk4::Align::Start);
    label.set_hexpand(true);
    label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    label.add_css_class("app-name");
    hbox.append(&label);

    // La barra de confirmación vive en un Overlay sobre el contenido, no dentro
    // del Box: como hijo del Box horizontal sería un tercer elemento al lado
    // del icono y la etiqueta, y empujaría el texto. En Overlay se dibuja
    // encima sin participar del layout.
    let overlay = gtk4::Overlay::new();
    overlay.set_child(Some(&hbox));
    let bar = gtk4::Box::new(Orientation::Horizontal, 0);
    bar.add_css_class("hold-bar");
    bar.set_halign(gtk4::Align::Start);
    bar.set_valign(gtk4::Align::End);
    bar.set_visible(false);
    overlay.add_overlay(&bar);

    cell.set_child(Some(&overlay));
    (cell, Some(bar))
}
