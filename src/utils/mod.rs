use gettextrs::gettext;

mod attendee_type_filter;
mod child_property_ext;
mod macros;
mod paintable_callbacks;
mod participation_status_filter;
mod template_callbacks;

pub use self::{
    attendee_type_filter::*, child_property_ext::*, paintable_callbacks::*,
    participation_status_filter::*, template_callbacks::*,
};

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
