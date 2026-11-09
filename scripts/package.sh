#!/usr/bin/env bash
# Empaqueta el binario de loon-launch para instalar sin Nix: tarball portable
# (Arch, manual), .deb (Debian/Ubuntu) y .rpm (Fedora).
#
# Se espera el binario ya construido: `target/portable/bin/loon-launch` (el
# que produce scripts/build-portable.sh dentro de un contenedor, contra un
# glibc antiguo) más `target/portable/lib/libgtk4-layer-shell.so.0*`. El
# script solo envuelve: no recompila, para que la release y el empaquetado no
# diverjan.
#
# gtk4-layer-shell se vende en un directorio PRIVADO con un wrapper que fija
# LD_LIBRARY_PATH (ver build-portable.sh por qué): nunca en la ruta del
# sistema, para no chocar con el paquete oficial donde sí existe.
set -euo pipefail

PKG="${1:-}"
# shellcheck source=scripts/version.sh
source "$(dirname "${BASH_SOURCE[0]}")/version.sh"
VERSION="$LOON_LAUNCH_VERSION"
BIN="loon-launch"
DIST="dist"
PKG_NAME="loon-launch"

die() { echo "error: $*" >&2; exit 1; }

BINARY=""
for candidate in "target/portable/bin/$BIN" "result/bin/$BIN"; do
  if [ -x "$candidate" ]; then
    BINARY="$candidate"
    break
  fi
done
[ -n "$PKG" ] || die "uso: $0 <tarball|deb|rpm>"
[ -n "$BINARY" ] || die "no encuentro el binario en target/portable/bin ni result/bin; construye con scripts/build-portable.sh"

# Un binario compilado con Nix lleva grabado el intérprete del store y solo
# arranca dentro de Nix. Publicado como paquete nativo, instala sin error y
# muere al ejecutar en una máquina limpia.
case "$(readelf -l "$BINARY" 2>/dev/null | sed -n 's/.*interpreter: \(.*\)\]/\1/p')" in
  /nix/store/*) die "el binario viene de Nix: no sirve para paquetes nativos; compila con scripts/build-portable.sh" ;;
esac

LIB=""
for f in target/portable/lib/libgtk4-layer-shell.so.0*; do
  [ -f "$f" ] && LIB="$f" && break
done
[ -n "$LIB" ] || die "falta target/portable/lib/libgtk4-layer-shell.so.0*; construye con scripts/build-portable.sh"

have_docker() { command -v docker >/dev/null 2>&1 && docker info >/dev/null 2>&1; }
have_podman() { command -v podman >/dev/null 2>&1 && podman info >/dev/null 2>&1; }

echo "==> Empaquetando $PKG desde $BINARY + $LIB"
mkdir -p "$DIST"

# Raíz común del paquete: el wrapper en bin/, lo real en libexec privada.
# El mismo árbol vale para tarball, deb y rpm: solo cambia el prefijo.
stage_tree() {
  local root="$1" prefix="$2"
  rm -rf "$root"
  mkdir -p "$root/$prefix/bin" "$root/$prefix/lib/$PKG_NAME" "$root/$prefix/share/applications" "$root/$prefix/share/$PKG_NAME"
  install -Dm755 "$BINARY" "$root/$prefix/lib/$PKG_NAME/$BIN.real"
  install -Dm644 "$LIB" "$root/$prefix/lib/$PKG_NAME/$(basename "$LIB")"
  [ -e "$root/$prefix/lib/$PKG_NAME/$(basename "$LIB")" ] || die "no se vendió la librería"
  cat > "$root/$prefix/bin/$BIN" <<WRAPPER
#!/bin/sh
# Dir privado con la layer-shell vendida (no existe en Debian 12/Ubuntu 22.04).
exec env LD_LIBRARY_PATH="$prefix/lib/$PKG_NAME:\$LD_LIBRARY_PATH" "$prefix/lib/$PKG_NAME/$BIN.real" "\$@"
WRAPPER
  chmod 755 "$root/$prefix/bin/$BIN"
  cp -r assets/headers "$root/$prefix/share/$PKG_NAME/headers"
  cat > "$root/$prefix/share/applications/$BIN.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Name=loon-launch
GenericName=App Launcher
Comment=App launcher Wayland para niri: apps + acciones de poder
Exec=$BIN
Terminal=false
Categories=Utility;
Keywords=launcher;apps;niri;
StartupNotify=true
DESKTOP
}

case "$PKG" in
  tarball)
    ROOT="$DIST/tarball-root"
    stage_tree "$ROOT" "loon-launch-$VERSION"
    cat > "$ROOT/loon-launch-$VERSION/INSTALL.txt" <<INSTALL
loon-launch $VERSION — tarball portable (sin Nix)
=================================================
Descomprime donde quieras y usa bin/loon-launch, o copia bin/, lib/ y share/
sobre /usr/local (queda /usr/local/bin/loon-launch).

Dependencias del sistema (la layer-shell ya viene vendida dentro):
- Debian/Ubuntu: sudo apt install libgtk-4-1 libadwaita-1-0
  (opcional, miniaturas de video: ffmpeg)
- Fedora: sudo dnf install gtk4 libadwaita
  (opcional: ffmpeg)
- Arch: sudo pacman -S gtk4 libadwaita
  (opcional: ffmpeg)

Tus fondos van en ~/.loon-launch/wallpapers/ (se crea solo, con las imágenes
que trae la app; suelta ahí las tuyas, imágenes o videos).

Atajos sugeridos en niri: Super+Space y Super+B apuntando a loon-launch.
INSTALL
    tar -C "$DIST/tarball-root" -czf "$DIST/$PKG_NAME-$VERSION-x86_64-linux.tar.gz" "loon-launch-$VERSION"
    echo "construido $DIST/$PKG_NAME-$VERSION-x86_64-linux.tar.gz"
    ;;

  deb)
    ROOT="$DIST/deb-root"
    stage_tree "$ROOT" "usr"
    mkdir -p "$ROOT/DEBIAN" "$ROOT/usr/share/doc/$PKG_NAME"
    cp README.md "$ROOT/usr/share/doc/$PKG_NAME/README.md" 2>/dev/null || true
    # dpkg no calcula Depends al armar a mano: lista explícita. La
    # layer-shell NO va aquí porque se vende privada (ver arriba).
    cat > "$ROOT/DEBIAN/control" <<CONTROL
Package: $PKG_NAME
Version: $VERSION
Section: utils
Priority: optional
Architecture: amd64
Depends: libgtk-4-1, libadwaita-1-0
Recommends: ffmpeg
Maintainer: loonbac <loonbac@users.noreply.github.com>
Description: App launcher Wayland para niri
 Lista las apps instaladas, filtra al escribir y ofrece acciones de poder
 con el prefijo '>'. Incluye selector de fondos de pantalla.
CONTROL
    if command -v dpkg-deb >/dev/null 2>&1; then
      dpkg-deb --build "$ROOT" "$DIST/$PKG_NAME-$VERSION.deb"
    else
      WORK="$DIST/.deb-build"; rm -rf "$WORK"; mkdir -p "$WORK/control" "$WORK/data"
      cp -a "$ROOT/DEBIAN/." "$WORK/control/"
      cp -a "$ROOT/." "$WORK/data/"
      tar -C "$WORK/control" -czf "$WORK/control.tar.gz" .
      tar -C "$WORK/data" -cJf "$WORK/data.tar.xz" .
      printf '2.0\n' > "$WORK/debian-binary"
      ( cd "$WORK" && ar rc "$OLDPWD/$DIST/$PKG_NAME-$VERSION.deb" debian-binary control.tar.gz data.tar.xz )
      rm -rf "$WORK"
      echo "construido $DIST/$PKG_NAME-$VERSION.deb (armado a mano, sin dpkg-deb)"
    fi
    ;;

  rpm)
    SPEC="$DIST/loon-launch.spec"
    ROOT="$DIST/rpm-root"
    stage_tree "$ROOT" "usr"
    cat > "$SPEC" <<SPEC
Name:           $PKG_NAME
Version:        $VERSION
Release:        1%{?dist}
Summary:        App launcher Wayland para niri
License:        MIT
BuildArch:      x86_64
Requires:       gtk4 libadwaita
Recommends:     ffmpeg
%description
Lista las apps instaladas, filtra al escribir y ofrece acciones de poder
con el prefijo '>'. Incluye selector de fondos de pantalla.
%install
cp -a %{_sourcedir}/rpm-root/* %{buildroot}/
%files
/usr/bin/$BIN
/usr/lib/$PKG_NAME/
/usr/share/applications/$BIN.desktop
/usr/share/$PKG_NAME/
%changelog
SPEC
    if command -v rpmbuild >/dev/null 2>&1; then
      rm -rf "$DIST/rpmbuild" && mkdir -p "$DIST/rpmbuild"/{SOURCES,SPECS,BUILD,BUILDROOT,RPMS,SRPMS}
      cp -a "$ROOT/." "$DIST/rpmbuild/SOURCES/rpm-root/"
      cp "$SPEC" "$DIST/rpmbuild/SPECS/"
      rpmbuild -bb --define "_topdir $PWD/$DIST/rpmbuild" "$DIST/rpmbuild/SPECS/loon-launch.spec"
    elif have_docker || have_podman; then
      ENGINE="docker"; have_docker || ENGINE="podman"
      echo "==> Construyendo el .rpm dentro de fedora:latest con $ENGINE"
      rm -rf "$DIST/rpmbuild" && mkdir -p "$DIST/rpmbuild"/{SOURCES,SPECS,BUILD,BUILDROOT,RPMS,SRPMS}
      cp -a "$ROOT/." "$DIST/rpmbuild/SOURCES/rpm-root/"
      cp "$SPEC" "$DIST/rpmbuild/SPECS/$PKG_NAME.spec"
      "$ENGINE" run --rm -i -v "$PWD:/src:rw" -w /src docker.io/library/fedora:latest sh -s <<'DOCKER'
set -eu
dnf -y install rpm-build
rpmbuild -bb --define "_topdir /src/dist/rpmbuild" /src/dist/rpmbuild/SPECS/loon-launch.spec
chmod -R a+rwX /src/dist/rpmbuild
DOCKER
    else
      echo "AVISO: solo se generó el .spec (sin rpmbuild ni contenedor)"
      exit 0
    fi
    built="$(find "$DIST/rpmbuild/RPMS" -type f -name '*.rpm' | head -1)"
    [ -n "$built" ] || die "rpmbuild no produjo ningún .rpm"
    cp "$built" "$DIST/$PKG_NAME-$VERSION.rpm"
    echo "construido $DIST/$PKG_NAME-$VERSION.rpm"
    ;;
esac
