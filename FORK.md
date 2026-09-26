# Era, with Omarchy theming

A fork of [Era](https://gitlab.gnome.org/TitouanReal/Era) that follows the
[Omarchy](https://omarchy.org/) desktop theme, the same way Vmux and Rustle do.
Era is in beta and moves fast, so the fork is kept as a small stack of commits
on top of upstream that can be rebased at any time.

## What the fork adds

Where an Omarchy theme is present, Era takes the window colours, accent and
light/dark style from `~/.local/state/omarchy/current/theme/colors.toml` and
follows every `omarchy theme set` live. The palette uses the same fallbacks as
`omarchy-theme-color`, so legacy `color0`..`color15` themes work too. Calendar
colours are left alone.

*Main menu → Follow Omarchy Theme* turns it off, which restores the system
style. The item is hidden on machines without Omarchy.

A theme can take full control by shipping an `era.css` (GTK CSS setting
libadwaita's variables), either in the theme directory or generated from
`~/.config/omarchy/themed/era.css.tpl`. That file replaces the derived palette.

## Branches

| Branch    | Contents                                                        |
|-----------|-----------------------------------------------------------------|
| `main`    | Exact mirror of upstream `main`. Never commit to it.            |
| `omarchy` | `main` + the fork's commits. Default branch; build from here.   |

Remotes: `origin` is this fork, `upstream` is gitlab.gnome.org/TitouanReal/Era.

## Pulling in upstream changes

```sh
scripts/sync-upstream.sh          # fetch, fast-forward main, rebase omarchy, test, push
scripts/sync-upstream.sh --no-push
```

If the rebase conflicts, resolve the files, `git add` them,
`git rebase --continue`, and run the script again. `git rerere` is switched on,
so a conflict you have resolved once resolves itself the next time. Before each
rebase the script saves the old branch as `omarchy-before-sync`.

The *Upstream sync* GitHub Action runs daily. It mirrors upstream into `main`
and keeps one issue labelled `upstream-sync` open while `omarchy` is behind,
saying whether the rebase would apply cleanly or which files conflict. It never
rewrites `omarchy` itself.

## Where the fork touches upstream files

Everything substantial is in new files, which cannot conflict:

- `src/omarchy/palette.rs`: colors.toml → libadwaita CSS, with unit tests
  (ported from Rustle's `rustle-core/src/omarchy.rs`; keep them in step)
- `src/omarchy/mod.rs`: CSS provider, light/dark forcing, file monitor, and the
  toggle action
- `install.sh`, `scripts/sync-upstream.sh`, `.github/workflows/upstream-sync.yml`, this file

Upstream files carry only these hooks, each marked `fork`. When one conflicts,
keep upstream's version and put the hook back:

| File | Hook |
|------|------|
| `src/main.rs` | `mod omarchy;` and `application.connect_startup(... omarchy::install ...)` right after `Application::new` |
| `data/org.gnome.gitlab.TitouanReal.Era.gschema.xml.in` | the `follow-omarchy-theme` key |
| `data/resources/ui/window.blp` | first section of `menu primary_menu`: the *Follow Omarchy Theme* item |
| `build-aux/org.gnome.gitlab.TitouanReal.Era.Eds.Devel.json` | `--filesystem=~/.local/state/omarchy:ro` in `finish-args` |

If upstream ever gets a preferences dialog, the toggle could move there, but
that would mean a larger hook.

## Installing without Flatpak

```sh
./install.sh                    # ~/.local; your calendars through evolution-data-server
BACKEND=mock ./install.sh       # demo data, no evolution-data-server needed
PREFIX=/usr sudo ./install.sh   # system-wide
```

The script names any missing packages (on Arch:
`sudo pacman -S --needed gobject-introspection evolution-data-server`). It
then builds Era's calendar libraries (clepsydre) at the commit `Cargo.lock`
pins, installs them to `$PREFIX/lib/era` with the binary's rpath pointing
there, and installs the desktop entry, D-Bus service, GSettings schema, icons
and translations. Build intermediates go in `target/native`. Run it again after
a sync to upgrade; clepsydre is rebuilt only when its pinned commit changes.

Upstream's meson build can't be used here: meson 1.12's Cargo-subproject
support doesn't run build scripts, which newer `serde` needs. So `install.sh`
builds clepsydre's Rust library with cargo, and only its C part
(`libclepsydre-eds`) with meson.

For development after one `install.sh` run:

```sh
export PKG_CONFIG_PATH=$PWD/target/native/stage/lib/pkgconfig
export GI_TYPELIB_PATH=$PWD/target/native/stage/lib/girepository-1.0
export LD_LIBRARY_PATH=$PWD/target/native/stage/lib
cargo test --no-default-features --features backend-mock,platform-flatpak omarchy
```
