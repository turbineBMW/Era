use adw::{prelude::*, subclass::prelude::*};
use clepsydre::{Calendar, prelude::*};
use glib::{DateTime, clone};
use tracing::{debug, warn};

mod calendar_combo_row;

use crate::{
    spawn,
    utils::TemplateCallbacks,
    widgets::components::{ErrorDialog, LoadingButton, TimeframePicker},
};

use self::calendar_combo_row::CalendarComboRow;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/gitlab/TitouanReal/Era/create_event_dialog.ui")]
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
        conference: TemplateChild<adw::EntryRow>,
        #[template_child]
        calendar_choice: TemplateChild<CalendarComboRow>,
        #[template_child]
        timeframe_picker: TemplateChild<TimeframePicker>,
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

    impl ObjectImpl for CreateEventDialog {
        fn constructed(&self) {
            self.parent_constructed();

            let update_save_action = clone!(
                #[weak(rename_to = imp)]
                self,
                move || {
                    let name = imp.name.text();
                    let timeframe = imp.timeframe_picker.timeframe();
                    let is_invalid = name.trim().is_empty() || timeframe.is_none();
                    let enabled = !is_invalid;
                    imp.obj()
                        .action_set_enabled("create-event-dialog.save", enabled);
                }
            );

            self.name.connect_changed(clone!(
                #[strong]
                update_save_action,
                move |_| update_save_action()
            ));
            self.timeframe_picker.connect_timeframe_notify(clone!(
                #[strong]
                update_save_action,
                move |_| update_save_action()
            ));
            self.description.connect_changed(clone!(
                #[strong]
                update_save_action,
                move |_| update_save_action()
            ));

            update_save_action();
        }
    }

    impl WidgetImpl for CreateEventDialog {}
    impl AdwDialogImpl for CreateEventDialog {}

    #[gtk::template_callbacks]
    impl CreateEventDialog {
        #[template_callback]
        fn focus_name(&self) {
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
