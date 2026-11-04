use std::cell::{Cell, RefCell};

use adw::{prelude::*, subclass::prelude::*};
use clepsydre::{Event, Timeframe};
use gettextrs::gettext;
use glib::{DateTime, TimeSpan, TimeZone};

use crate::{system::ClockFormat, utils::Date, widgets::components::ColorStripe};

/// Returns the 1-based day index of this row, and the event's total number of calendar days, in
/// the given timezone.
fn event_day_position(timeframe: &Timeframe, date: Date, timezone: &TimeZone) -> (i32, i32) {
    let start = timeframe.start();
    let end = timeframe.end();

    // Use the last included instant for non-zero duration event
    let end = if end > start {
        end.add(TimeSpan::from_microseconds(-1)).unwrap()
    } else {
        end
    };

    let (start, end) = if timeframe.is_all_day() {
        (start, end)
    } else {
        (
            start.to_timezone(timezone).unwrap(),
            end.to_timezone(timezone).unwrap(),
        )
    };

    let start_jiff = Date::from(&start).to_jiff();
    let end_jiff = Date::from(&end).to_jiff();
    let date_jiff = date.to_jiff();

    let day_index = (date_jiff - start_jiff).get_days() + 1;
    let day_count = (end_jiff - start_jiff).get_days() + 1;
    (day_index, day_count)
}

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(file = "data/resources/ui/views/agenda_view/agenda_view_row.blp")]
    #[properties(wrapper_type = super::AgendaViewRow)]
    pub struct AgendaViewRow {
        #[template_child]
        top_label: TemplateChild<gtk::Label>,
        #[template_child]
        bottom_label: TemplateChild<gtk::Label>,

        #[property(get, set)]
        date: Cell<Date>,
        #[property(get, set)]
        event: RefCell<Option<Event>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AgendaViewRow {
        const NAME: &'static str = "AgendaViewRow";
        type Type = super::AgendaViewRow;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            ColorStripe::ensure_type();

            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for AgendaViewRow {}
    impl WidgetImpl for AgendaViewRow {}
    impl BoxImpl for AgendaViewRow {}

    #[gtk::template_callbacks]
    impl AgendaViewRow {
        #[template_callback]
        fn top_label(
            &self,
            timeframe: Timeframe,
            date: Date,
            system_datetime: DateTime,
            clock_format: ClockFormat,
        ) -> String {
            let local_timezone = system_datetime.timezone();
            let start = timeframe.start();

            let (day_index, day_count) = event_day_position(&timeframe, date, &local_timezone);

            let time_format = match clock_format {
                ClockFormat::TwelveHours => "%-I:%M %p",
                ClockFormat::TwentyFourHours => "%H:%M",
            };

            if timeframe.is_all_day() {
                self.top_label.remove_css_class("numeric");
                gettext("All-day")
            } else {
                let converted_start = start.to_timezone(&local_timezone).unwrap();

                if day_index == 1 {
                    self.top_label.add_css_class("numeric");
                    converted_start.format(time_format).unwrap().to_string()
                } else if day_index < day_count {
                    self.top_label.remove_css_class("numeric");
                    gettext("All-day")
                } else {
                    self.top_label.remove_css_class("numeric");
                    // TRANSLATORS: last day of a multiday event, eg Day 3/3
                    // {day_count} is the number of days that the event span
                    gettext("Day {day_count}/{day_count}")
                        .replace("{day_count}", &day_count.to_string())
                }
            }
        }

        #[template_callback]
        fn bottom_label(
            &self,
            timeframe: Timeframe,
            date: Date,
            system_datetime: DateTime,
            clock_format: ClockFormat,
        ) -> String {
            let local_timezone = system_datetime.timezone();
            let start = timeframe.start();
            let end = timeframe.end();

            let (day_index, day_count) = event_day_position(&timeframe, date, &local_timezone);

            let time_format = match clock_format {
                ClockFormat::TwelveHours => "%-I:%M %p",
                ClockFormat::TwentyFourHours => "%H:%M",
            };

            if timeframe.is_all_day() {
                if day_count == 1 {
                    String::new()
                } else {
                    self.bottom_label.remove_css_class("numeric");
                    // TRANSLATORS: day of a multiday event, eg Day 2/3
                    // {day_index} is the index of the day in the event span
                    // {day_count} is the number of days that the event span
                    gettext("Day {day_index}/{day_count}")
                        .replace("{day_index}", &day_index.to_string())
                        .replace("{day_count}", &day_count.to_string())
                }
            } else {
                let converted_end = end.to_timezone(&local_timezone).unwrap();

                // Zero-length events don't have a bottom label.
                if start == end{
                    String::new()
                } else if day_index == day_count {
                    self.bottom_label.add_css_class("numeric");
                    converted_end.format(time_format).unwrap().to_string()
                } else {
                    self.bottom_label.remove_css_class("numeric");
                    // TRANSLATORS: day of a multiday event, eg Day 2/3
                    // {day_index} is the index of the day in the event span
                    // {day_count} is the number of days that the event span
                    gettext("Day {day_index}/{day_count}")
                        .replace("{day_index}", &day_index.to_string())
                        .replace("{day_count}", &day_count.to_string())
                }
            }
        }
    }
}

glib::wrapper! {
    pub struct AgendaViewRow(ObjectSubclass<imp::AgendaViewRow>)
    @extends gtk::Widget, gtk::Box,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl AgendaViewRow {
    pub fn new() -> Self {
        glib::Object::new()
    }
}
