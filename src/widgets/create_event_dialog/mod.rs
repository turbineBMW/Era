use adw::{prelude::*, subclass::prelude::*};
use clepsydre::{Calendar, Timeframe};
use tracing::{debug, warn};

mod calendar_combo_row;

use crate::widgets::components::{ErrorDialog, LoadingButton};

use self::calendar_combo_row::CalendarComboRow;

mod imp {
    use glib::DateTime;

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

    // TODO: Call adw_entry_row_grab_focus_without_selecting on the name entry row
    impl ObjectImpl for CreateEventDialog {}
    impl WidgetImpl for CreateEventDialog {}
    impl AdwDialogImpl for CreateEventDialog {}

    #[gtk::template_callbacks]
    impl CreateEventDialog {
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
            let start = DateTime::from_iso8601("20250929T000000Z", None)
                .expect("Failed to parse start date");
            let end =
                DateTime::from_iso8601("20251001T000000Z", None).expect("Failed to parse end date");
            let timeframe = Timeframe::new(
                true,
                &start.format_iso8601().unwrap(),
                &end.format_iso8601().unwrap(),
            );
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
                    debug!("Event created: {}", event.unwrap().uri().unwrap());
                    self.obj().close();
                }
                Err(error) => {
                    self.cancel.set_sensitive(true);
                    self.create.set_is_loading(false);
                    self.description.set_sensitive(true);
                    self.calendar_choice.set_sensitive(true);
                    self.name.set_sensitive(true);

                    warn!("Failed to create calendar: {error}");
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
