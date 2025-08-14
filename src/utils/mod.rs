use gettextrs::gettext;

mod macros;
mod paintables;

pub use paintables::*;

pub fn get_month_name(month: i8) -> String {
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
        _ => gettext("Invalid Month"),
    }
}
