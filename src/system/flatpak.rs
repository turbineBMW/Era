use std::cell::OnceCell;

use ashpd::zvariant;
use futures_util::StreamExt;
use gio::DBusConnection;
use glib::{DateTime, TimeZone, clone};
use gtk::{prelude::*, subclass::prelude::*};
use tracing::error;

use crate::{spawn, utils::WeekDay};

use super::{ClockFormat, System, SystemImpl};

const GNOME_DESKTOP_INTERFACE_NAMESPACE: &str = "org.gnome.desktop.interface";
const GNOME_DESKTOP_CALENDAR_NAMESPACE: &str = "org.gnome.desktop.calendar";
const CLOCK_FORMAT_KEY: &str = "clock-format";
const WEEK_START_DAY_KEY: &str = "week-start-day";

/// Returns the first day of the week according to the current locale (`LC_TIME`).
fn locale_first_week_day() -> WeekDay {
    // These glibc items are not exposed by the `libc` crate. `_NL_ITEM(category, index)` is
    // `(category << 16) | index`.
    const NL_TIME_WEEK_1STDAY: libc::nl_item = ((libc::LC_TIME as libc::nl_item) << 16) | 102;
    const NL_TIME_FIRST_WEEKDAY: libc::nl_item = ((libc::LC_TIME as libc::nl_item) << 16) | 104;

    // SAFETY: `WEEK_1STDAY` returns a value stored in the pointer itself and `FIRST_WEEKDAY`
    // returns a pointer to a single byte. Neither is null for valid items and the pointers are
    // not retained.
    let (week_1st_day, first_weekday) = unsafe {
        (
            libc::nl_langinfo(NL_TIME_WEEK_1STDAY) as usize,
            *libc::nl_langinfo(NL_TIME_FIRST_WEEKDAY) as u8 as usize,
        )
    };

    // `WEEK_1STDAY` is a reference date (YYYYMMDD) that tells which day `FIRST_WEEKDAY == 1`
    // refers to. 1997-11-30 was a Sunday, 1997-12-01 was a Monday.
    let reference_day = match week_1st_day {
        19971130 => 0,
        19971201 => 1,
        _ => return WeekDay::Monday,
    };

    if !(1..=7).contains(&first_weekday) {
        return WeekDay::Monday;
    }

    match (reference_day + first_weekday - 1) % 7 {
        0 => WeekDay::Sunday,
        1 => WeekDay::Monday,
        2 => WeekDay::Tuesday,
        3 => WeekDay::Wednesday,
        4 => WeekDay::Thursday,
        5 => WeekDay::Friday,
        _ => WeekDay::Saturday,
    }
}

impl TryFrom<&str> for ClockFormat {
    type Error = zvariant::Error;

    fn try_from(string: &str) -> Result<Self, Self::Error> {
        match string {
            "12h" => Ok(Self::TwelveHours),
            "24h" => Ok(Self::TwentyFourHours),
            _ => Err(zvariant::Error::Message(format!(
                "Invalid string `{string}`, expected `12h` or `24h`"
            ))),
        }
    }
}

impl TryFrom<&zvariant::OwnedValue> for ClockFormat {
    type Error = zvariant::Error;

    fn try_from(value: &zvariant::OwnedValue) -> Result<Self, Self::Error> {
        let Ok(string) = <&str>::try_from(value) else {
            return Err(zvariant::Error::IncorrectType);
        };

        Self::try_from(string)
    }
}

impl TryFrom<zvariant::OwnedValue> for ClockFormat {
    type Error = zvariant::Error;

    fn try_from(value: zvariant::OwnedValue) -> Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}

impl TryFrom<&str> for WeekDay {
    type Error = zvariant::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "monday" => Ok(Self::Monday),
            "tuesday" => Ok(Self::Tuesday),
            "wednesday" => Ok(Self::Wednesday),
            "thursday" => Ok(Self::Thursday),
            "friday" => Ok(Self::Friday),
            "saturday" => Ok(Self::Saturday),
            "sunday" => Ok(Self::Sunday),
            "default" => Ok(locale_first_week_day()),
            _ => Err(zvariant::Error::Message(format!(
                "Invalid string `{s}`, expected a week day"
            ))),
        }
    }
}

impl TryFrom<&zvariant::OwnedValue> for WeekDay {
    type Error = zvariant::Error;

    fn try_from(value: &zvariant::OwnedValue) -> Result<Self, Self::Error> {
        let Ok(s) = <&str>::try_from(value) else {
            return Err(zvariant::Error::IncorrectType);
        };

        Self::try_from(s)
    }
}

impl TryFrom<zvariant::OwnedValue> for WeekDay {
    type Error = zvariant::Error;

    fn try_from(value: zvariant::OwnedValue) -> Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct FlatpakSystem {
        system_bus: OnceCell<DBusConnection>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FlatpakSystem {
        const NAME: &'static str = "FlatpakSystem";
        type Type = super::FlatpakSystem;
        type ParentType = System;
    }

    impl ObjectImpl for FlatpakSystem {
        fn constructed(&self) {
            self.parent_constructed();

            let system_bus = gio::bus_get_sync(gio::BusType::System, gio::Cancellable::NONE)
                .expect("Failed to connect to system D-Bus");
            self.system_bus
                .set(system_bus)
                .expect("System bus should not already be initialized");

            self.update_datetime();

            // The settings are read by blocking on the portal proxy, so that the UI is built with
            // correct values directly.
            let main_context = glib::MainContext::default();
            let proxy = main_context
                .block_on(ashpd::desktop::settings::Settings::new())
                .expect("Could not connect to the settings portal");
            self.read_initial_settings(&main_context, &proxy);

            glib::timeout_add_seconds_local(
                1,
                clone!(
                    #[weak(rename_to = imp)]
                    self,
                    #[upgrade_or]
                    glib::ControlFlow::Break,
                    move || {
                        imp.update_datetime();
                        glib::ControlFlow::Continue
                    }
                ),
            );

            spawn!(clone!(
                #[weak(rename_to = imp)]
                self,
                async move {
                    let system = imp.obj().clone().upcast::<System>();

                    proxy
                        .receive_setting_changed()
                        .await
                        .unwrap()
                        .for_each(move |setting| {
                            let system_weak = system.downgrade();
                            async move {
                                let Some(system) = system_weak.upgrade() else {
                                    error!(
                                    "Could not update system setting: could not upgrade weak reference"
                                );
                                return;
                            };

                            let namespace = setting.namespace();
                            let key = setting.key();

                            if namespace == GNOME_DESKTOP_INTERFACE_NAMESPACE
                                && key == CLOCK_FORMAT_KEY
                            {
                                match ClockFormat::try_from(setting.value()) {
                                    Ok(clock_format) => system.set_clock_format(clock_format),
                                    Err(error) => {
                                        error!("Could not update clock format setting: {error}");
                                    }
                                }
                            } else if namespace == GNOME_DESKTOP_CALENDAR_NAMESPACE
                                && key == WEEK_START_DAY_KEY
                            {
                                match WeekDay::try_from(setting.value()) {
                                    Ok(first_week_day) => system.set_first_week_day(first_week_day),
                                    Err(error) => {
                                        error!(
                                            "Could not update first day of week setting: {error}"
                                        );
                                    }
                                }
                            }
                            }
                        })
                        .await;
                }
            ));
        }
    }

    impl SystemImpl for FlatpakSystem {}

    impl FlatpakSystem {
        /// Reads the settings from the portal, blocking on the main context, so that they are
        /// correct as soon as the object is constructed. Later changes are handled by the portal
        /// subscription.
        fn read_initial_settings(
            &self,
            main_context: &glib::MainContext,
            proxy: &ashpd::desktop::settings::Settings,
        ) {
            let system = self.obj().clone().upcast::<System>();

            match main_context.block_on(
                proxy.read::<ClockFormat>(GNOME_DESKTOP_INTERFACE_NAMESPACE, CLOCK_FORMAT_KEY),
            ) {
                Ok(clock_format) => system.set_clock_format(clock_format),
                Err(error) => {
                    error!("Could not access clock format system setting: {error}");
                }
            }

            match main_context.block_on(
                proxy.read::<WeekDay>(GNOME_DESKTOP_CALENDAR_NAMESPACE, WEEK_START_DAY_KEY),
            ) {
                Ok(first_week_day) => system.set_first_week_day(first_week_day),
                Err(error) => {
                    error!("Could not access first day of week system setting: {error}");
                    system.set_first_week_day(locale_first_week_day());
                }
            }
        }

        /// Reads the system timezone from org.freedesktop.timedate1.
        fn read_system_timezone(&self) -> TimeZone {
            let connection = self
                .system_bus
                .get()
                .expect("System bus should be initialized");

            let result = connection.call_sync(
                Some("org.freedesktop.timedate1"),
                "/org/freedesktop/timedate1",
                "org.freedesktop.DBus.Properties",
                "Get",
                Some(&glib::Variant::from((
                    "org.freedesktop.timedate1",
                    "Timezone",
                ))),
                Some(glib::VariantTy::new("(v)").expect("Variant Type should be valid")),
                gio::DBusCallFlags::NONE,
                -1,
                gio::Cancellable::NONE,
            );

            let iana_name = result
                .expect("Failed to read Timezone from timedate1")
                .child_value(0)
                .as_variant()
                .expect("Variant should contain another variant")
                .get::<String>()
                .expect("Variant should contain a string");

            TimeZone::from_identifier(Some(&iana_name)).expect("TimeZone should exist")
        }

        /// Returns the current time in the system timezone.
        fn now(&self) -> DateTime {
            let tz = self.read_system_timezone();
            DateTime::now(&tz).expect("Now should exist in any timezone")
        }

        /// Updates the `current-datetime` property from the system clock.
        fn update_datetime(&self) {
            let new_dt = self.now();

            let old = self.obj().upcast_ref::<System>().datetime();
            let changed = old.year() != new_dt.year()
                || old.month() != new_dt.month()
                || old.day_of_month() != new_dt.day_of_month()
                || old.hour() != new_dt.hour()
                || old.minute() != new_dt.minute();

            if changed {
                self.obj().upcast_ref::<System>().set_datetime(new_dt);
            }
        }
    }
}

glib::wrapper! {
    /// API to access system state on Flatpak, via the XDG Desktop Settings portal.
    pub struct FlatpakSystem(ObjectSubclass<imp::FlatpakSystem>)
        @extends System;
}

impl FlatpakSystem {
    pub fn new() -> Self {
        glib::Object::new()
    }
}

impl Default for FlatpakSystem {
    fn default() -> Self {
        Self::new()
    }
}
