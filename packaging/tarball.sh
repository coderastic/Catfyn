#!/usr/bin/env bash
# Build dist/catfyn-<version>-x86_64.tar.gz: the binary, the player files,
# the desktop entry and icon, and install.sh. Needs mpv and curl on the
# system it runs on. Uses the binary staged by stage.sh.
set -euo pipefail
cd "$(dirname "$0")/.."
VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)"
NAME="catfyn-$VERSION-x86_64"
[[ -x dist/stage/bin/catfyn ]] || { echo "nothing staged: run packaging/stage.sh first" >&2; exit 1; }
rm -rf "dist/$NAME" && mkdir -p "dist/$NAME"
D="dist/$NAME"
install -Dm755 dist/stage/bin/catfyn "$D/bin/catfyn"
install -Dm644 assets/catfyn-osd.lua "$D/share/catfyn/catfyn-osd.lua"
install -Dm644 assets/catfyn-input.conf "$D/share/catfyn/catfyn-input.conf"
install -Dm644 packaging/io.github.coderastic.Catfyn.desktop "$D/share/applications/io.github.coderastic.Catfyn.desktop"
install -Dm644 packaging/io.github.coderastic.Catfyn.svg "$D/share/icons/hicolor/scalable/apps/io.github.coderastic.Catfyn.svg"
install -Dm644 packaging/io.github.coderastic.Catfyn.metainfo.xml "$D/share/metainfo/io.github.coderastic.Catfyn.metainfo.xml"
install -Dm755 packaging/install.sh "$D/install.sh"
install -Dm644 README.md "$D/README.md"
install -Dm644 LICENSE "$D/LICENSE"
install -Dm644 assets/fonts/LICENSE-Inter.txt "$D/LICENSE-Inter.txt"
tar -C dist -czf "dist/$NAME.tar.gz" "$NAME"
rm -rf "dist/$NAME"
(cd dist && sha256sum "$NAME.tar.gz" > "$NAME.tar.gz.sha256")
echo "dist/$NAME.tar.gz"
