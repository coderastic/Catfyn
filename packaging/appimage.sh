#!/usr/bin/env bash
# Build dist/Catfyn-<version>-x86_64.AppImage (Catfyn with its own mpv)
# from dist/stage (run stage.sh first). Needs patchelf and appimagetool
# (APPIMAGETOOL=path, or on PATH).
set -euo pipefail
cd "$(dirname "$0")/.."
VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)"
TOOL="${APPIMAGETOOL:-$(command -v appimagetool || true)}"
S=dist/stage
[[ -x "$TOOL" ]] || { echo "appimagetool not found (set APPIMAGETOOL)" >&2; exit 1; }
[[ -x "$S/bin/mpv" ]] || { echo "nothing staged: run packaging/stage.sh first" >&2; exit 1; }
command -v patchelf >/dev/null || { echo "patchelf not installed" >&2; exit 1; }

A=dist/Catfyn.AppDir
rm -rf "$A" && mkdir -p "$A/usr"
cp -a "$S/bin" "$S/lib" "$A/usr/"
patchelf --set-rpath '$ORIGIN/../lib' "$A/usr/bin/mpv"
for f in "$A"/usr/lib/*; do patchelf --set-rpath '$ORIGIN' "$f"; done
install -Dm644 assets/catfyn-osd.lua "$A/usr/share/catfyn/catfyn-osd.lua"
install -Dm644 assets/catfyn-input.conf "$A/usr/share/catfyn/catfyn-input.conf"
install -Dm644 packaging/io.github.coderastic.Catfyn.metainfo.xml \
  "$A/usr/share/metainfo/io.github.coderastic.Catfyn.appdata.xml"
install -Dm644 packaging/io.github.coderastic.Catfyn.svg \
  "$A/usr/share/icons/hicolor/scalable/apps/io.github.coderastic.Catfyn.svg"
cp packaging/io.github.coderastic.Catfyn.desktop packaging/io.github.coderastic.Catfyn.svg "$A/"
cat > "$A/AppRun" <<'RUN'
#!/bin/sh
HERE="$(dirname "$(readlink -f "$0")")"
export PATH="$HERE/usr/bin:$PATH"
export CATFYN_DATA="$HERE/usr/share/catfyn"
exec "$HERE/usr/bin/catfyn" "$@"
RUN
chmod 755 "$A/AppRun"
OUT="dist/Catfyn-$VERSION-x86_64.AppImage"
ARCH=x86_64 "$TOOL" --appimage-extract-and-run --no-appstream "$A" "$OUT" >/dev/null
rm -rf "$A"
(cd dist && sha256sum "$(basename "$OUT")" > "$(basename "$OUT").sha256")
echo "$OUT"
