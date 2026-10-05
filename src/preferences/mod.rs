//! User preferences: the view Era opens on, and whether weeks start on
//! Sunday.
//!
//! Fork-only: upstream Era has no preferences. Everything lives in this
//! directory; elsewhere there are only the one-line hooks listed in FORK.md.
//! The values are GSettings keys. Where the schema isn't installed (an
//! uninstalled `cargo run`), the defaults apply and the panel is insensitive.

mod panel;

use std::cell::{Cell, OnceCell};

use adw::prelude::*;
use glib::clone;

pub use self::panel::PreferencesPanel;

use crate::{application::Application, config::APP_ID, utils::WeekDay};

pub const DEFAULT_VIEW: &str = "default-view";
pub const WEEK_STARTS_ON_SUNDAY: &str = "week-starts-on-sunday";

/// The values `default-view` can take, in the order the panel lists them.
pub const VIEWS: [&str; 3] = ["year", "month", "agenda"];

thread_local! {
    static SETTINGS: OnceCell<Option<gio::Settings>> = const { OnceCell::new() };
    /// The first day of the week as the desktop reports it, before the
    /// Sunday preference is applied.
    static DESKTOP_FIRST_WEEK_DAY: Cell<WeekDay> = Cell::new(WeekDay::default());
}

/// The app's settings, or `None` when the schema (or one of our keys) isn't
/// installed. GSettings aborts on a missing schema, so look first.
pub fn settings() -> Option<gio::Settings> {
    SETTINGS.with(|settings| {
        settings
            .get_or_init(|| {
                gio::SettingsSchemaSource::default()
                    .and_then(|source| source.lookup(APP_ID, true))
                    .filter(|schema| {
                        schema.has_key(DEFAULT_VIEW) && schema.has_key(WEEK_STARTS_ON_SUNDAY)
                    })
                    .map(|_| gio::Settings::new(APP_ID))
            })
            .clone()
    })
}

/// Name of the view to open on: "year", "month" or "agenda".
pub fn default_view() -> String {
    settings().map_or_else(
        || "month".to_owned(),
        |settings| settings.string(DEFAULT_VIEW).into(),
    )
}

/// Hook for `System::set_first_week_day`: remembers the desktop's value and
/// returns the one to use.
pub fn first_week_day(desktop: WeekDay) -> WeekDay {
    DESKTOP_FIRST_WEEK_DAY.set(desktop);

    if settings().is_some_and(|settings| settings.boolean(WEEK_STARTS_ON_SUNDAY)) {
        WeekDay::Sunday
    } else {
        desktop
    }
}

/// Call once from the application's `startup`, before any window exists.
pub fn install(app: &Application) {
    PreferencesPanel::ensure_type();

    let system = app.system();

    if let Some(settings) = settings() {
        settings.connect_changed(
            Some(WEEK_STARTS_ON_SUNDAY),
            clone!(
                #[weak]
                system,
                move |_, _| {
                    system.set_first_week_day(DESKTOP_FIRST_WEEK_DAY.get());
                }
            ),
        );
    }

    // Apply the preference to the value the system starts with; the desktop's
    // own value, read asynchronously, goes through the same hook.
    system.set_first_week_day(system.first_week_day());
}
