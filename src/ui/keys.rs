// Manejo del teclado: flechas, enter, escape, backspace y escritura.
// La captura en la ventana evita que el Entry robe las flechas.
use glib::clone;
use gtk4::gdk::Key;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use crate::filter::{
    apply_backspace, apply_char, hold_is_armed, hold_progress, move_sel_grid, move_selection_wrap,
};
use crate::models::Item;
use crate::ui::grid::GridRefs;

#[derive(Clone)]
pub struct KeyState {
    pub grid_refs: GridRefs,
    pub entry: gtk4::Entry,
    pub sel_idx: Rc<RefCell<i32>>,
    /// Listas para repoblar sin pasar por el signal `changed` del Entry
    /// (issue #23: `set_text("")` es no-op con el Entry ya vacío).
    pub all_apps: Rc<RefCell<Vec<Item>>>,
    pub power: Rc<RefCell<Vec<Item>>>,
    pub wallpapers: Rc<RefCell<Vec<Item>>>,
    pub current_items: Rc<RefCell<Vec<Item>>>,
    pub hide: Rc<dyn Fn()>,
    pub run_selected: Rc<dyn Fn()>,
    /// Si la acción seleccionada exige mantener Enter (acción de poder).
    pub selected_needs_hold: Rc<dyn Fn() -> bool>,
}

/// Estado de una pulsación sostenida en curso.
#[derive(Default)]
struct HoldRun {
    started: Option<Instant>,
    ticker: Option<glib::SourceId>,
}

pub fn setup_key_controller(window: &gtk4::ApplicationWindow, state: KeyState) {
    let controller = gtk4::EventControllerKey::new();
    // Se guarda por controlador: sólo hay una pulsación sostenida a la vez.
    let hold: Rc<RefCell<HoldRun>> = Rc::new(RefCell::new(HoldRun::default()));
    controller.connect_key_pressed(clone!(
        #[strong]
        state,
        #[strong]
        hold,
        move |_, key, _, _| {
            let apply_sel = {
                let refs = state.grid_refs.clone();
                let sel_ref = state.sel_idx.clone();
                move |new_idx: i32| {
                    *sel_ref.borrow_mut() = new_idx;
                    refs.apply_sel(new_idx);
                }
            };

            let positions = state.grid_refs.positions.borrow().clone();
            let current = *state.sel_idx.borrow();
            let wallpaper_mode = *state.grid_refs.wallpaper_mode.borrow();

            match key {
                Key::Escape => {
                    (state.hide)();
                    glib::Propagation::Stop
                }
                Key::Down => {
                    let next = if wallpaper_mode {
                        move_selection_wrap(current, 1, positions.len())
                    } else {
                        move_sel_grid(current, 1, 0, &positions)
                    };
                    apply_sel(next);
                    glib::Propagation::Stop
                }
                Key::Up => {
                    let next = if wallpaper_mode {
                        move_selection_wrap(current, -1, positions.len())
                    } else {
                        move_sel_grid(current, -1, 0, &positions)
                    };
                    apply_sel(next);
                    glib::Propagation::Stop
                }
                Key::Right => {
                    let next = if wallpaper_mode {
                        move_selection_wrap(current, 1, positions.len())
                    } else {
                        move_sel_grid(current, 0, 1, &positions)
                    };
                    apply_sel(next);
                    glib::Propagation::Stop
                }
                Key::Left => {
                    let next = if wallpaper_mode {
                        move_selection_wrap(current, -1, positions.len())
                    } else {
                        move_sel_grid(current, 0, -1, &positions)
                    };
                    apply_sel(next);
                    glib::Propagation::Stop
                }
                Key::Return | Key::KP_Enter => {
                    // Acción de poder: no se ejecuta con un roce. Arranca una
                    // pulsación sostenida y sólo dispara si se mantiene el
                    // umbral; al soltar antes se cancela sin efecto.
                    if (state.selected_needs_hold)() {
                        let already = hold.borrow().started.is_some();
                        if !already {
                            start_hold(&hold, &state);
                        }
                        return glib::Propagation::Stop;
                    }
                    (state.run_selected)();
                    glib::Propagation::Stop
                }
                Key::Tab | Key::ISO_Left_Tab if wallpaper_mode => {
                    // Alterna animados/estáticos dentro del selector. El modo
                    // sigue siendo fondos: no se toca el texto del Entry.
                    let show_static = !*state.grid_refs.wallpaper_static.borrow();
                    state.grid_refs.set_wallpaper_mode(true, show_static);
                    *state.sel_idx.borrow_mut() = 0;
                    // Repoblado explícito: `set_text("")` no emite `changed`
                    // cuando el Entry ya está vacío (el caso normal: abrir
                    // Super+B y pulsar Tab sin teclear), y sin repoblado la
                    // lista visible no cambia aunque el flag sí (issue #23).
                    let apps = state.all_apps.borrow();
                    let power = state.power.borrow();
                    let wallpapers = state.wallpapers.borrow();
                    *state.current_items.borrow_mut() = state.grid_refs.repopulate(
                        &apps,
                        &power,
                        &wallpapers,
                        "",
                        &state.sel_idx,
                        true,
                    );
                    state.entry.set_text("");
                    state.grid_refs.scrolled.grab_focus();
                    glib::Propagation::Stop
                }
                Key::BackSpace => {
                    let text = state.entry.text().to_string();
                    state.entry.set_text(&apply_backspace(&text));
                    state.entry.set_position(-1);
                    glib::Propagation::Stop
                }
                _ => {
                    if let Some(c) = key.to_unicode() {
                        if !c.is_control() {
                            // Cualquier tecla nueva cancela una pulsación en
                            // curso: el usuario está cambiando de intención.
                            cancel_hold(&hold, &state);
                            let text = state.entry.text().to_string();
                            state.entry.set_text(&apply_char(&text, c));
                            state.entry.set_position(-1);
                        }
                        glib::Propagation::Stop
                    } else {
                        glib::Propagation::Proceed
                    }
                }
            }
        },
    ));

    // Al soltar Enter se decide: si no llegó al umbral, se cancela.
    controller.connect_key_released(clone!(
        #[strong]
        state,
        #[strong]
        hold,
        move |_, key, _, _| {
            if matches!(key, Key::Return | Key::KP_Enter) {
                finish_hold(&hold, &state);
            }
        }
    ));
    controller.set_propagation_phase(gtk4::PropagationPhase::Capture);
    window.add_controller(controller);
}

/// Arranca una pulsación sostenida y anima la barra de la tarjeta seleccionada
/// a ~60 fps. El ticker se cancela solo al alcanzar el umbral.
fn start_hold(hold: &Rc<RefCell<HoldRun>>, state: &KeyState) {
    let started = Instant::now();
    {
        let mut h = hold.borrow_mut();
        h.started = Some(started);
    }
    let idx = *state.sel_idx.borrow();
    state.grid_refs.set_hold_progress(idx, 0.0);
    let hold_tick = hold.clone();
    let refs = state.grid_refs.clone();
    let sel = state.sel_idx.clone();
    let ticker = glib::timeout_add_local(Duration::from_millis(16), move || {
        // Pinta el avance y decide si la fuente sigue viva. El préstamo se
        // suelta al terminar la expresión: retirar el SourceId de abajo
        // necesita un préstamo mutuo del mismo `HoldRun`.
        let keep_running = match hold_tick.borrow().started {
            None => false,
            Some(started) => {
                let elapsed = started.elapsed().as_millis() as u64;
                refs.set_hold_progress(*sel.borrow(), hold_progress(elapsed));
                // Umbral alcanzado: la barra queda llena y se deja de animar.
                // El disparo real ocurre al soltar, para no ejecutar dos veces.
                !hold_is_armed(elapsed)
            }
        };
        if keep_running {
            return glib::ControlFlow::Continue;
        }
        // La fuente termina aquí, así que hay que retirar su `SourceId` de
        // `HoldRun`. Si quedara guardado, `finish_hold` intentaría remover una
        // fuente ya destruida: `SourceId::remove()` paniquea dentro de glib y,
        // al correr en el trampolín C de `key-released`, ese panic no puede
        // desenrollarse, el launcher aborta y la acción de poder nunca se
        // ejecuta (así fallaba Apagar/Reiniciar/Suspender/Bloquear).
        hold_tick.borrow_mut().ticker = None;
        glib::ControlFlow::Break
    });
    let mut h = hold.borrow_mut();
    h.ticker = Some(ticker);
}

/// Resuelve la pulsación al soltar: ejecuta si llegó al umbral, cancela si no.
///
/// `ticker` sólo se guarda mientras la fuente siga viva: el propio ticker lo
/// retira al terminar, así que `remove()` de abajo sólo corre sobre una fuente
/// que todavía existe.
fn finish_hold(hold: &Rc<RefCell<HoldRun>>, state: &KeyState) {
    let (started, ticker) = {
        let mut h = hold.borrow_mut();
        let started = h.started.take();
        let ticker = h.ticker.take();
        (started, ticker)
    };
    if let Some(ticker) = ticker {
        ticker.remove();
    }
    state.grid_refs.clear_hold();
    let Some(started) = started else {
        return;
    };
    if hold_is_armed(started.elapsed().as_millis() as u64) {
        (state.run_selected)();
    }
}

/// Cancela una pulsación en curso sin ejecutar nada.
fn cancel_hold(hold: &Rc<RefCell<HoldRun>>, state: &KeyState) {
    let ticker = {
        let mut h = hold.borrow_mut();
        h.started = None;
        h.ticker.take()
    };
    if let Some(ticker) = ticker {
        ticker.remove();
    }
    state.grid_refs.clear_hold();
}
