use glib::DateTime;

use crate::system::DayOfWeek;

/// Returns the last occurrence of the given weekday before or on the given date.
pub fn get_last_occurrence_of_weekday(date: DateTime, weekday: DayOfWeek) -> DateTime {
    let base = match weekday {
        DayOfWeek::Monday => 1,
        DayOfWeek::Tuesday => 2,
        DayOfWeek::Wednesday => 3,
        DayOfWeek::Thursday => 4,
        DayOfWeek::Friday => 5,
        DayOfWeek::Saturday => 6,
        DayOfWeek::Sunday => 7,
    };
    // This returns the number of the weekday of date, from 1 (Monday) to 7 (Sunday)
    let offset = date.day_of_week();

    let go_back_by = (offset - base).rem_euclid(7);

    date.add_days(-go_back_by)
        .expect("DateTime should be valid")
}
