/// A day of the week.
#[derive(Debug, Default, Hash, Eq, PartialEq, Clone, Copy, glib::Enum)]
#[enum_type(name = "EraWeekDay")]
#[repr(i32)]
pub enum WeekDay {
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
