use std::cell::RefCell;

use adw::{prelude::*, subclass::prelude::*};
use glib::DateTime;
use gtk::EventControllerFocus;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Kalendasom/date_picker_row.ui")]
    #[properties(wrapper_type = super::DatePickerRow)]
    pub struct DatePickerRow {
        #[property(get, set)]
        date: RefCell<Option<DateTime>>,
        #[template_child]
        row_focus: TemplateChild<EventControllerFocus>,
        #[template_child]
        button_focus: TemplateChild<EventControllerFocus>,
        #[template_child]
        calendar: TemplateChild<gtk::Calendar>,
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
        #[template_callback]
        fn format(&self) -> String {
            let Some(date) = self.obj().date() else {
                return String::new();
            };

            date.format("%Y-%m-%d")
                .expect("Date should be formattable")
                .to_string()
        }

        #[template_callback]
        fn maybe_validate_entry(&self) {
            if self.row_focus.contains_focus() && !self.button_focus.contains_focus() {
                return;
            }

            let text = self.obj().text();
            let parts: Vec<_> = text.split('-').collect();
            if parts.len() != 3 {
                self.set_entry_from_date();
                return;
            }

            let (Ok(year), Ok(month), Ok(day)) = (
                parts[0].parse::<i32>(),
                parts[1].parse::<i32>(),
                parts[2].parse::<i32>(),
            ) else {
                self.set_entry_from_date();
                return;
            };

            let Ok(date) = DateTime::from_utc(year, month, day, 0, 0, 0.) else {
                self.set_entry_from_date();
                return;
            };

            self.obj().set_date(date);
        }

        fn set_entry_from_date(&self) {
            self.obj().set_text(
                &self
                    .obj()
                    .date()
                    .expect("Date should be initialized")
                    .format("%Y-%m-%d")
                    .expect("Date should be formattable"),
            );
        }
    }
}

glib::wrapper! {
    pub struct DatePickerRow(ObjectSubclass<imp::DatePickerRow>)
        @extends gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow, adw::EntryRow,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget,
            gtk::Editable;
}

impl DatePickerRow {
    pub fn new() -> Self {
        glib::Object::new()
    }
}

impl Default for DatePickerRow {
    fn default() -> Self {
        Self::new()
    }
}
