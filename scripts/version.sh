#!/usr/bin/env bash
# Fuente única de la versión del proyecto.
#
# La versión sale de una sola parte, que además es la que ya usa cargo, de
# modo que no puede haber dos verdades: publicar un tag vX.Y.Z genera un
# artefacto llamado loon-launch-X.Y.Z.
#
# Uso:  source scripts/version.sh ; $LOON_LAUNCH_VERSION
LOON_LAUNCH_VERSION="$(sed -n 's/^version = "\(.*\)"$/\1/p' "$(dirname "${BASH_SOURCE[0]}")/../Cargo.toml" | head -1)"
: "${LOON_LAUNCH_VERSION:?no se pudo leer la version de Cargo.toml}"
