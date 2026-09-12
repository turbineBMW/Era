use std::cell::Cell;

use adw::{prelude::*, subclass::prelude::*};
use glib::DateTime;
use gtk::EventControllerFocus;

use crate::utils::Date;

mod imp {
    use super::*;

    #[derive(Debug, gtk::CompositeTemplate, glib::Properties)]
    #[template(file = "data/resources/ui/components/timeframe_picker/date_picker_row.blp")]
    #[properties(wrapper_type = super::DatePickerRow)]
    pub struct DatePickerRow {
        #[property(get, set = Self::set_date)]
        date: Cell<Date>,

        #[template_child]
        row_focus: TemplateChild<EventControllerFocus>,
        #[template_child]
        button_focus: TemplateChild<EventControllerFocus>,
        #[template_child]
        calendar: TemplateChild<gtk::Calendar>,
    }

    impl Default for DatePickerRow {
        fn default() -> Self {
            let date = jiff::civil::Date::new(1, 1, 1).unwrap();

            Self {
                date: Cell::new(Date::from(date)),
                row_focus: TemplateChild::default(),
                button_focus: TemplateChild::default(),
                calendar: TemplateChild::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DatePickerRow {
        const NAME: &'static str = "DatePickerRow";
        type Type = super::DatePickerRow;
        type ParentType = adw::EntryRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for DatePickerRow {}
    impl WidgetImpl for DatePickerRow {}
    impl ListBoxRowImpl for DatePickerRow {}
    impl PreferencesRowImpl for DatePickerRow {}
    impl EntryRowImpl for DatePickerRow {}

    #[gtk::template_callbacks]
    impl DatePickerRow {
        fn set_date(&self, date: Date) {
            if self.date.get() == date {
                return;
            }

            self.date.set(date);

            let date_time = date.to_glib_date_time_utc();
            self.calendar.set_date(&date_time);
            self.obj().set_text(
                &date_time
                    .format("%Y-%m-%d")
                    .expect("Date should be formattable"),
            );

            self.obj().notify_date();
        }

        #[template_callback]
        fn update_entry(&self) {
            let date_time = self.calendar.date();
            let date = Date::from(&date_time);

            self.date.set(date);
            self.obj().set_text(
                &date_time
                    .format("%Y-%m-%d")
                    .expect("Date should be formattable"),
            );

            self.obj().notify_date();
        }

        #[template_callback]
        fn update_calendar(&self) {
            if self.row_focus.contains_focus() && !self.button_focus.contains_focus() {
                return;
            }

            let text = self.obj().text();
            let parts: Vec<_> = text.split('-').collect();
            if parts.len() != 3 {
                self.obj().set_text(
                    &self
                        .calendar
                        .date()
                        .format("%Y-%m-%d")
                        .expect("Date should be formattable"),
                );
                return;
            }

            let (Ok(year), Ok(month), Ok(day)) = (
                parts[0].parse::<i32>(),
                parts[1].parse::<i32>(),
                parts[2].parse::<i32>(),
            ) else {
                self.obj().set_text(
                    &self
                        .calendar
                        .date()
                        .format("%Y-%m-%d")
                        .expect("Date should be formattable"),
                );
                return;
            };

            let Ok(date) = DateTime::from_utc(year, month, day, 0, 0, 0.) else {
                self.obj().set_text(
                    &self
                        .calendar
                        .date()
                        .format("%Y-%m-%d")
                        .expect("Date should be formattable"),
                );
                return;
            };

            self.calendar.set_date(&date);
        }
    }
}

glib::wrapper! {
    pub struct DatePickerRow(ObjectSubclass<imp::DatePickerRow>)
        @extends gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow, adw::EntryRow,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget,
            gtk::Editable;
}
