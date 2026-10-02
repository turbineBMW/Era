//! Collection of template callbacks.

use clepsydre::{AttendeeRole, AttendeeType, ParticipationStatus};
use gettextrs::gettext;
use glib::Object;

use crate::utils::{Date, WeekDay};

/// Struct used as a collection of template callbacks.
pub struct TemplateCallbacks {}

#[gtk::template_callbacks(functions)]
impl TemplateCallbacks {
    /// Inverts the given boolean.
    #[template_callback]
    pub fn not(boolean: bool) -> bool {
        !boolean
    }

    /// Applies a logical "and".
    #[template_callback]
    pub fn both(left: bool, right: bool) -> bool {
        left && right
    }

    /// Applies a logical "or".
    #[template_callback]
    pub fn either(left: bool, right: bool) -> bool {
        left || right
    }

    /// Applies a logical "and(not, not)".
    #[template_callback]
    pub fn neither(left: bool, right: bool) -> bool {
        !left && !right
    }

    /// Applies a logical "and(not, not, not)".
    #[template_callback]
    pub fn none_of_three(left: bool, middle: bool, right: bool) -> bool {
        !left && !middle && !right
    }

    /// Applies a ternary operator.
    #[template_callback]
    pub fn ternary(condition: bool, left: &str, right: &str) -> String {
        if condition {
            left.to_string()
        } else {
            right.to_string()
        }
    }

    /// Returns `true` when the given string is empty or contains only whitespace.
    #[template_callback]
    pub fn trimmed_string_empty(string: &str) -> bool {
        string.trim().is_empty()
    }

    /// Returns `true` when the given strings are equal.
    #[template_callback]
    pub fn string_equals(left: &str, right: &str) -> bool {
        left == right
    }

    /// Returns `true` when the given option object is some.
    #[template_callback]
    pub fn is_some(option: Option<Object>) -> bool {
        option.is_some()
    }

    /// Returns `true` when the given option object is none.
    #[template_callback]
    pub fn is_none(option: Option<Object>) -> bool {
        option.is_none()
    }

    /// Returns `true` when the given number is zero.
    #[template_callback]
    pub fn is_zero(int: u32) -> bool {
        int == 0
    }

    /// Returns `true` when the given numbers are equal.
    #[template_callback]
    pub fn int_equals(left: i32, right: i32) -> bool {
        left == right
    }

    /// Returns `true` when the participation statuses are equals.
    #[template_callback]
    pub fn participation_status_equals(
        left: ParticipationStatus,
        right: ParticipationStatus,
    ) -> bool {
        left == right
    }

    /// Returns `true` when the attendee types are equals.
    #[template_callback]
    pub fn attendee_type_equals(left: AttendeeType, right: AttendeeType) -> bool {
        left == right
    }

    /// Returns `true` when the attendee roles are equals.
    #[template_callback]
    pub fn attendee_role_equals(left: AttendeeRole, right: AttendeeRole) -> bool {
        left == right
    }

    /// Returns the abbreviation of the day.
    #[template_callback]
    pub fn day_abbreviation(first_week_day: WeekDay, offset: i32) -> String {
        let base: i32 = first_week_day as i32;
        let day = (base + offset) % 7;
        match day {
            1 => gettext("MON"),
            2 => gettext("TUE"),
            3 => gettext("WED"),
            4 => gettext("THU"),
            5 => gettext("FRI"),
            6 => gettext("SAT"),
            0 => gettext("SUN"),
            _ => panic!("Invalid day number: {day}"),
        }
    }

    /// Returns the capitalized name of the month.
    #[template_callback]
    pub fn capitalized_month_name(month: i32) -> String {
        match month {
            1 => gettext("January"),
            2 => gettext("February"),
            3 => gettext("March"),
            4 => gettext("April"),
            5 => gettext("May"),
            6 => gettext("June"),
            7 => gettext("July"),
            8 => gettext("August"),
            9 => gettext("September"),
            10 => gettext("October"),
            11 => gettext("November"),
            12 => gettext("December"),
            // TODO: Fix the issues that stop us from panicking here.
            _ => "invalid month".to_string(),
        }
    }

    /// Returns the abbreviation of the month.
    #[template_callback]
    pub fn date_format(date: Date, format: &str) -> String {
        date.to_glib_date_time_utc()
            .format(format)
            .unwrap()
            .to_string()
    }
}
