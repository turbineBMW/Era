use gettextrs::gettext;

mod child_property_ext;
mod macros;
mod paintable_callbacks;
mod template_callbacks;

pub use self::{child_property_ext::*, paintable_callbacks::*, template_callbacks::*};

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
