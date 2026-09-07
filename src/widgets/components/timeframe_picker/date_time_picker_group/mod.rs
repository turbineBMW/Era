use std::cell::{Cell, RefCell};

use adw::{prelude::*, subclass::prelude::*};
use glib::{DateTime, TimeZone, clone};
use gtk::{Adjustment, EventControllerFocus, Popover, SpinButton, Widget};

mod date_picker_row;
mod object_time_zone;
mod time_zone_picker_dialog;

use self::{date_picker_row::DatePickerRow, time_zone_picker_dialog::TimeZonePickerDialog};

mod imp {
    use super::*;

    #[derive(Debug, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Era/date_time_picker_group.ui")]
    #[properties(wrapper_type = super::DateTimePickerGroup)]
    pub struct DateTimePickerGroup {
        #[property(get, set)]
        date_title: RefCell<String>,
        #[property(get, set)]
        time_title: RefCell<String>,
        #[property(get, set)]
        date_only: Cell<bool>,
        #[property(get, set = Self::set_date_time)]
        date_time: RefCell<DateTime>,
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
    }

    impl Default for DateTimePickerGroup {
        fn default() -> Self {
            let timezone = TimeZone::local();
            let now = DateTime::now_local().expect("Local time should be available");
            Self {
                date_title: RefCell::default(),
                time_title: RefCell::default(),
                date_only: Cell::default(),
                date_time: RefCell::new(
                    DateTime::from_utc(now.year(), now.month(), now.day_of_month(), 0, 0, 0.)
                        .and_then(|d| d.to_timezone(&timezone))
                        .expect("Date should be valid"),
                ),
                error: Cell::default(),
                date: TemplateChild::default(),
                time: TemplateChild::default(),
                hour: TemplateChild::default(),
                minute: TemplateChild::default(),
                popover: TemplateChild::default(),
                row_focus: TemplateChild::default(),
                button_focus: TemplateChild::default(),
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
                    let time = imp.obj().date_time();
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
        fn set_date_time(&self, date_time: DateTime) {
            if self.date_time.borrow().clone() == date_time {
                return;
            }

            self.date.set_date(&date_time);
            self.date_time.replace(date_time);

            self.obj().notify_date_time();
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
        fn format(&self) -> String {
            let date_time = self.obj().date_time();
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

            let old_date_time = self.obj().date_time();
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
            let date_time = self.obj().date_time();
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
            let date = self.obj().date_time();
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
            self.obj().date_time().hour()
        }

        #[template_callback]
        fn minute(&self) -> i32 {
            self.obj().date_time().minute()
        }

        #[template_callback]
        fn timezone(&self) -> String {
            self.obj().date_time().timezone().identifier().to_string()
        }

        #[template_callback]
        fn select_timezone(&self) {
            self.popover.popdown();
            let dialog = TimeZonePickerDialog::new();
            dialog.connect_time_zone_picked(clone!(
                #[weak(rename_to = imp)]
                self,
                move |_dialog, time_zone_id| {
                    let date = imp.obj().date_time();
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
