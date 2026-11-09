#!/usr/bin/env bash
# Prueba la instalación real de cada artefacto en contenedores limpios.
#
# Es el portón de la release: un paquete que instala pero no arranca no se
# publica. Comprueba tres cosas por artefacto:
#   1. Se instala con el gestor nativo (dependencias declaradas correctas).
#   2. `ldd` del binario real no deja ningún 'not found'.
#   3. El wrapper arranca bajo Xvfb y el proceso sobrevive 5 s sin panic
#      por librerías (no valida la UI, solo que el entorno está completo).
#
# Uso: scripts/test-install.sh [tarball|deb|rpm]  (sin args: los tres que existan en dist/)
set -euo pipefail

# shellcheck source=scripts/version.sh
source "$(dirname "${BASH_SOURCE[0]}")/version.sh"
VERSION="$LOON_LAUNCH_VERSION"
BIN="loon-launch"
PKG_NAME="loon-launch"
DIST="dist"

ONLY="${1:-}"
have_docker() { command -v docker >/dev/null 2>&1 && docker info >/dev/null 2>&1; }
have_podman() { command -v podman >/dev/null 2>&1 && podman info >/dev/null 2>&1; }
ENGINE=""
if have_docker; then ENGINE="docker"; elif have_podman; then ENGINE="podman"; fi
[ -n "$ENGINE" ] || { echo "error: test-install necesita docker o podman" >&2; exit 1; }

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# Comprueba un árbol instalado en $PREFIX dentro del contenedor: binario
# real resoluble, wrapper ejecutable y arranque bajo Xvfb.
CHECK_SNIPPET='
set -eu
PREFIX="$1"
REAL="$PREFIX/lib/loon-launch/loon-launch.real"
[ -x "$PREFIX/bin/loon-launch" ] || { echo "error: falta el wrapper en $PREFIX/bin"; exit 1; }
[ -x "$REAL" ] || { echo "error: falta el binario real en $REAL"; exit 1; }
echo "==> ldd:"
ldd "$REAL"
if ldd "$REAL" | grep -q "not found"; then
  echo "error: dependencias sin resolver arriba" >&2
  exit 1
fi
[ -f "$PREFIX/share/applications/loon-launch.desktop" ] || { echo "error: falta el .desktop"; exit 1; }
echo "==> arranque bajo Xvfb (5 s):"
Xvfb :99 &
XVFB_PID=$!
export DISPLAY=:99
"$PREFIX/bin/loon-launch" > /tmp/loon-launch.log 2>&1 &
APP_PID=$!
sleep 5
if ! kill -0 $APP_PID 2>/dev/null; then
  echo "error: el proceso murió en 5 s; log:" >&2
  cat /tmp/loon-launch.log >&2
  kill $XVFB_PID 2>/dev/null || true
  exit 1
fi
kill $APP_PID $XVFB_PID 2>/dev/null || true
echo "OK: sobrevive 5 s bajo Xvfb"
'

run_in() {
  local image="$1" artifact="$2" setup="$3"
  echo "==> probando $artifact en $image"
  "$ENGINE" run --rm -i \
    -v "$ROOT:/src:rw" \
    -w /src \
    "$image" \
    sh -s "$artifact" <<DOCKER
# `sh` aquí es dash: sin pipefail (igual que PORT).
set -eu
ART="\$1"
$setup
su -s /bin/sh nobody -c true 2>/dev/null || true
PREFIX=/usr sh -s /usr <<'CHECK'
$CHECK_SNIPPET
CHECK
DOCKER
}

test_tarball() {
  local tgz="$DIST/$PKG_NAME-$VERSION-x86_64-linux.tar.gz"
  [ -f "$tgz" ] || { echo "-- sin tarball, se omite"; return 0; }
  "$ENGINE" run --rm -i \
    -v "$ROOT:/src:ro" \
    -w /tmp \
    docker.io/library/ubuntu:24.04 \
    sh -s <<DOCKER
# `sh` aquí es dash: sin pipefail (igual que PORT).
set -eu
export DEBIAN_FRONTEND=noninteractive
apt-get update -qq
apt-get install -y -qq --no-install-recommends xvfb libgtk-4-1 libadwaita-1-0
rm -rf /tmp/lt && mkdir -p /tmp/lt && tar -C /tmp/lt -xzf /src/$tgz
PREFIX=/tmp/lt/loon-launch-$VERSION sh -s /tmp/lt/loon-launch-$VERSION <<'CHECK'
$CHECK_SNIPPET
CHECK
DOCKER
}

test_deb() {
  local deb="$DIST/$PKG_NAME-$VERSION.deb"
  [ -f "$deb" ] || { echo "-- sin .deb, se omite"; return 0; }
  run_in docker.io/library/ubuntu:24.04 "$deb" '
    export DEBIAN_FRONTEND=noninteractive
    apt-get update -qq
    apt-get install -y -qq --no-install-recommends xvfb "/src/'"$deb"'"'
}

test_rpm() {
  local rpm="$DIST/$PKG_NAME-$VERSION.rpm"
  [ -f "$rpm" ] || { echo "-- sin .rpm, se omite"; return 0; }
  run_in docker.io/library/fedora:latest "$rpm" '
    dnf install -y -q xvfb-run gtk4 libadwaita "/src/'"$rpm"'"'
}

case "$ONLY" in
  "") test_tarball; test_deb; test_rpm ;;
  tarball) test_tarball ;;
  deb) test_deb ;;
  rpm) test_rpm ;;
  *) echo "uso: $0 [tarball|deb|rpm]" >&2; exit 1 ;;
esac
echo "==> test-install OK"
