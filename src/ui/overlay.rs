// El launcher como superficie layer-shell (`wlr-layer-shell`).
//
// En Wayland una ventana normal ocupa espacio en su workspace. En niri, abrirla
// sobre la última workspace vacía hace que el compositor cree otra para
// mantener su invariante de "siempre hay una vacía al final", y el usuario
// termina en un workspace que no pidió. Una superficie de capa no participa del
// layout en mosaico, así que abrir el launcher deja de mover a nadie.
//
// En X11, o en un compositor sin el protocolo, la app mantiene la ventana
// normal: layer-shell es una mejora, no un requisito para funcionar.

use gtk4::prelude::*;
use gtk4_layer_shell::{KeyboardMode, Layer, LayerShell};

/// Identificador de la superficie. Es el nombre que el compositor ve y con el
/// que se podrían escribir reglas de ventana.
pub const NAMESPACE: &str = "loon-launch";

/// La capa overlay solo se usa en Wayland y con soporte del compositor.
///
/// En cualquier otro caso se deja la ventana como estaba: una superficie de capa
/// sobre un backend que no la soporta no se mapea.
pub fn use_overlay(is_wayland: bool, layer_shell_supported: bool) -> bool {
    is_wayland && layer_shell_supported
}

/// Configuración de la superficie, como dato comprobable.
///
/// El `KeyboardMode` real sólo se observa a través del compositor, así que la
/// configuración se declara aquí y `apply` la consume: una prueba puede
/// afirmarla sin abrir una ventana.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Config {
    pub layer: Layer,
    pub keyboard: KeyboardMode,
    pub exclusive_zone: i32,
    pub namespace: &'static str,
}

/// La superficie es un overlay modal que no reserva espacio.
///
/// Teclado *on-demand*, no exclusivo. Medido en niri con la ventana ya abierta y
/// `niri msg action focus-window` hacia otra ventana:
///
/// - `Exclusive`: el compositor no retira el foco, `EventControllerFocus` nunca
///   emite `leave` y la capa queda encima de todo de forma permanente. El
///   launcher sólo se cierra con su propio toggle.
/// - `OnDemand`: el compositor sí transfiere el foco, `leave` se emite y
///   `hide_window` desmapea la capa. La escritura sigue funcionando: verificado
///   tecleando en la entrada con `wtype`, el texto aparece.
/// - `None`: la superficie no recibe teclado y la entrada no se puede usar.
pub const CONFIG: Config = Config {
    layer: Layer::Overlay,
    keyboard: KeyboardMode::OnDemand,
    exclusive_zone: 0,
    namespace: NAMESPACE,
};

/// Convierte la ventana en superficie layer-shell. Devuelve si lo aplicó.
///
/// Sin anclas y con zona exclusiva en cero, el compositor centra la superficie y
/// la deja a su tamaño natural, que es el comportamiento del launcher.
pub fn apply(window: &gtk4::ApplicationWindow) -> bool {
    let is_wayland = gtk4::gdk::Display::default()
        .map(|display| display.type_().name().contains("Wayland"))
        .unwrap_or(false);
    if !use_overlay(is_wayland, gtk4_layer_shell::is_supported()) {
        return false;
    }
    window.init_layer_shell();
    window.set_namespace(Some(CONFIG.namespace));
    window.set_layer(CONFIG.layer);
    window.set_keyboard_mode(CONFIG.keyboard);
    window.set_exclusive_zone(CONFIG.exclusive_zone);
    true
}
