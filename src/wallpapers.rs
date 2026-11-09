// Wallpapers: lista los fondos disponibles (videos animados e imágenes del
// backdrop) como Items del launcher, separados en tres secciones:
//   - "Fondo de pantalla": videos (mpvpaper, capa por workspace)
//   - "Background": imágenes (niri-backdrop, capa detrás de todo)
//   - "Mis fondos": ~/.loon-launch/wallpapers (dotdir propio, mixto por extensión)
// Los videos se representan con un frame extraído con ffmpeg (cacheado en
// ~/.cache/loon-launch) para la miniatura de respaldo si el preview en vivo falla.
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::models::Item;

fn home_dir() -> PathBuf {
    env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/"))
}

fn videos_dir() -> PathBuf {
    videos_dir_for(&home_dir())
}

fn images_dir() -> PathBuf {
    images_dir_for(&home_dir())
}

fn cache_dir() -> PathBuf {
    env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir().join(".cache"))
        .join("loon-launch/thumbs")
}

/// Variantes puras para tests: resuelven bajo el `home` dado.
pub(crate) fn videos_dir_for(home: &Path) -> PathBuf {
    home.join("Videos/Wallpapers")
}

pub(crate) fn images_dir_for(home: &Path) -> PathBuf {
    home.join("Pictures/Wallpaper")
}

pub(crate) fn cache_dir_for(home: &Path) -> PathBuf {
    home.join(".cache/loon-launch/thumbs")
}

/// Directorio propio del usuario, fuera de los lugares comunes: aquí suelta
/// sus imágenes (y videos) y aquí se siembran las que trae la app por defecto.
fn custom_dir() -> PathBuf {
    custom_dir_for(&home_dir())
}

/// Variante pura para tests: resuelve bajo el `home` dado.
pub(crate) fn custom_dir_for(home: &Path) -> PathBuf {
    home.join(".loon-launch/wallpapers")
}

/// Extensiones que el selector acepta como fondo, en minúsculas (los tres
/// directorios aceptan las mismas: comunes y propio).
pub(crate) fn is_wallpaper_ext(ext: &str) -> bool {
    matches!(
        ext,
        "png" | "jpg" | "jpeg" | "webp" | "mp4" | "webm" | "mkv" | "mov" | "gif"
    )
}

/// De esas, las que se reproducen como video (mpvpaper + miniatura ffmpeg);
/// el resto va al backdrop como imagen estática. Recibe minúsculas.
pub(crate) fn is_video_ext(ext: &str) -> bool {
    matches!(ext, "mp4" | "webm" | "mkv" | "mov" | "gif")
}

/// Dónde puede estar `share/loon-launch/headers` instalado: XDG primero,
/// rutas clásicas después. La primera que exista gana.
fn headers_src_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let base = env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir().join(".local/share"));
    dirs.push(base.join("loon-launch/headers"));
    match env::var_os("XDG_DATA_DIRS") {
        Some(list) => {
            for d in std::env::split_paths(&list) {
                dirs.push(d.join("loon-launch/headers"));
            }
        }
        None => {
            dirs.push(PathBuf::from("/usr/local/share/loon-launch/headers"));
            dirs.push(PathBuf::from("/usr/share/loon-launch/headers"));
        }
    }
    dirs
}

/// Copia al dotdir lo que falte por nombre desde los headers instalados.
/// Devuelve cuántos copió. Pura en rutas explícitas para poder probarla.
pub(crate) fn seed_missing(dir: &Path, src: &Path) -> usize {
    let Ok(entries) = fs::read_dir(src) else {
        return 0;
    };
    let mut copied = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let dest = dir.join(entry.file_name());
        if !dest.exists() && fs::copy(&path, &dest).is_ok() {
            copied += 1;
        }
    }
    copied
}

/// Efecto de arranque del selector: garantiza `~/.loon-launch/wallpapers`.
/// La primera vez también siembra ahí las imágenes por defecto que trae el
/// paquete y deja el marcador `.seeded`: los borrados del usuario no
/// resucitan, y borrar el marcador re-siembra a pedido.
fn ensure_user_dirs() {
    let dir = custom_dir();
    if fs::create_dir_all(&dir).is_err() {
        return;
    }
    let marker = dir.join(".seeded");
    if marker.is_file() {
        return;
    }
    if let Some(src) = headers_src_dirs().into_iter().find(|d| d.is_dir()) {
        seed_missing(&dir, &src);
    }
    let _ = fs::write(
        &marker,
        "sembrado con las imágenes por defecto; bórralo para re-sembrar\n",
    );
}

pub fn wallpapers() -> Vec<Item> {
    // Efecto explícito y barato: crea ~/.loon-launch/wallpapers (sembrado
    // una sola vez) para que exista el lugar donde el usuario suelta las
    // suyas. Después solo lee: create_dir_all y el marcador son no-ops.
    ensure_user_dirs();
    let mut items = Vec::new();
    // Sección 1: Fondo de pantalla (videos animados).
    items.push(Item::header("Fondo de pantalla"));
    collect_dir(&videos_dir(), true, &mut items);
    // Sección 2: Background (imágenes estáticas del backdrop).
    items.push(Item::header("Background"));
    collect_dir(&images_dir(), false, &mut items);
    // Sección 3: Mis fondos (dotdir propio, clasificado por extensión:
    // cada archivo usa su comando; la UI ya filtra por el flag animado).
    items.push(Item::header("Mis fondos"));
    collect_mixed_dir(&custom_dir(), &mut items);
    items
}

/// Un Item de fondo desde su ruta: videos con miniatura ffmpeg + mpvpaper,
/// imágenes directas + backdrop. La UI ya filtra por `is_animated_wallpaper`.
fn wallpaper_item(path: &Path, is_video: bool) -> Item {
    let name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let thumb = if is_video {
        video_thumb(path)
    } else {
        path.to_string_lossy().to_string()
    };
    let exec = if is_video {
        format!("mpvpaper-wallpaper set {}", shell_quote(&name))
    } else {
        format!("niri-backdrop set {}", shell_quote(&name))
    };
    Item::wallpaper(name, exec, thumb)
        .with_media(path.to_string_lossy().into_owned())
        .with_animated_wallpaper(is_video)
}

fn collect_dir(dir: &Path, is_video: bool, out: &mut Vec<Item>) {
    if !dir.is_dir() {
        return;
    }
    if let Ok(entries) = fs::read_dir(dir) {
        let mut files: Vec<_> = entries
            .flatten()
            .filter(|e| {
                let ext = e
                    .path()
                    .extension()
                    .and_then(|x| x.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                is_wallpaper_ext(&ext)
            })
            .collect();
        files.sort_by_key(|e| e.file_name());
        for entry in files {
            out.push(wallpaper_item(&entry.path(), is_video));
        }
    }
}

/// El dotdir propio mezcla imágenes y videos: cada archivo se clasifica por
/// extensión en vez de asumir la sección, como hacen los directorios comunes.
fn collect_mixed_dir(dir: &Path, out: &mut Vec<Item>) {
    if !dir.is_dir() {
        return;
    }
    if let Ok(entries) = fs::read_dir(dir) {
        let mut files: Vec<_> = entries
            .flatten()
            .filter(|e| {
                let ext = e
                    .path()
                    .extension()
                    .and_then(|x| x.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                is_wallpaper_ext(&ext)
            })
            .collect();
        files.sort_by_key(|e| e.file_name());
        for entry in files {
            let path = entry.path();
            let ext = path
                .extension()
                .and_then(|x| x.to_str())
                .unwrap_or("")
                .to_lowercase();
            out.push(wallpaper_item(&path, is_video_ext(&ext)));
        }
    }
}

/// Extrae (y cachea) un frame de un video para usarlo de miniatura.
fn video_thumb(video: &Path) -> String {
    let Some(stem) = video.file_stem().and_then(|s| s.to_str()) else {
        return String::new();
    };
    fs::create_dir_all(cache_dir()).ok();
    let thumb = cache_dir().join(format!("{stem}.jpg"));
    let thumb = thumb.to_string_lossy().into_owned();
    if !Path::new(&thumb).is_file() {
        let _ = Command::new("ffmpeg")
            .args([
                "-y",
                "-ss",
                "00:00:01",
                "-i",
                video.to_str().unwrap_or(""),
                "-frames:v",
                "1",
                "-vf",
                "scale=640:-1",
                "-q:v",
                "4",
                &thumb,
            ])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    }
    thumb
}

/// Escapa un argumento para sh (nombres con espacios).
fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}
