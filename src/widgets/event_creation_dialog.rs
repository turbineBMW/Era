use std::str::FromStr;

use adw::{prelude::*, subclass::prelude::*};
use clepsydre::{Calendar, prelude::*};
use glib::{DateTime, TimeZone, clone};
use tracing::{debug, warn};

use crate::{
    spawn,
    utils::{EventPropertiesPreset, TemplateCallbacks},
    widgets::components::{CalendarComboRow, ErrorDialog, LoadingButton, TimeframePicker},
};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(file = "data/resources/ui/event_creation_dialog.blp")]
    pub struct EventCreationDialog {
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
        conference: TemplateChild<adw::EntryRow>,
        #[template_child]
        calendar_choice: TemplateChild<CalendarComboRow>,
        #[template_child]
        timeframe_picker: TemplateChild<TimeframePicker>,
        #[template_child]
        description: TemplateChild<adw::EntryRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for EventCreationDialog {
        const NAME: &'static str = "EventCreationDialog";
        type Type = super::EventCreationDialog;
        type ParentType = adw::Dialog;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
            TemplateCallbacks::bind_template_callbacks(klass);

            klass.install_action("create-event-dialog.save", None, |obj, _, _| {
                let imp = obj.imp();
                spawn!(clone!(
                    #[weak]
                    imp,
                    async move {
                        imp.create_event().await;
                    }
                ));
            });
            klass.add_binding_action(
                gdk::Key::S,
                gdk::ModifierType::CONTROL_MASK,
                "create-event-dialog.save",
            );

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

    impl ObjectImpl for EventCreationDialog {
        fn constructed(&self) {
            self.parent_constructed();

            self.name.connect_changed(clone!(
                #[weak(rename_to = imp)]
                self,
                move |_| imp.update_save_action()
            ));
            self.timeframe_picker.connect_timeframe_notify(clone!(
                #[weak(rename_to = imp)]
                self,
                move |_| imp.update_save_action()
            ));

            self.update_save_action();
        }
    }

    impl WidgetImpl for EventCreationDialog {}
    impl AdwDialogImpl for EventCreationDialog {}

    #[gtk::template_callbacks]
    impl EventCreationDialog {
        pub(super) fn set_data(&self, preset: EventPropertiesPreset) {
            self.name.set_text(&preset.name);
            self.description.set_text(&preset.description);
            self.location.set_text(&preset.location);
            self.conference.set_text(&preset.conference);

            let start = jiff::Zoned::from_str(&preset.start).unwrap();
            let end = jiff::Zoned::from_str(&preset.end).unwrap();

            let start_tz_id = start.time_zone().iana_name().unwrap();
            let start_tz = TimeZone::from_identifier(Some(start_tz_id)).unwrap();
            let glib_start = DateTime::new(
                &start_tz,
                start.year() as i32,
                start.month() as i32,
                start.day() as i32,
                start.hour() as i32,
                start.minute() as i32,
                0.,
            )
            .expect("Failed to create glib::DateTime");

            let end_tz_id = end.time_zone().iana_name().unwrap();
            let end_tz = TimeZone::from_identifier(Some(end_tz_id)).unwrap();
            let glib_end = DateTime::new(
                &end_tz,
                end.year() as i32,
                end.month() as i32,
                end.day() as i32,
                end.hour() as i32,
                end.minute() as i32,
                0.,
            )
            .expect("Failed to create glib::DateTime");

            self.timeframe_picker
                .set_data(preset.all_day, &glib_start, &glib_end);
        }

        fn update_save_action(&self) {
            let name = self.name.text();
            let timeframe = self.timeframe_picker.timeframe();

            let name_is_empty = name.trim().is_empty();
            let timeframe_is_invalid = timeframe.is_none();

            let is_invalid = name_is_empty || timeframe_is_invalid;
            let enabled = !is_invalid;
            self.obj()
                .action_set_enabled("create-event-dialog.save", enabled);
        }

        #[template_callback]
        fn name_grab_focus(&self) {
            self.name.grab_focus();
        }

        #[template_callback(function)]
        fn invalid_schedule(schedule_type: &str, start: DateTime, end: DateTime) -> bool {
            if schedule_type == "all-day" {
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

        async fn create_event(&self) {
            self.toast_overlay.dismiss_all();
            self.cancel.set_sensitive(false);
            self.create.set_is_loading(true);
            self.name.set_sensitive(false);
            self.location.set_sensitive(false);
            self.conference.set_sensitive(false);
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
            let conference = self.conference.text();
            let timeframe = self.timeframe_picker.timeframe().expect(
                "A timeframe should be set if the user was able to activate the event creation",
            );

            match calendar
                .try_create_event_future(&name, &description, &location, &conference, &timeframe)
                .await
            {
                Ok(event) => {
                    debug!("Event created: {}", event.uri().unwrap());
                    self.obj().close();
                }
                Err(error) => {
                    warn!("Failed to create event: {error}");
                    let toast = adw::Toast::new("An error occurred");
                    toast.set_button_label(Some("Details"));
                    toast.set_action_name(Some("create-event-dialog.show-error"));
                    toast.set_action_target(Some(&error.message()));
                    self.toast_overlay.add_toast(toast);
                }
            }

            self.cancel.set_sensitive(true);
            self.create.set_is_loading(false);
            self.name.set_sensitive(true);
            self.location.set_sensitive(true);
            self.conference.set_sensitive(true);
            self.calendar_choice.set_sensitive(true);
            self.description.set_sensitive(true);
        }
    }
}

glib::wrapper! {
    pub struct EventCreationDialog(ObjectSubclass<imp::EventCreationDialog>)
        @extends gtk::Widget, adw::Dialog,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::ShortcutManager;
}

impl EventCreationDialog {
    pub fn set_data(&self, preset: EventPropertiesPreset) {
        self.imp().set_data(preset);
    }
}
