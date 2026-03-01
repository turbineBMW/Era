use adw::{prelude::*, subclass::prelude::*};
use clepsydre::{Calendar, Timeframe};
use glib::DateTime;
use tracing::{debug, warn};

mod calendar_combo_row;
mod date_picker_row;
mod date_time_picker_group;

use crate::{
    utils::TemplateCallbacks,
    widgets::components::{ErrorDialog, LoadingButton},
};

use self::{calendar_combo_row::CalendarComboRow, date_time_picker_group::DateTimePickerGroup};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/gitlab/TitouanReal/Kalendasom/create_event_dialog.ui")]
    pub struct CreateEventDialog {
        #[template_child]
        toast_overlay: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        cancel: TemplateChild<gtk::Button>,
        #[template_child]
        create: TemplateChild<LoadingButton>,
        #[template_child]
        name: TemplateChild<adw::EntryRow>,
        #[template_child]
        location: TemplateChild<adw::EntryRow>,
        #[template_child]
        video_conference: TemplateChild<adw::EntryRow>,
        #[template_child]
        calendar_choice: TemplateChild<CalendarComboRow>,
        #[template_child]
        schedule_type: TemplateChild<adw::ToggleGroup>,
        #[template_child]
        start: TemplateChild<DateTimePickerGroup>,
        #[template_child]
        end: TemplateChild<DateTimePickerGroup>,
        #[template_child]
        description: TemplateChild<adw::EntryRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CreateEventDialog {
        const NAME: &'static str = "CreateEventDialog";
        type Type = super::CreateEventDialog;
        type ParentType = adw::Dialog;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
            TemplateCallbacks::bind_template_callbacks(klass);

            klass.install_action(
                "create-event-dialog.show-error",
                Some(&String::static_variant_type()),
                |obj, _, param| {
                    let error_message = &param
                        .and_then(glib::Variant::get::<String>)
                        .expect("The parameter should be a string");
                    let dialog = ErrorDialog::new(error_message);
                    dialog.present(Some(obj));
                },
            );
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for CreateEventDialog {}
    impl WidgetImpl for CreateEventDialog {}
    impl AdwDialogImpl for CreateEventDialog {}

    #[gtk::template_callbacks]
    impl CreateEventDialog {
        #[template_callback]
        fn focus_name(&self) {
            self.name.grab_focus();
        }

        #[template_callback(function)]
        fn invalid_schedule(
            schedule_type: &str,
            start: Option<DateTime>,
            end: Option<DateTime>,
        ) -> bool {
            let all_day = schedule_type == "all-day";
            let Some(start) = start else {
                return false;
            };
            let Some(end) = end else {
                return false;
            };

            if all_day {
                let start_date =
                    DateTime::from_utc(start.year(), start.month(), start.day_of_month(), 0, 0, 0.)
                        .expect("Date should be valid");
                let end_date =
                    DateTime::from_utc(end.year(), end.month(), end.day_of_month(), 0, 0, 0.)
                        .expect("Date should be valid");
                end_date < start_date
            } else {
                end < start
            }
        }

        #[template_callback(function)]
        fn is_event_valid(
            name: &str,
            schedule_type: &str,
            start: Option<DateTime>,
            end: Option<DateTime>,
        ) -> bool {
            let all_day = schedule_type == "all-day";
            let Some(start) = start else {
                return false;
            };
            let Some(end) = end else {
                return false;
            };

            let invalid_name = name.is_empty();

            let invalid_schedule = if all_day {
                let start_date =
                    DateTime::from_utc(start.year(), start.month(), start.day_of_month(), 0, 0, 0.)
                        .expect("Date should be valid");
                let end_date =
                    DateTime::from_utc(end.year(), end.month(), end.day_of_month(), 0, 0, 0.)
                        .expect("Date should be valid");
                end_date < start_date
            } else {
                end < start
            };

            invalid_name || invalid_schedule
        }

        #[template_callback]
        async fn create_event(&self) {
            self.cancel.set_sensitive(false);
            self.create.set_is_loading(true);
            self.name.set_sensitive(false);
            self.calendar_choice.set_sensitive(false);
            self.description.set_sensitive(false);

            let calendar: Calendar = self
                .calendar_choice
                .selected_item()
                .expect("There should be a selected item")
                .downcast()
                .expect("Selected item should be a Calendar");

            let name = self.name.text();
            let description = self.description.text();
            let location = self.location.text();
            let video_conference = self.video_conference.text();
            let all_day = self
                .schedule_type
                .active_name()
                .expect("A schedule type should be set")
                == "all-day";
            let start = self
                .start
                .date_time()
                .expect("DateTime should be initialized");
            let end = self
                .end
                .date_time()
                .expect("DateTime should be initialized");
            let timeframe = Timeframe::new(all_day, &start, &end);
            match calendar
                .try_create_event_future(
                    &name,
                    &description,
                    &location,
                    &video_conference,
                    &timeframe,
                )
                .await
            {
                Ok(event) => {
                    debug!("Event created: {}", event.uri().unwrap());
                    self.obj().close();
                }
                Err(error) => {
                    self.cancel.set_sensitive(true);
                    self.create.set_is_loading(false);
                    self.description.set_sensitive(true);
                    self.calendar_choice.set_sensitive(true);
                    self.name.set_sensitive(true);

                    warn!("Failed to create event: {error}");
                    self.toast_overlay.dismiss_all();
                    let toast = adw::Toast::new("An error occurred");
                    toast.set_button_label(Some("Details"));
                    toast.set_action_name(Some("create-event-dialog.show-error"));
                    toast.set_action_target(Some(&error.message()));
                    self.toast_overlay.add_toast(toast);
                }
            }
        }
    }
}

glib::wrapper! {
    pub struct CreateEventDialog(ObjectSubclass<imp::CreateEventDialog>)
        @extends gtk::Widget, adw::Dialog,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::ShortcutManager;
}

impl CreateEventDialog {
    pub fn new() -> Self {
        glib::Object::new()
    }
}

impl Default for CreateEventDialog {
    fn default() -> Self {
        Self::new()
    }
}
