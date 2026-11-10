#!/usr/bin/env bash
# Remove what install.sh put in place (fork-only; see FORK.md).
#   ./uninstall.sh                     -> from ~/.local; your settings stay for a reinstall
#   ./uninstall.sh --purge             -> and reset Era's settings for the current user
#   PREFIX=/usr sudo ./uninstall.sh    -> a system-wide install
# Both backends (eds and mock) install the same binary, so both are removed.
# Your calendars live in evolution-data-server and are never touched.
set -euo pipefail
cd "$(dirname "$0")"

PREFIX="${PREFIX:-$HOME/.local}"
BIN="$PREFIX/bin"
SHARE="$PREFIX/share"
LIBDIR="$PREFIX/lib/era"
BASE_ID=org.gnome.gitlab.TitouanReal.Era
APP_IDS=("$BASE_ID" "$BASE_ID.Mock")

purge=false
for arg in "$@"; do
  case $arg in
    --purge) purge=true ;;
    -h|--help) sed -n '2,7p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) echo "unknown option: $arg (try --help)" >&2; exit 2 ;;
  esac
done

# A running Era quits the way Quit does; if that doesn't reach it, it is stopped.
if pgrep -x era >/dev/null 2>&1; then
  for id in "${APP_IDS[@]}"; do gapplication action "$id" quit 2>/dev/null || true; done
  for _ in 1 2 3 4 5; do pgrep -x era >/dev/null 2>&1 || break; sleep 1; done
  pkill -x era 2>/dev/null || true
fi

rm -f "$BIN/era"
rm -rf "$LIBDIR"
for id in "${APP_IDS[@]}"; do
  rm -f "$SHARE/applications/$id.desktop" \
        "$SHARE/dbus-1/services/$id.service" \
        "$SHARE/metainfo/$id.metainfo.xml" \
        "$SHARE/icons/hicolor/scalable/apps/$id.svg" \
        "$SHARE/icons/hicolor/symbolic/apps/$id-symbolic.svg"
done
# Every locale, not just today's po/LINGUAS: a sync may have dropped one.
rm -f "$SHARE"/locale/*/LC_MESSAGES/era.mo
gtk4-update-icon-cache -q -t -f "$SHARE/icons/hicolor" 2>/dev/null ||
  gtk-update-icon-cache -q -t -f "$SHARE/icons/hicolor" 2>/dev/null || true
update-desktop-database -q "$SHARE/applications" 2>/dev/null || true
[[ -n ${DBUS_SESSION_BUS_ADDRESS:-} ]] && dbus-send --session --dest=org.freedesktop.DBus \
  --type=method_call / org.freedesktop.DBus.ReloadConfig >/dev/null 2>&1 || true

# Settings are in dconf, not files; both app IDs share one path. Reset them
# while the schema is still installed to describe them.
if $purge; then
  dconf reset -f /org/gnome/gitlab/TitouanReal/Era/ 2>/dev/null || true
fi
for id in "${APP_IDS[@]}"; do
  rm -f "$SHARE/glib-2.0/schemas/$id.gschema.xml"
done
glib-compile-schemas "$SHARE/glib-2.0/schemas" >/dev/null 2>&1 || true

if $purge; then
  echo "Reset Era's settings. Your calendars are in evolution-data-server and were not touched."
else
  echo "Your settings are kept in dconf; --purge resets them."
fi
echo "Uninstalled from $PREFIX. Build files remain in target/native; delete it to free the space."
