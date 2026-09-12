use std::{
    cell::{Cell, RefCell},
    marker::PhantomData,
};

use adw::{prelude::*, subclass::prelude::*};
use glib::{DateTime, TimeZone, clone};
use gtk::{Adjustment, EventControllerFocus, Popover, SpinButton, Widget};

use crate::utils::Date;

mod date_picker_row;
mod object_time_zone;
mod time_zone_picker_dialog;

use self::{date_picker_row::DatePickerRow, time_zone_picker_dialog::TimeZonePickerDialog};

mod imp {
    use super::*;

    #[derive(Debug, gtk::CompositeTemplate, glib::Properties)]
    #[template(file = "data/resources/ui/components/timeframe_picker/date_time_picker_group.blp")]
    #[properties(wrapper_type = super::DateTimePickerGroup)]
    pub struct DateTimePickerGroup {
        #[property(get, set)]
        date_title: RefCell<String>,
        #[property(get, set)]
        time_title: RefCell<String>,
        #[property(get, set)]
        date_only: Cell<bool>,
        #[property(get = Self::date_time, set = Self::set_date_time)]
        date_time: PhantomData<DateTime>,
        #[property(get, set = Self::set_error)]
        error: Cell<bool>,

        #[template_child]
        date: TemplateChild<DatePickerRow>,
        #[template_child]
        time: TemplateChild<adw::EntryRow>,
        #[template_child]
        hour: TemplateChild<Adjustment>,
        #[template_child]
        minute: TemplateChild<Adjustment>,
        #[template_child]
        popover: TemplateChild<Popover>,
        #[template_child]
        row_focus: TemplateChild<EventControllerFocus>,
        #[template_child]
        button_focus: TemplateChild<EventControllerFocus>,
        #[template_child]
        timezone_button_content: TemplateChild<adw::ButtonContent>,

        timezone: RefCell<TimeZone>,
    }

    impl Default for DateTimePickerGroup {
        fn default() -> Self {
            Self {
                date_title: RefCell::new(String::new()),
                time_title: RefCell::new(String::new()),
                date_only: Cell::new(false),
                date_time: PhantomData,
                error: Cell::new(false),
                date: TemplateChild::default(),
                time: TemplateChild::default(),
                hour: TemplateChild::default(),
                minute: TemplateChild::default(),
                popover: TemplateChild::default(),
                row_focus: TemplateChild::default(),
                button_focus: TemplateChild::default(),
                timezone_button_content: TemplateChild::default(),
                timezone: RefCell::new(TimeZone::utc()),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DateTimePickerGroup {
        const NAME: &'static str = "DateTimePickerGroup";
        type Type = super::DateTimePickerGroup;
        type ParentType = adw::PreferencesGroup;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for DateTimePickerGroup {
        fn constructed(&self) {
            self.parent_constructed();

            self.date.connect_date_notify(clone!(
                #[weak(rename_to = imp)]
                self,
                move |_| {
                    imp.obj().notify_date_time();
                }
            ));

            self.update_timezone_button_content();
        }
    }

    impl WidgetImpl for DateTimePickerGroup {}
    impl PreferencesGroupImpl for DateTimePickerGroup {}

    #[gtk::template_callbacks]
    impl DateTimePickerGroup {
        fn date_time(&self) -> DateTime {
            let date = self.date.date().to_glib_date_time_utc();

            DateTime::new(
                &self.timezone.borrow(),
                date.year(),
                date.month(),
                date.day_of_month(),
                self.hour.value() as i32,
                self.minute.value() as i32,
                0.0,
            )
            .unwrap()
        }

        fn set_date_time(&self, date_time: DateTime) {
            let date = Date::from(&date_time);
            self.date.set_date(date);

            self.hour.set_value(date_time.hour() as f64);
            self.minute.set_value(date_time.minute() as f64);

            self.time.set_text(&format!(
                "{:02}:{:02}",
                self.hour.value() as i32,
                self.minute.value() as i32
            ));

            self.timezone.replace(date_time.timezone());
            self.update_timezone_button_content();

            // Notification is sent by the handlers of DatePicker::notify_date or
            // Adjustment::notify_value if necessary
        }

        fn set_error(&self, error: bool) {
            if error == self.error.get() {
                return;
            }

            if error {
                self.date.add_css_class("error");
                self.time.add_css_class("error");
            } else {
                self.date.remove_css_class("error");
                self.time.remove_css_class("error");
            }

            self.error.set(error);
            self.obj().notify_error();
        }

        #[template_callback]
        fn update_date_time_from_spinner(&self) {
            let hour = self.hour.value() as u32;
            let minute = self.minute.value() as u32;

            self.time.set_text(&format!("{:02}:{:02}", hour, minute));

            self.obj().notify_date_time();
        }

        #[template_callback]
        fn format_spin_time(&self, spin_button: Widget) -> bool {
            let spin_button = spin_button
                .downcast::<SpinButton>()
                .expect("Widget should be a spin button");
            let adjustment = spin_button.adjustment();
            let value = adjustment.value();
            spin_button.set_text(&format!("{value:02}"));
            true
        }

        #[template_callback]
        fn maybe_parse_time(&self) {
            if self.row_focus.contains_focus() && !self.button_focus.contains_focus() {
                return;
            }

            let text = self.time.text();
            let parts: Vec<_> = text.split(|char: char| !char.is_numeric()).collect();

            let old_hour = self.hour.value() as i32;
            let old_minute = self.minute.value() as i32;

            let (hour, minute) = match parts.len() {
                1 => {
                    if let Ok(hour) = parts[0].parse::<i32>() {
                        (hour, 0)
                    } else {
                        (old_hour, old_minute)
                    }
                }
                2 => {
                    if let (Ok(hour), Ok(minute)) =
                        (parts[0].parse::<i32>(), parts[1].parse::<i32>())
                    {
                        (hour, minute)
                    } else {
                        (old_hour, old_minute)
                    }
                }
                _ => (old_hour, old_minute),
            };

            self.hour.set_value(hour as f64);
            self.minute.set_value(minute as f64);

            self.time.set_text(&format!("{:02}:{:02}", hour, minute));

            self.obj().notify_date_time();
        }

        #[template_callback]
        fn select_timezone(&self) {
            self.popover.popdown();
            let dialog = TimeZonePickerDialog::new();
            dialog.connect_time_zone_picked(clone!(
                #[weak(rename_to = imp)]
                self,
                move |_dialog, timezone_id| {
                    let timezone = TimeZone::from_identifier(Some(&timezone_id))
                        .expect("TimeZone ID should be valid");
                    imp.timezone.replace(timezone);

                    imp.update_timezone_button_content();

                    imp.obj().notify_date_time();
                }
            ));
            dialog.present(Some(&*self.obj()));
        }

        fn update_timezone_button_content(&self) {
            self.timezone_button_content
                .set_label(&self.timezone.borrow().identifier());
        }
    }
}

glib::wrapper! {
    pub struct DateTimePickerGroup(ObjectSubclass<imp::DateTimePickerGroup>)
        @extends gtk::Widget, adw::PreferencesGroup,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}
