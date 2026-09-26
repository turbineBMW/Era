#!/usr/bin/env bash
# Install Era for the current user, without Flatpak (fork-only; see FORK.md).
#   ./install.sh                     -> ~/.local (binary in ~/.local/bin)
#   PREFIX=/usr sudo ./install.sh    -> system-wide
#   BACKEND=mock ./install.sh        -> demo data instead of your calendars
#
# Era talks to its calendar backend through clepsydre, whose C libraries
# nobody packages yet. This builds them at the commit Cargo.lock pins, keeps
# them in $PREFIX/lib/era, and bakes that directory into era's rpath.
# Everything intermediate lives under target/native.
set -euo pipefail
cd "$(dirname "$0")"

PREFIX="${PREFIX:-$HOME/.local}"
BACKEND="${BACKEND:-eds}"
BIN="$PREFIX/bin"
SHARE="$PREFIX/share"
LIBDIR="$PREFIX/lib/era"
BASE_ID=org.gnome.gitlab.TitouanReal.Era
case $BACKEND in
  eds) APP_ID=$BASE_ID ;;
  mock) APP_ID=$BASE_ID.Mock ;;
  *) echo "BACKEND must be eds or mock" >&2; exit 1 ;;
esac

WORK="$PWD/target/native"
STAGE="$WORK/stage"            # clepsydre headers, libraries, .pc, typelib
export PKG_CONFIG_PATH="$STAGE/lib/pkgconfig${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}"
export GI_TYPELIB_PATH="$STAGE/lib/girepository-1.0${GI_TYPELIB_PATH:+:$GI_TYPELIB_PATH}"
export LD_LIBRARY_PATH="$STAGE/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"

# --- Dependencies ------------------------------------------------------------
# Checked up front so a missing one is named, with the package to install,
# instead of surfacing halfway through a build.
if command -v pacman >/dev/null 2>&1; then distro=arch
elif command -v apt-get >/dev/null 2>&1; then distro=debian
elif command -v dnf >/dev/null 2>&1; then distro=fedora
else distro=unknown; fi

missing="" what=""
# need <what> <arch pkg> <debian pkg> <fedora pkg>
need() {
  local pkg
  case $distro in arch) pkg=$2 ;; debian) pkg=$3 ;; fedora) pkg=$4 ;; *) pkg=$1 ;; esac
  what="$what
  - $1"
  case " $missing " in *" $pkg "*) ;; *) missing="$missing $pkg" ;; esac
}
have_cmd() { command -v "$1" >/dev/null 2>&1; }
have_lib() { pkg-config --exists "$1${2:+ >= $2}" 2>/dev/null; }

have_cmd cargo || need cargo rust cargo cargo
have_cmd git || need git git git git
have_cmd blueprint-compiler || need blueprint-compiler blueprint-compiler blueprint-compiler blueprint-compiler
have_cmd glib-compile-schemas || need glib-compile-schemas glib2 libglib2.0-dev-bin glib2-devel
have_cmd g-ir-scanner || need g-ir-scanner gobject-introspection gobject-introspection gobject-introspection-devel
if [[ $BACKEND == eds ]]; then
  have_cmd meson || need meson meson meson meson
  have_cmd ninja || need ninja ninja ninja-build ninja-build
fi
if have_cmd pkg-config; then
  have_lib gtk4 4.22 || need "gtk4 >= 4.22" gtk4 libgtk-4-dev gtk4-devel
  have_lib libadwaita-1 1.9 || need "libadwaita >= 1.9" libadwaita libadwaita-1-dev libadwaita-devel
  if [[ $BACKEND == eds ]]; then
    have_lib libecal-2.0 3.58.2 ||
      need "evolution-data-server >= 3.58.2" evolution-data-server evolution-data-server-dev evolution-data-server-devel
  fi
else
  need pkg-config pkgconf pkg-config pkgconf-pkg-config
fi

if [[ -n $missing ]]; then
  echo "Era can't be built; missing:$what" >&2
  echo "Install them with:" >&2
  case $distro in
    arch) echo "  sudo pacman -S --needed$missing" >&2 ;;
    debian) echo "  sudo apt install$missing" >&2 ;;
    fedora) echo "  sudo dnf install$missing" >&2 ;;
    *) echo " $missing" >&2 ;;
  esac
  exit 1
fi

# style.scss is compiled with grass, as upstream's meson build does.
GRASS=$(command -v grass || true)
if [[ -z $GRASS ]]; then
  GRASS="$WORK/tools/bin/grass"
  [[ -x $GRASS ]] || cargo install --quiet --root "$WORK/tools" grass
fi

# --- clepsydre ---------------------------------------------------------------
# The same commit the Rust bindings in Cargo.lock were generated against.
REV=$(sed -n 's|^source = "git+https://gitlab.gnome.org/TitouanReal/clepsydre.git#\(.*\)"|\1|p' Cargo.lock | head -1)
[[ -n $REV ]] || { echo "no clepsydre commit in Cargo.lock" >&2; exit 1; }
SRC="$WORK/clepsydre"
STAMP="$STAGE/.built-$REV-$BACKEND"

if [[ ! -f $STAMP ]]; then
  echo "==> building clepsydre ${REV:0:10}"
  if [[ ! -d $SRC/.git ]]; then
    git init --quiet "$SRC"
    git -C "$SRC" remote add origin https://gitlab.gnome.org/TitouanReal/clepsydre.git
  fi
  git -C "$SRC" fetch --quiet --depth 1 origin "$REV"
  git -C "$SRC" checkout --quiet --force FETCH_HEAD
  rm -rf "$STAGE"
  mkdir -p "$STAGE/lib/pkgconfig" "$STAGE/lib/girepository-1.0" "$STAGE/include/clepsydre-0" "$STAGE/share/gir-1.0"

  # libclepsydre is a Rust cdylib with a C ABI. Upstream builds it with
  # meson's Cargo support, which can't run build scripts yet; cargo can.
  CARGO_TARGET_DIR="$WORK/clepsydre-target" cargo build --quiet --release \
    --manifest-path "$SRC/Cargo.toml" -p clepsydre
  install -m755 "$WORK/clepsydre-target/release/libclepsydre.so" "$STAGE/lib/libclepsydre-0.so.0"
  ln -sf libclepsydre-0.so.0 "$STAGE/lib/libclepsydre-0.so"
  ln -sf libclepsydre-0.so.0 "$STAGE/lib/libclepsydre.so"  # what clepsydre-sys links
  install -m644 "$SRC/clepsydre/include/clepsydre.h" "$STAGE/include/clepsydre-0/"
  cat > "$STAGE/lib/pkgconfig/clepsydre-0.pc" <<EOF
prefix=$STAGE
libdir=\${prefix}/lib
includedir=\${prefix}/include
Name: clepsydre-0
Description: Clepsydre: Manage calendars and events
Version: 0.1.0
Requires: gio-2.0 gtk4
Libs: -L\${libdir} -lclepsydre-0
Cflags: -I\${includedir}/clepsydre-0
EOF

  # Era's Blueprint templates name Clepsydre types, so they need its typelib.
  (cd "$WORK" && g-ir-scanner --quiet --no-libtool \
    --namespace=Clepsydre --nsversion=0 \
    --identifier-prefix=Clepsydre --symbol-prefix=clepsydre \
    --include=GLib-2.0 --include=GObject-2.0 --include=Gio-2.0 --include=Gdk-4.0 \
    --pkg=gtk4 --library=clepsydre-0 --library-path="$STAGE/lib" \
    --c-include=clepsydre.h --pkg-export=clepsydre-0 \
    -o "$STAGE/share/gir-1.0/Clepsydre-0.gir" "$STAGE/include/clepsydre-0/clepsydre.h" 2>/dev/null)
  g-ir-compiler "$STAGE/share/gir-1.0/Clepsydre-0.gir" \
    -o "$STAGE/lib/girepository-1.0/Clepsydre-0.typelib"

  # libclepsydre-eds is plain C, which meson builds fine.
  if [[ $BACKEND == eds ]]; then
    rm -rf "$WORK/clepsydre-eds-build"
    meson setup --quiet "$WORK/clepsydre-eds-build" "$SRC" \
      --prefix="$STAGE" --libdir=lib --buildtype=release -Dlibclepsydre-eds=true >/dev/null
    meson install --quiet -C "$WORK/clepsydre-eds-build" >/dev/null
  fi
  touch "$STAMP"
fi

# --- Era ---------------------------------------------------------------------
echo "==> building era ($BACKEND backend)"
# What meson would generate. All four are gitignored.
sed -e "s|@APP_ID@|\"$APP_ID\"|" -e 's|@APP_NAME@|"Era"|' \
    -e 's|@BASE_RESOURCE_PATH@|"/org/gnome/gitlab/TitouanReal/Era/"|' \
    -e 's|@GETTEXT_PACKAGE@|"era"|' -e "s|@LOCALEDIR@|\"$SHARE/locale\"|" \
    -e 's|@PROJECT_NAME@|"era"|' \
    -e "s|@VERSION@|\"$(sed -n "s/^version = \"\(.*\)\"/\1/p" Cargo.toml | head -1)\"|" \
    src/config.rs.in > src/config.rs
sed -e 's|@GRESOURCE_PATH@|"/org/gnome/gitlab/TitouanReal/Era/"|' \
    -e "s|@GRESOURCE_DIR@|\"$PWD/data/resources\"|" \
    src/resources.rs.in > src/resources.rs
"$GRASS" data/resources/style.scss data/resources/style.css
blueprint-compiler compile data/resources/ui/shortcuts_dialog.blp \
  --output data/resources/shortcuts-dialog.ui

RUSTFLAGS="${RUSTFLAGS:-} -C link-arg=-Wl,-rpath,$LIBDIR" \
  cargo build --release --no-default-features --features "backend-$BACKEND,platform-flatpak"

# --- Install -----------------------------------------------------------------
echo "==> installing to $PREFIX"
install -Dm755 target/release/era "$BIN/era"
install -d "$LIBDIR"
cp -P "$STAGE"/lib/libclepsydre*.so* "$LIBDIR/"

SCHEMAS="$SHARE/glib-2.0/schemas"
install -d "$SCHEMAS"
sed -e "s|@app-id@|$APP_ID|" -e 's|@gettext-package@|era|' \
    -e 's|@resource-path@|/org/gnome/gitlab/TitouanReal/Era/|' \
    data/$BASE_ID.gschema.xml.in > "$SCHEMAS/$APP_ID.gschema.xml"
glib-compile-schemas "$SCHEMAS"

# The desktop entry is D-Bus activatable, so it needs a service file, and
# its name must match the app ID. Launchers often lack ~/.local/bin on PATH,
# so both carry the absolute binary path.
install -d "$SHARE/applications" "$SHARE/dbus-1/services"
sed -e 's|@app-name@|Era|' -e "s|@project-name@|$BIN/era|" -e "s|@icon@|$APP_ID|" \
    data/$BASE_ID.desktop.in.in > "$SHARE/applications/$APP_ID.desktop"
sed -e "s|@app-id@|$APP_ID|" -e "s|@bindir@|$BIN|" -e 's|@project-name@|era|' \
    data/$BASE_ID.service.in > "$SHARE/dbus-1/services/$APP_ID.service"
# dbus-broker only notices a services directory that existed when the bus
# started, so a first install isn't activatable until the next login unless
# the bus re-reads its config.
[[ -n ${DBUS_SESSION_BUS_ADDRESS:-} ]] && dbus-send --session --dest=org.freedesktop.DBus \
  --type=method_call / org.freedesktop.DBus.ReloadConfig >/dev/null 2>&1 || true

sed -e "s|@app-id@|$APP_ID|g" -e 's|@gettext-package@|era|g' \
  data/$BASE_ID.metainfo.xml.in.in > "$WORK/$APP_ID.metainfo.xml"
install -Dm644 "$WORK/$APP_ID.metainfo.xml" "$SHARE/metainfo/$APP_ID.metainfo.xml"

install -Dm644 data/icons/$APP_ID.svg "$SHARE/icons/hicolor/scalable/apps/$APP_ID.svg"
install -Dm644 data/icons/$BASE_ID-symbolic.svg \
  "$SHARE/icons/hicolor/symbolic/apps/$APP_ID-symbolic.svg"

if have_cmd msgfmt; then
  while read -r lang; do
    [[ -z $lang || $lang == \#* ]] && continue
    install -d "$SHARE/locale/$lang/LC_MESSAGES"
    msgfmt -o "$SHARE/locale/$lang/LC_MESSAGES/era.mo" "po/$lang.po"
  done < po/LINGUAS
fi

gtk4-update-icon-cache -q -t -f "$SHARE/icons/hicolor" 2>/dev/null ||
  gtk-update-icon-cache -q -t -f "$SHARE/icons/hicolor" 2>/dev/null || true
update-desktop-database -q "$SHARE/applications" 2>/dev/null || true
echo "Installed to $PREFIX. Run: $BIN/era"
