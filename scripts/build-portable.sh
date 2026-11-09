#!/usr/bin/env bash
# Compila el binario de release de loon-launch contra un glibc antiguo.
#
# Dos reglas para que arranque en la mayor parte de distribuciones:
#
#  1. Enlazar contra el glibc MAS ANTIGUO que se quiera soportar, nunca contra
#     el del sistema que compila. Por eso se compila en Debian 12 (glibc 2.36)
#     en vez de en la máquina del desarrollador: el mismo binario arranca en
#     Debian 12+, Ubuntu 22.04+, Fedora 36+ y Arch.
#  2. No usar el binario de Nix para paquetes nativos: lleva grabado el
#     intérprete del store (/nix/store/.../ld-linux-x86-64.so.2) y solo
#     arranca dentro de Nix.
#
# Excepción documentada: Debian 12 no empaqueta gtk4-layer-shell (llegó en
# trixie), así que esa librería se compila desde fuente (tag v1.0.4, el mismo
# que empaqueta trixie) y se vende DENTRO de nuestros paquetes en un
# directorio privado con un wrapper que fija LD_LIBRARY_PATH. No se instala
# en la ruta del sistema para no chocar con el paquete oficial donde sí
# existe. `scripts/package.sh` arma ese layout y `scripts/test-install.sh`
# lo comprueba en limpio.
#
# Es el binario que consume scripts/package.sh (ramas tarball/deb/rpm).
#
# Requisitos: docker (o podman). Sin motor de contenedores no hay build
# reproducible del glibc suelo: se aborta en vez de compilar contra el
# sistema y publicar un binario que no arranca en otras distros.
set -euo pipefail

# shellcheck source=scripts/version.sh
source "$(dirname "${BASH_SOURCE[0]}")/version.sh"

VERSION="$LOON_LAUNCH_VERSION"
BIN="loon-launch"
# Debian 12 trae glibc 2.36, que es el suelo razonable: Ubuntu 22.04, Fedora 36
# y RHEL/Rocky 9.3 arrancan con el mismo binario.
# Nombre completamente cualificado: los nombres cortos no se resuelven en un
# entorno limpio (podman exige el registro explícito).
IMAGE="docker.io/library/debian:12-slim"
# La misma versión que empaqueta Debian trixie (1.0.4-2): paridad con la
# distro que sí la trae y un tag inmutable en vez de una rama móvil.
LAYER_SHELL_TAG="v1.0.4"

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

OUT="target/portable"
rm -rf "$OUT"
mkdir -p "$OUT/bin" "$OUT/lib"

have_docker() { command -v docker >/dev/null 2>&1 && docker info >/dev/null 2>&1; }
have_podman() { command -v podman >/dev/null 2>&1 && podman info >/dev/null 2>&1; }

ENGINE=""
if have_docker; then
  ENGINE="docker"
elif have_podman; then
  ENGINE="podman"
else
  echo "error: sin docker ni podman no hay build portable (el glibc del sistema no vale como suelo)" >&2
  exit 1
fi

# `-i` no es opcional: sin ella el motor no engancha stdin, el heredoc de abajo
# nunca llega a `sh -s` y el contenedor sale con código 0 sin haber compilado
# nada. El fallo se descubre más tarde, cuando falta el binario.
echo "==> Compilando dentro de $IMAGE con $ENGINE"
"$ENGINE" run --rm -i \
  -v "$ROOT:/src:rw" \
  -w /src \
  "$IMAGE" \
  sh -s "$LAYER_SHELL_TAG" <<'DOCKER'
set -euo pipefail
LAYER_TAG="$1"

export DEBIAN_FRONTEND=noninteractive
apt-get update -qq
# Toolchain + cabeceras GTK + herramientas para compilar gtk4-layer-shell.
# `git` solo se usa para traer el tag de gtk4-layer-shell; el código de
# loon-launch entra por el bind mount, nunca se clona dentro.
apt-get install -y -qq --no-install-recommends \
  ca-certificates curl git pkg-config gcc \
  meson ninja-build \
  libgtk-4-dev libadwaita-1-dev libglib2.0-dev \
  libwayland-dev wayland-protocols

# Rust estable mínimo: solo lo que necesita cargo, sin componentes extra.
curl -fsSL --proto '=https' --tlsv1.2 https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain stable
export PATH="$HOME/.cargo/bin:$PATH"

# gtk4-layer-shell no existe en Debian 12: se compila el tag fijado arriba.
# `--depth 1 --branch` trae solo ese tag, no todo el historial.
rm -rf /tmp/gtk4-layer-shell
git clone --depth 1 --branch "$LAYER_TAG" https://github.com/wmww/gtk4-layer-shell /tmp/gtk4-layer-shell
meson setup /tmp/gtk4-layer-shell/build /tmp/gtk4-layer-shell
ninja -C /tmp/gtk4-layer-shell/build
DESTDIR=/ ninja -C /tmp/gtk4-layer-shell/build install
ldconfig

# El lock es la verdad: si Cargo.toml y Cargo.lock divergen, esto falla aquí
# en vez de publicar un binario con dependencias distintas a las del repo.
cargo build --release --locked
install -Dm755 "target/release/loon-launch" "/src/target/portable/bin/loon-launch"

# Se vende la librería compilada arriba: es lo que empaqueta package.sh en
# el directorio privado. Se resuelve por SONAME real, no por nombre supuesto.
LIB="$(ldconfig -p | grep 'libgtk4-layer-shell.so.0 ' | head -1 | tr -d ' ' | cut -d'>' -f2 || true)"
[ -n "$LIB" ] || { echo "error: no quedó instalada libgtk4-layer-shell.so.0" >&2; exit 1; }
cp -a "$LIB"* /src/target/portable/lib/ 2>/dev/null || cp -a "$LIB" /src/target/portable/lib/

# Guardarraíl: ningún 'not found' puede llegar a la release.
if ldd /src/target/portable/bin/loon-launch | grep -q 'not found'; then
  echo "error: el binario deja dependencias sin resolver:" >&2
  ldd /src/target/portable/bin/loon-launch >&2
  exit 1
fi
echo "==> binario y librería en target/portable:"
ls -la /src/target/portable/bin /src/target/portable/lib
DOCKER

echo "==> listo: $OUT/bin/$BIN + $OUT/lib/"
ls -la "$OUT/bin" "$OUT/lib"
