use std::cell::{Cell, RefCell};

use glib::{DateTime, TimeZone};
use gtk::{prelude::*, subclass::prelude::*};

use crate::utils::{Date, WeekDay};

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

mod imp {
    use super::*;

    #[repr(C)]
    pub struct SystemClass {
        parent_class: glib::object::Class<glib::Object>,
    }

    unsafe impl ClassStruct for SystemClass {
        type Type = System;
    }

    #[derive(Debug, glib::Properties)]
    #[properties(wrapper_type = super::System)]
    pub struct System {
        #[property(get)]
        pub(super) datetime: RefCell<DateTime>,
        /// The clock format setting.
        #[property(get, builder(ClockFormat::default()))]
        pub(super) clock_format: Cell<ClockFormat>,
        /// The first day of the week setting.
        #[property(get, builder(WeekDay::default()))]
        pub(super) first_week_day: Cell<WeekDay>,
    }

    impl Default for System {
        fn default() -> Self {
            Self {
                datetime: RefCell::new(
                    DateTime::new(&TimeZone::utc(), 2000, 1, 1, 0, 0, 0.).unwrap(),
                ),
                clock_format: Cell::default(),
                first_week_day: Cell::default(),
            }
        }
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

    /// Sets the system time.
    fn set_datetime(&self, datetime: DateTime) {
        if self.datetime() == datetime {
            return;
        }

        self.imp().datetime.replace(datetime);
        self.notify_datetime();
    }

    /// Sets the clock format setting.
    fn set_clock_format(&self, clock_format: ClockFormat) {
        if self.clock_format() == clock_format {
            return;
        }

        self.imp().clock_format.set(clock_format);
        self.notify_clock_format();
    }

    /// Sets the first day of the week setting.
    pub(crate) fn set_first_week_day(&self, first_week_day: WeekDay) {
        let first_week_day = crate::preferences::first_week_day(first_week_day); // fork
        if self.first_week_day() == first_week_day {
            return;
        }

        self.imp().first_week_day.set(first_week_day);
        self.notify_first_week_day();
    }

    pub fn date(&self) -> Date {
        let datetime = self.imp().datetime.borrow();
        let year = datetime.year();
        let month = datetime.month();
        let day = datetime.day_of_month();
        jiff::civil::Date::new(year as i16, month as i8, day as i8)
            .unwrap()
            .into()
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
