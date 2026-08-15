use std::cell::Cell;

use gtk::{prelude::*, subclass::prelude::*};

#[cfg(feature = "platform-android")]
mod android;
#[cfg(feature = "platform-flatpak")]
mod flatpak;

/// The clock format setting.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, glib::Enum)]
#[enum_type(name = "ClockFormat")]
pub enum ClockFormat {
    /// The 12h format, i.e. AM/PM.
    TwelveHours,
    /// The 24h format.
    #[default]
    TwentyFourHours,
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

mod imp {
    use super::*;

    #[repr(C)]
    pub struct SystemSettingsClass {
        parent_class: glib::object::Class<glib::Object>,
    }

    unsafe impl ClassStruct for SystemSettingsClass {
        type Type = SystemSettings;
    }

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
        type Class = SystemSettingsClass;
    }

    #[glib::derived_properties]
    impl ObjectImpl for SystemSettings {}
}

glib::wrapper! {
    /// A sublassable API to access system settings.
    pub struct SystemSettings(ObjectSubclass<imp::SystemSettings>);
}

impl SystemSettings {
    pub fn new() -> Self {
        #[cfg(feature = "platform-flatpak")]
        let obj = flatpak::FlatpakSystemSettings::new().upcast();

        #[cfg(feature = "platform-android")]
        let obj = android::AndroidSystemSettings::new().upcast();

        obj
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
