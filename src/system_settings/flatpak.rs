use std::sync::Arc;

use ashpd::{desktop::settings::Settings, zvariant};
use futures_util::StreamExt;
use glib::clone;
use gtk::{glib, prelude::*, subclass::prelude::*};
use tracing::error;

use super::{ClockFormat, DayOfWeek, SystemSettings, SystemSettingsImpl};
use crate::spawn;

const GNOME_DESKTOP_INTERFACE_NAMESPACE: &str = "org.gnome.desktop.interface";
const GNOME_DESKTOP_CALENDAR_NAMESPACE: &str = "org.gnome.desktop.calendar";
const CLOCK_FORMAT_KEY: &str = "clock-format";
const WEEK_START_DAY_KEY: &str = "week-start-day";

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

    #[derive(Debug, Default)]
    pub struct FlatpakSystemSettings {}

    #[glib::object_subclass]
    impl ObjectSubclass for FlatpakSystemSettings {
        const NAME: &'static str = "FlatpakSystemSettings";
        type Type = super::FlatpakSystemSettings;
        type ParentType = SystemSettings;
    }

    impl ObjectImpl for FlatpakSystemSettings {
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

    impl SystemSettingsImpl for FlatpakSystemSettings {}

    impl FlatpakSystemSettings {
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
                Ok(clock_format) => obj
                    .upcast_ref::<SystemSettings>()
                    .set_clock_format(clock_format),
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
                Ok(first_day_of_week) => obj
                    .upcast_ref::<SystemSettings>()
                    .set_first_day_of_week(first_day_of_week),
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
                        let obj = obj.upcast_ref::<SystemSettings>();

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
    /// API to access system settings on Flatpak, via the XDG Desktop Settings portal.
    pub struct FlatpakSystemSettings(ObjectSubclass<imp::FlatpakSystemSettings>)
        @extends SystemSettings;
}

impl FlatpakSystemSettings {
    pub fn new() -> Self {
        glib::Object::new()
    }
}

impl Default for FlatpakSystemSettings {
    fn default() -> Self {
        Self::new()
    }
}
