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
    pub struct SystemClass {
        parent_class: glib::object::Class<glib::Object>,
    }

    unsafe impl ClassStruct for SystemClass {
        type Type = System;
    }

    #[derive(Debug, Default, glib::Properties)]
    #[properties(wrapper_type = super::System)]
    pub struct System {
        /// The clock format setting.
        #[property(get, builder(ClockFormat::default()))]
        pub(super) clock_format: Cell<ClockFormat>,
        /// The first day of the week setting.
        #[property(get, builder(DayOfWeek::default()))]
        pub(super) first_day_of_week: Cell<DayOfWeek>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for System {
        const NAME: &'static str = "EraSystem";
        type Type = super::System;
        type Class = SystemClass;
    }

    #[glib::derived_properties]
    impl ObjectImpl for System {}
}

glib::wrapper! {
    /// A sublassable API to access system state.
    pub struct System(ObjectSubclass<imp::System>);
}

impl System {
    pub fn new() -> Self {
        #[cfg(feature = "platform-flatpak")]
        let obj = flatpak::FlatpakSystem::new().upcast();

        #[cfg(feature = "platform-android")]
        let obj = android::AndroidSystem::new().upcast();

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

impl Default for System {
    fn default() -> Self {
        Self::new()
    }
}

/// Public trait that must be implemented for everything that derives from `System`.
pub trait SystemImpl: ObjectImpl {}

unsafe impl<T> IsSubclassable<T> for System
where
    T: SystemImpl,
    T::Type: IsA<System>,
{
}
