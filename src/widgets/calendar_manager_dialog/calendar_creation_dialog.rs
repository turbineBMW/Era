use std::cell::RefCell;

use adw::{prelude::*, subclass::prelude::*};
use clepsydre::Collection;
use tracing::{debug, warn};

use crate::widgets::components::{ErrorDialog, LoadingButton};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Kalendasom/calendar_creation_dialog.ui")]
    #[properties(wrapper_type = super::CalendarCreationDialog)]
    pub struct CalendarCreationDialog {
        #[property(get, set, construct_only)]
        pub collection: RefCell<Option<Collection>>,
        #[template_child]
        pub toast_overlay: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        pub cancel: TemplateChild<gtk::Button>,
        #[template_child]
        pub create: TemplateChild<LoadingButton>,
        #[template_child]
        pub name: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub color: TemplateChild<gtk::ColorDialogButton>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CalendarCreationDialog {
        const NAME: &'static str = "CalendarCreationDialog";
        type Type = super::CalendarCreationDialog;
        type ParentType = adw::Dialog;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();

            klass.install_action(
                "calendar-creation-dialog.show-error",
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

    #[glib::derived_properties]
    impl ObjectImpl for CalendarCreationDialog {}
    impl WidgetImpl for CalendarCreationDialog {}
    impl AdwDialogImpl for CalendarCreationDialog {}

    #[gtk::template_callbacks]
    impl CalendarCreationDialog {
        #[template_callback]
        async fn create_calendar(&self) {
            self.cancel.set_sensitive(false);
            self.create.set_is_loading(true);

            let collection = self
                .obj()
                .collection()
                .expect("collection should be initialized");

            match collection
                .try_create_calendar_future(&self.name.text(), self.color.rgba())
                .await
            {
                Ok(calendar) => {
                    debug!("Calendar created: {}", calendar.uri());
                    self.obj().close();
                }
                Err(error) => {
                    self.cancel.set_sensitive(true);
                    self.create.set_is_loading(false);

                    warn!("Failed to create calendar: {error}");
                    self.toast_overlay.dismiss_all();
                    let toast = adw::Toast::new("An error occurred");
                    toast.set_button_label(Some("Details"));
                    toast.set_action_name(Some("calendar-creation-dialog.show-error"));
                    toast.set_action_target(Some(&error.message()));
                    self.toast_overlay.add_toast(toast);
                }
            }
        }
    }
}

glib::wrapper! {
    pub struct CalendarCreationDialog(ObjectSubclass<imp::CalendarCreationDialog>)
        @extends gtk::Widget, adw::Dialog,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::ShortcutManager;
}

impl CalendarCreationDialog {
    pub fn new(collection: &Collection) -> Self {
        glib::Object::builder()
            .property("collection", collection)
            .build()
    }
}
