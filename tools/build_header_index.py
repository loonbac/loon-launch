#!/usr/bin/env python3
"""Genera el índice de color de las cabeceras del launcher.

Para cada imagen de `assets/headers/` calcula el tono dominante una sola vez y lo
deja en `assets/headers/index.tsv` (una línea por imagen: archivo, tono,
saturación, luminosidad). Así el launcher no decodifica 400 imágenes en cada
apertura: elige entre las que se parecen al acento del wallpaper con los datos ya
calculados, y el formato es trivial de leer sin dependencias.

Uso:  python3 tools/build_header_index.py
"""

import colorsys
import pathlib
import shutil
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor

ROOT = pathlib.Path(__file__).resolve().parent.parent
HEADERS = ROOT / "assets" / "headers"
INDEX = HEADERS / "index.tsv"
SUFFIXES = {".png", ".jpg", ".jpeg", ".webp"}


def dominant(image: pathlib.Path):
    """Tono dominante de la imagen, reducida a un píxel."""
    try:
        out = subprocess.run(
            ["magick", str(image), "-resize", "1x1!", "-format",
             "%[fx:int(255*r)],%[fx:int(255*g)],%[fx:int(255*b)]", "info:"],
            capture_output=True,
            text=True,
            check=True,
        ).stdout
    except (subprocess.CalledProcessError, FileNotFoundError):
        return None
    # Formato: "120,80,60" gracias a `fx` (0-255, sin porcentajes).
    digits = out.strip().split(",")
    if len(digits) != 3:
        return None
    try:
        r, g, b = (int(float(value)) / 255.0 for value in digits[:3])
    except ValueError:
        return None
    hue, lightness, saturation = colorsys.rgb_to_hls(r, g, b)
    return {
        "hue": round(hue * 360.0, 1),
        "sat": round(saturation, 3),
        "light": round(lightness, 3),
    }


def main():
    if shutil.which("magick") is None:
        sys.exit("falta ImageMagick: corre el script con `nix shell nixpkgs#imagemagick`")
    images = sorted(p for p in HEADERS.iterdir() if p.suffix.lower() in SUFFIXES)
    if not images:
        sys.exit(f"sin imágenes en {HEADERS}")

    entries = []
    with ThreadPoolExecutor(max_workers=8) as pool:
        for image, colour in zip(images, pool.map(dominant, images)):
            if colour is None:
                print(f"omitida (no se pudo leer): {image.name}", file=sys.stderr)
                continue
            entries.append({"file": image.name, **colour})

    if len(entries) < len(images) // 2:
        sys.exit(f"solo se indexaron {len(entries)} de {len(images)} imágenes: revisa ImageMagick")

    lines = ["# loon-launch headers v1: archivo\ttono\tsaturacion\tluminosidad"]
    lines += [
        f"{entry['file']}\t{entry['hue']}\t{entry['sat']}\t{entry['light']}"
        for entry in entries
    ]
    INDEX.write_text("\n".join(lines) + "\n")
    print(f"{len(entries)} cabeceras indexadas en {INDEX.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
