#!/bin/sh
# Install Catfyn from this folder.
#   ./install.sh              into ~/.local (no root needed)
#   sudo ./install.sh /usr/local
#   ./install.sh --uninstall [PREFIX]
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
uninstall=0
if [ "${1:-}" = "--uninstall" ]; then uninstall=1; shift; fi
prefix="${1:-$HOME/.local}"
files="bin/catfyn share/catfyn/catfyn-osd.lua share/catfyn/catfyn-input.conf
share/applications/io.github.coderastic.Catfyn.desktop
share/icons/hicolor/scalable/apps/io.github.coderastic.Catfyn.svg
share/metainfo/io.github.coderastic.Catfyn.metainfo.xml"
if [ "$uninstall" = 1 ]; then
  for f in $files; do rm -f "$prefix/$f"; done
  rmdir "$prefix/share/catfyn" 2>/dev/null || true
  echo "Catfyn removed from $prefix (your accounts stay in ~/.config/catfyn)"
  exit 0
fi
for f in $files; do
  mkdir -p "$(dirname "$prefix/$f")"
  cp "$here/$f" "$prefix/$f"
done
command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database -q "$prefix/share/applications" || true
command -v mpv >/dev/null 2>&1 || echo "Note: Catfyn plays videos with mpv. Install it with your package manager."
command -v curl >/dev/null 2>&1 || echo "Note: Catfyn needs curl. Install it with your package manager."
echo "Catfyn installed in $prefix (run: catfyn)"
