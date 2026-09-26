//! Follow the Omarchy desktop theme, live.
//!
//! Fork-only: upstream Era has none of this. Everything lives in this
//! directory so rebasing onto upstream only has to carry a few one-line
//! hooks elsewhere (see FORK.md).
//!
//! The palette is resolved in [`palette`]; this is the half that touches
//! GTK. The theme's CSS goes into a provider stacked above the app
//! stylesheet and the calendar colours, and libadwaita is forced light or
//! dark to match the theme's mode. The `app.follow-omarchy-theme` action
//! turns it off: the provider is emptied and the colour scheme handed back
//! to the system, so nothing here is sticky.

mod palette;

use std::{
    cell::{Cell, OnceCell},
    rc::Rc,
    time::Duration,
};

use adw::prelude::*;
use glib::clone;
use tracing::{debug, warn};

use crate::config::APP_ID;

/// One theme switch is a burst of file events; reload once it settles.
const SETTLE: Duration = Duration::from_millis(120);

/// The action and GSettings key that turn theme following on and off.
const FOLLOW: &str = "follow-omarchy-theme";

thread_local! {
    /// Owns the provider and the file monitor for the life of the process.
    static THEME: OnceCell<Rc<Inner>> = const { OnceCell::new() };
}

struct Inner {
    provider: gtk::CssProvider,
    /// Stateful boolean; its state is the single source of truth.
    action: gio::Action,
    /// Kept alive so the action stays bound to the key.
    _settings: Option<gio::Settings>,
    monitor: OnceCell<gio::FileMonitor>,
}

/// Start following the theme. Call once, from the application's `startup`,
/// when the display exists. Does nothing where Omarchy isn't installed --
/// not even adding the action, so its menu item stays hidden.
pub fn install(app: &gio::Application) {
    if !palette::detected(&glib::home_dir()) {
        return;
    }
    let Some(display) = gdk::Display::default() else {
        return;
    };

    let provider = gtk::CssProvider::new();
    provider.connect_parsing_error(|_, section, error| {
        warn!(
            "omarchy theme css line {}: {error}",
            section.start_location().lines() + 1
        );
    });
    // Above the app stylesheet (APPLICATION) and the window's calendar
    // colours (APPLICATION + 1). The theme only sets libadwaita's :root
    // variables, so calendar colours still win where they apply.
    gtk::style_context_add_provider_for_display(
        &display,
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION + 2,
    );

    // Persist the choice when the schema is installed; an uninstalled
    // `cargo run` has none, and GSettings aborts on a missing schema.
    let settings = gio::SettingsSchemaSource::default()
        .and_then(|source| source.lookup(APP_ID, true))
        .filter(|schema| schema.has_key(FOLLOW))
        .map(|_| gio::Settings::new(APP_ID));
    let action: gio::Action = match &settings {
        Some(settings) => settings.create_action(FOLLOW),
        None => gio::SimpleAction::new_stateful(FOLLOW, None, &true.to_variant()).upcast(),
    };
    app.add_action(&action);

    let inner = Rc::new(Inner {
        provider,
        action,
        _settings: settings,
        monitor: OnceCell::new(),
    });
    inner.action.connect_notify_local(
        Some("state"),
        clone!(
            #[weak]
            inner,
            move |_, _| inner.reload()
        ),
    );
    inner.watch();
    inner.reload();
    THEME.with(|theme| {
        let _ = theme.set(inner);
    });
}

impl Inner {
    fn following(&self) -> bool {
        self.action
            .state()
            .and_then(|state| state.get::<bool>())
            .unwrap_or(true)
    }

    /// Re-read the active theme into the provider. A theme that is missing,
    /// unreadable, or not being followed leaves the provider empty.
    fn reload(&self) {
        let theme = self
            .following()
            .then(|| palette::load(&glib::home_dir()))
            .flatten();
        self.provider
            .load_from_string(theme.as_ref().map_or("", |theme| theme.css.as_str()));
        if theme.is_some() {
            debug!(
                "following omarchy theme {}",
                palette::theme_name(&glib::home_dir()).unwrap_or_default()
            );
        }
        adw::StyleManager::default().set_color_scheme(match &theme {
            Some(theme) if theme.light => adw::ColorScheme::ForceLight,
            Some(_) => adw::ColorScheme::ForceDark,
            None => adw::ColorScheme::Default,
        });
    }

    /// Recolour live when the desktop theme changes. `omarchy theme set`
    /// renames a staging directory over `current/theme` and rewrites
    /// `current/theme.name`, so the watch is on the stable parent -- a monitor
    /// on a file inside the theme would not survive the directory swap.
    fn watch(self: &Rc<Self>) {
        let dir = palette::state_dir(&glib::home_dir());
        let monitor = match gio::File::for_path(&dir)
            .monitor_directory(gio::FileMonitorFlags::WATCH_MOVES, gio::Cancellable::NONE)
        {
            Ok(monitor) => monitor,
            Err(error) => {
                warn!("cannot watch {}: {error}", dir.display());
                return;
            }
        };
        let pending = Rc::new(Cell::new(false));
        monitor.connect_changed(clone!(
            #[weak(rename_to = inner)]
            self,
            move |_, _, _, _| {
                if pending.replace(true) {
                    return;
                }
                let pending = pending.clone();
                glib::timeout_add_local_once(
                    SETTLE,
                    clone!(
                        #[weak]
                        inner,
                        move || {
                            pending.set(false);
                            inner.reload();
                        }
                    ),
                );
            }
        ));
        let _ = self.monitor.set(monitor);
    }
}
