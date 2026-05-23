use std::{cell::Cell, sync::Arc};

use ashpd::{desktop::settings::Settings, zvariant};
use futures_util::StreamExt;
use glib::clone;
use gtk::{glib, prelude::*, subclass::prelude::*};
use tracing::error;

use crate::spawn;

const GNOME_DESKTOP_INTERFACE_NAMESPACE: &str = "org.gnome.desktop.interface";
const GNOME_DESKTOP_CALENDAR_NAMESPACE: &str = "org.gnome.desktop.calendar";
const CLOCK_FORMAT_KEY: &str = "clock-format";
const WEEK_START_DAY_KEY: &str = "week-start-day";

/// The clock format setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, glib::Enum)]
#[enum_type(name = "ClockFormat")]
pub enum ClockFormat {
    /// The 12h format, i.e. AM/PM.
    TwelveHours,
    /// The 24h format.
    TwentyFourHours,
}

impl Default for ClockFormat {
    fn default() -> Self {
        // Use the locale's default clock format as a fallback.
        let local_formatted_time = glib::DateTime::now_local()
            .and_then(|d| d.format("%X"))
            .map(|s| s.to_ascii_lowercase());
        match &local_formatted_time {
            Ok(s) if s.ends_with("am") || s.ends_with("pm") => ClockFormat::TwelveHours,
            Ok(_) => ClockFormat::TwentyFourHours,
            Err(error) => {
                error!("Could not get local formatted time: {error}");
                ClockFormat::TwelveHours
            }
        }
    }
}

impl TryFrom<&zvariant::OwnedValue> for ClockFormat {
    type Error = zvariant::Error;

    fn try_from(value: &zvariant::OwnedValue) -> Result<Self, Self::Error> {
        let Ok(s) = <&str>::try_from(value) else {
            return Err(zvariant::Error::IncorrectType);
        };

        match s {
            "12h" => Ok(Self::TwelveHours),
            "24h" => Ok(Self::TwentyFourHours),
            _ => Err(zvariant::Error::Message(format!(
                "Invalid string `{s}`, expected `12h` or `24h`"
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

/// The first day of the week setting.
#[derive(Debug, Default, Hash, Eq, PartialEq, Clone, Copy, glib::Enum)]
#[enum_type(name = "DayOfWeek")]
#[repr(i32)]
pub enum DayOfWeek {
    #[default]
    #[enum_value(name = "Monday", nick = "monday")]
    Monday,
    #[enum_value(name = "Tuesday", nick = "tuesday")]
    Tuesday,
    #[enum_value(name = "Wednesday", nick = "wednesday")]
    Wednesday,
    #[enum_value(name = "Thursday", nick = "thursday")]
    Thursday,
    #[enum_value(name = "Friday", nick = "friday")]
    Friday,
    #[enum_value(name = "Saturday", nick = "saturday")]
    Saturday,
    #[enum_value(name = "Sunday", nick = "sunday")]
    Sunday,
}

impl TryFrom<&zvariant::OwnedValue> for DayOfWeek {
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
                "Invalid string `{s}`, expected a day of the week"
            ))),
        }
    }
}

impl TryFrom<zvariant::OwnedValue> for DayOfWeek {
    type Error = zvariant::Error;

    fn try_from(value: zvariant::OwnedValue) -> Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}

mod imp {
    use super::*;

    #[derive(Debug, Default, glib::Properties)]
    #[properties(wrapper_type = super::SystemSettings)]
    pub struct SystemSettings {
        /// The clock format setting.
        #[property(get, builder(ClockFormat::default()))]
        pub(super) clock_format: Cell<ClockFormat>,
        /// The first day of the week setting.
        #[property(get, builder(DayOfWeek::default()))]
        pub(super) first_day_of_week: Cell<DayOfWeek>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SystemSettings {
        const NAME: &'static str = "SystemSettings";
        type Type = super::SystemSettings;
    }

    #[glib::derived_properties]
    impl ObjectImpl for SystemSettings {
        fn constructed(&self) {
            self.parent_constructed();

            spawn!(clone!(
                #[weak(rename_to = imp)]
                self,
                async move {
                    imp.init().await;
                }
            ));
        }
    }

    impl SystemSettings {
        /// Initialize the system settings.
        async fn init(&self) {
            let obj = self.obj();

            let proxy = match Settings::new().await {
                Ok(proxy) => proxy,
                Err(error) => {
                    error!("Could not access settings portal: {error}");
                    return;
                }
            };
            let proxy = Arc::new(proxy);

            // Read the initial clock format value.
            let proxy_clone = proxy.clone();
            match proxy_clone
                .read::<ClockFormat>(GNOME_DESKTOP_INTERFACE_NAMESPACE, CLOCK_FORMAT_KEY)
                .await
            {
                Ok(clock_format) => obj.set_clock_format(clock_format),
                Err(error) => {
                    error!("Could not access clock format system setting: {error}");
                }
            }

            // Read the initial first day of the week value.
            let proxy_clone = proxy.clone();
            match proxy_clone
                .read::<DayOfWeek>(GNOME_DESKTOP_CALENDAR_NAMESPACE, WEEK_START_DAY_KEY)
                .await
            {
                Ok(first_day_of_week) => obj.set_first_day_of_week(first_day_of_week),
                Err(error) => {
                    error!("Could not access first day of week system setting: {error}");
                }
            }

            // Listen to setting changes.
            let setting_changed_stream = match proxy.receive_setting_changed().await {
                Ok(stream) => stream,
                Err(error) => {
                    error!("Could not listen to changes of system settings: {error}");
                    return;
                }
            };

            let obj_weak = obj.downgrade();
            setting_changed_stream
                .for_each(move |setting| {
                    let obj_weak = obj_weak.clone();
                    async move {
                        let Some(obj) = obj_weak.upgrade() else {
                            error!(
                                "Could not update system setting: could not upgrade weak reference"
                            );
                            return;
                        };

                        let namespace = setting.namespace();
                        let key = setting.key();

                        if namespace == GNOME_DESKTOP_INTERFACE_NAMESPACE && key == CLOCK_FORMAT_KEY
                        {
                            match ClockFormat::try_from(setting.value()) {
                                Ok(clock_format) => obj.set_clock_format(clock_format),
                                Err(error) => {
                                    error!("Could not update clock format setting: {error}");
                                }
                            }
                        } else if namespace == GNOME_DESKTOP_CALENDAR_NAMESPACE
                            && key == WEEK_START_DAY_KEY
                        {
                            match DayOfWeek::try_from(setting.value()) {
                                Ok(first_day_of_week) => {
                                    obj.set_first_day_of_week(first_day_of_week)
                                }
                                Err(error) => {
                                    error!("Could not update first day of week setting: {error}");
                                }
                            }
                        }
                    }
                })
                .await;
        }
    }
}

glib::wrapper! {
    /// A sublassable API to access system settings.
    pub struct SystemSettings(ObjectSubclass<imp::SystemSettings>);
}

impl SystemSettings {
    pub fn new() -> Self {
        glib::Object::new()
    }

    /// Set the clock format setting.
    fn set_clock_format(&self, clock_format: ClockFormat) {
        if self.clock_format() == clock_format {
            return;
        }

        self.imp().clock_format.set(clock_format);
        self.notify_clock_format();
    }

    /// Set the first day of the week setting.
    fn set_first_day_of_week(&self, first_day_of_week: DayOfWeek) {
        if self.first_day_of_week() == first_day_of_week {
            return;
        }

        self.imp().first_day_of_week.set(first_day_of_week);
        self.notify_first_day_of_week();
    }
}

impl Default for SystemSettings {
    fn default() -> Self {
        Self::new()
    }
}

/// Public trait that must be implemented for everything that derives from
/// `SystemSettings`.
pub trait SystemSettingsImpl: ObjectImpl {}

unsafe impl<T> IsSubclassable<T> for SystemSettings
where
    T: SystemSettingsImpl,
    T::Type: IsA<SystemSettings>,
{
}
