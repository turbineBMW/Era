use crate::utils::WeekDay;

/// Returns the last occurrence of the given weekday before or on the given date.
pub fn get_last_occurrence_of_weekday(date: glib::DateTime, weekday: WeekDay) -> glib::DateTime {
    let base = match weekday {
        WeekDay::Monday => 1,
        WeekDay::Tuesday => 2,
        WeekDay::Wednesday => 3,
        WeekDay::Thursday => 4,
        WeekDay::Friday => 5,
        WeekDay::Saturday => 6,
        WeekDay::Sunday => 7,
    };
    // This returns the number of the weekday of date, from 1 (Monday) to 7 (Sunday)
    let offset = date.day_of_week();

    let go_back_by = (offset - base).rem_euclid(7);

    date.add_days(-go_back_by)
        .expect("DateTime should be valid")
}
