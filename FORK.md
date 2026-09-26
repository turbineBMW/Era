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
- `scripts/sync-upstream.sh`, `.github/workflows/upstream-sync.yml`, this file

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

## Building outside Flatpak

Upstream builds with Flatpak or meson. On Arch in September 2026, meson 1.12's
Cargo-subproject support failed on newer `serde` (it doesn't run build
scripts). Cargo works once the `libclepsydre` C library is available:

1. `cargo build -p clepsydre --release` inside `subprojects/clepsydre` (meson
   downloads it on first setup). Install `libclepsydre.so` as
   `libclepsydre-0.so.0`, with `libclepsydre-0.so` and `libclepsydre.so`
   symlinks, plus `clepsydre/include/clepsydre.h` and a `clepsydre-0.pc`.
2. Generate the `Clepsydre-0` typelib with `g-ir-scanner` / `g-ir-compiler`
   (package `gobject-introspection`) so the Blueprint templates compile.
   Point `GI_TYPELIB_PATH` at it.
3. Run `meson setup` once to generate `src/config.rs` and `src/resources.rs`,
   then:

   ```sh
   cargo build --no-default-features --features backend-mock,platform-flatpak   # demo data
   cargo test  --no-default-features --features backend-mock,platform-flatpak omarchy
   ```

Real calendars need the default `backend-eds` feature, which also needs
`evolution-data-server` and `libclepsydre-eds`.
