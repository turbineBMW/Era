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

impl TryFrom<&zvariant::OwnedValue> for ClockFormat {
    type Error = zvariant::Error;

    fn try_from(value: &zvariant::OwnedValue) -> Result<Self, Self::Error> {
        let Ok(string) = <&str>::try_from(value) else {
            return Err(zvariant::Error::IncorrectType);
        };

        match string {
            "12h" => Ok(Self::TwelveHours),
            "24h" => Ok(Self::TwentyFourHours),
            _ => Err(zvariant::Error::Message(format!(
                "Invalid string `{string}`, expected `12h` or `24h`"
            ))),
        }
    }
}

impl TryFrom<zvariant::OwnedValue> for ClockFormat {
    type Error = zvariant::Error;

    fn try_from(value: zvariant::OwnedValue) -> Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}

impl TryFrom<&zvariant::OwnedValue> for WeekDay {
    type Error = zvariant::Error;

    fn try_from(value: &zvariant::OwnedValue) -> Result<Self, Self::Error> {
        let Ok(s) = <&str>::try_from(value) else {
            return Err(zvariant::Error::IncorrectType);
        };

        match s {
            "monday" => Ok(Self::Monday),
            "tuesday" => Ok(Self::Tuesday),
            "wednesday" => Ok(Self::Wednesday),
            "thursday" => Ok(Self::Thursday),
            "friday" => Ok(Self::Friday),
            "saturday" => Ok(Self::Saturday),
            "sunday" => Ok(Self::Sunday),
            // TODO: Retrieve from locale
            "default" => Ok(Self::Monday),
            _ => Err(zvariant::Error::Message(format!(
                "Invalid string `{s}`, expected a week day"
            ))),
        }
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
        // fork: none in a sandbox without the system bus (omarchy-mobile's)
        system_bus: OnceCell<Option<DBusConnection>>,
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

            // fork: without a system bus, the time zone is the local one
            let system_bus = gio::bus_get_sync(gio::BusType::System, gio::Cancellable::NONE).ok();
            self.system_bus
                .set(system_bus)
                .expect("System bus should not already be initialized");

            self.update_datetime();
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

                    let proxy = ashpd::desktop::settings::Settings::new().await.unwrap();

                    match proxy
                        .read::<ClockFormat>(GNOME_DESKTOP_INTERFACE_NAMESPACE, CLOCK_FORMAT_KEY)
                        .await
                    {
                        Ok(clock_format) => system.set_clock_format(clock_format),
                        Err(error) => {
                            error!("Could not access clock format system setting: {error}");
                        }
                    }

                    match proxy
                        .read::<WeekDay>(GNOME_DESKTOP_CALENDAR_NAMESPACE, WEEK_START_DAY_KEY)
                        .await
                    {
                        Ok(first_week_day) => system.set_first_week_day(first_week_day),
                        Err(error) => {
                            error!("Could not access first day of week system setting: {error}");
                        }
                    }

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
        /// Reads the system timezone from org.freedesktop.timedate1.
        fn read_system_timezone(&self) -> TimeZone {
            // fork: the local time zone (/etc/localtime, which timedate1
            // reports too) when the system bus or timedate1 isn't there
            let Some(connection) = self
                .system_bus
                .get()
                .expect("System bus should be initialized")
            else {
                return TimeZone::local();
            };

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

            let Ok(result) = result else {
                return TimeZone::local();
            };
            let iana_name = result
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
