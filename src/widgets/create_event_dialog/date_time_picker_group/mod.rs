use std::cell::{Cell, RefCell};

use adw::{prelude::*, subclass::prelude::*};
use glib::{DateTime, TimeZone, clone};
use gtk::{Adjustment, EventControllerFocus, Popover, SpinButton, Widget};

mod date_picker_row;
mod object_time_zone;
mod time_zone_picker_dialog;
mod time_zone_row;

use self::{date_picker_row::DatePickerRow, time_zone_picker_dialog::TimeZonePickerDialog};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Kalendasom/date_time_picker_group.ui")]
    #[properties(wrapper_type = super::DateTimePickerGroup)]
    pub struct DateTimePickerGroup {
        #[property(get, set)]
        date_title: RefCell<Option<String>>,
        #[property(get, set)]
        time_title: RefCell<Option<String>>,
        #[property(get, set)]
        date_only: Cell<bool>,
        #[property(get, set)]
        date_time: RefCell<Option<DateTime>>,
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
            let date = self.date.date().expect("Date should be initialized");
            let timezone = TimeZone::local();

            self.obj().set_date_time(
                DateTime::new(
                    &timezone,
                    date.year(),
                    date.month(),
                    date.day_of_month(),
                    0,
                    0,
                    0.,
                )
                .unwrap(),
            );

            self.date.connect_date_notify(clone!(
                #[weak(rename_to = imp)]
                self,
                move |_| {
                    let time = imp
                        .obj()
                        .date_time()
                        .expect("DateTime should be initialized");
                    let date = imp.date.date().expect("Date should be initialized");
                    imp.obj().set_date_time(
                        DateTime::new(
                            &time.timezone(),
                            date.year(),
                            date.month(),
                            date.day_of_month(),
                            time.hour(),
                            time.minute(),
                            0.,
                        )
                        .unwrap(),
                    );
                }
            ));
        }
    }

    impl WidgetImpl for DateTimePickerGroup {}
    impl PreferencesGroupImpl for DateTimePickerGroup {}

    #[gtk::template_callbacks]
    impl DateTimePickerGroup {
        #[template_callback]
        fn format(&self) -> String {
            let Some(date_time) = self.obj().date_time() else {
                return String::new();
            };
            format!("{:02}:{:02}", date_time.hour(), date_time.minute())
        }

        #[template_callback]
        fn maybe_validate_entry(&self) {
            if self.row_focus.contains_focus() && !self.button_focus.contains_focus() {
                return;
            }

            let text = self.time.text();
            let parts: Vec<_> = text.split(|c: char| !c.is_numeric()).collect();

            let (hour, minute) = match parts.len() {
                1 => {
                    let Ok(hour) = parts[0].parse::<i32>() else {
                        self.set_entry_from_time();
                        return;
                    };
                    (hour, 0)
                }
                2 => {
                    let (Ok(hour), Ok(minute)) = (parts[0].parse::<i32>(), parts[1].parse::<i32>())
                    else {
                        self.set_entry_from_time();
                        return;
                    };
                    (hour, minute)
                }
                _ => {
                    self.set_entry_from_time();
                    return;
                }
            };

            let old_date_time = self
                .obj()
                .date_time()
                .expect("DateTime should be initialized");
            let Ok(new_date_time) = DateTime::new(
                &old_date_time.timezone(),
                old_date_time.year(),
                old_date_time.month(),
                old_date_time.day_of_month(),
                hour,
                minute,
                0.,
            ) else {
                self.set_entry_from_time();
                return;
            };
            self.obj().set_date_time(new_date_time);
        }

        fn set_entry_from_time(&self) {
            let date_time = self
                .obj()
                .date_time()
                .expect("DateTime should be initialized");
            self.time.set_text(&format!(
                "{:02}:{:02}",
                date_time.hour(),
                date_time.minute()
            ));
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
        fn update_time(&self) {
            let date = self
                .obj()
                .date_time()
                .expect("DateTime should be initialized");
            let hour = self.hour.value();
            let minute = self.minute.value();
            self.obj().set_date_time(
                DateTime::new(
                    &date.timezone(),
                    date.year(),
                    date.month(),
                    date.day_of_month(),
                    hour as i32,
                    minute as i32,
                    0.,
                )
                .expect("DateTime should be constructible"),
            );
        }

        #[template_callback]
        fn hour(&self) -> i32 {
            let Some(date_time) = self.obj().date_time() else {
                return 0;
            };

            date_time.hour()
        }

        #[template_callback]
        fn minute(&self) -> i32 {
            let Some(date_time) = self.obj().date_time() else {
                return 0;
            };

            date_time.minute()
        }

        #[template_callback]
        fn timezone(&self) -> String {
            let Some(date_time) = self.obj().date_time() else {
                return String::new();
            };

            date_time.timezone().identifier().to_string()
        }

        #[template_callback]
        fn select_timezone(&self) {
            self.popover.popdown();
            let dialog = TimeZonePickerDialog::new();
            dialog.connect_time_zone_picked(clone!(
                #[weak(rename_to = imp)]
                self,
                move |_dialog, time_zone_id| {
                    let date = imp
                        .obj()
                        .date_time()
                        .expect("DateTime should be initialized");
                    let time_zone = TimeZone::from_identifier(Some(&time_zone_id))
                        .expect("TimeZone ID should be valid");
                    imp.obj().set_date_time(
                        DateTime::new(
                            &time_zone,
                            date.year(),
                            date.month(),
                            date.day_of_month(),
                            date.hour(),
                            date.minute(),
                            0.,
                        )
                        .unwrap(),
                    );
                }
            ));
            dialog.present(Some(&*self.obj()));
        }
    }
}

glib::wrapper! {
    pub struct DateTimePickerGroup(ObjectSubclass<imp::DateTimePickerGroup>)
        @extends gtk::Widget, adw::PreferencesGroup,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl DateTimePickerGroup {
    pub fn new() -> Self {
        glib::Object::new()
    }
}

impl Default for DateTimePickerGroup {
    fn default() -> Self {
        Self::new()
    }
}
