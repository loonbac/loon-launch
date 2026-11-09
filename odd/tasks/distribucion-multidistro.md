# Distribución multidistro de loon-launch

## Specs

- S1: `mi app launcher este disponible para cualquier persona` — objetivo del dueño.
- S2: `para los que usan nixos puedan ponerlo ahi el nombre y que se instale sin tantos pasos extras` — NixOS: declarar el nombre e instalar.
- S3: `si alguien de arch quiere instalarlo se descarga el binario y lo instala, no necesita tener nix` — Arch: binario descargable, sin nix.
- S4: `y lo mismo para las otras distros, eso quiero, para todas las distros de linux que este` — resto de distros Linux: mismo binario/paquetes nativos.
- S5: `no me gusta estar compilando a cada rato, tenemos cachix ya` + `que mi amigo no tenga que compilar tambien a cada rato, deberia poder descargarlo facil sin compilar` — Cachix `loon-flakes` publica cada build de `master`; PRs solo construyen.
- S6: PR #19 `fix(power): stop removing the hold ticker after it has already ended` cierra issue #18; sigue la visión (cambio mínimo en `src/ui/keys.rs`).

## Tasks

- [ ] T1 (S6): Revisar PR #19 contra la visión; merge cuando el CI en PR esté verde.
- [ ] T2 (S1): Anotar este objetivo en `AGENTS.md` del repo (sección Distribución, sin tocar el manifiesto).
- [ ] T3 (S5): CI de Cachix en `loon-launch`: `master` construye y publica `.#loon-launch`; PRs solo construyen (sin secreto, funciona desde forks).
- [ ] T4 (S2): Exponer `nixosModules.default` con `programs.loon-launch.enable` (paquete + servicio de usuario + atajos); documentar uso en README.
- [ ] T5 (S3,S4): Release portable sin nix: workflow `release.yml` (binario contra glibc antiguo + `dist/*.tar.gz` + `.deb`/`.rpm`), plantillas `scripts/package.sh` mínimas y prueba real de instalación.
- [ ] T6 (S1,S2): Merge + `nix flake update loon-launch` en `~/.nixos` + rebuild para tenerlo en el sistema.

## Log

- L1 (verbatim dueño): `ademas, apunta esto en mi agents que supongo sera importante, sabras que quiero que mi app launcher este disponible para cualquier persona, entonces lo ideal seria que para los que usan nixos puedan ponerlo ahi el nombre y que se instale sin tantos pasos extras, si alguien de arch quiere instalarlo se descarga el binario y lo instala, no necesita tener nix, y lo mismo para las otras distros, eso quiero, para todas las distros de linux que este, entonces haslo asi y arreglalo todo`
- L2: secreto `CACHIX_AUTH_TOKEN` ya configurado en el repo (2026-11-09) sin mostrar el valor.
- L3: issue #18 y PR #19 los abrió SaiToEJ (no el dueño); el PR dice `Closes #18`.
