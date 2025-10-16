use std::cell::RefCell;

use adw::{prelude::*, subclass::prelude::*};
use clepsydre::Collection;
use tracing::{debug, warn};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Kalendasom/calendar_creation_dialog.ui")]
    #[properties(wrapper_type = super::CalendarCreationDialog)]
    pub struct CalendarCreationDialog {
        #[property(get, set, construct_only)]
        pub collection: RefCell<Option<Collection>>,
        #[template_child]
        pub cancel: TemplateChild<gtk::Button>,
        #[template_child]
        pub create: TemplateChild<gtk::Button>,
        #[template_child]
        pub name: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub color: TemplateChild<gtk::ColorDialogButton>,
        #[template_child]
        pub spinner: TemplateChild<adw::Spinner>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CalendarCreationDialog {
        const NAME: &'static str = "CalendarCreationDialog";
        type Type = super::CalendarCreationDialog;
        type ParentType = adw::Dialog;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
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
            self.spinner.set_visible(true);
            self.cancel.set_sensitive(false);
            self.create.set_sensitive(false);
            match self
                .obj()
                .collection()
                .expect("collection should be initialized")
                .try_create_calendar_future(&self.name.text(), self.color.rgba())
                .await
            {
                Ok(calendar) => {
                    debug!("Calendar created: {:?}", calendar);
                }
                Err(e) => {
                    self.spinner.set_visible(false);
                    self.cancel.set_sensitive(true);
                    self.create.set_sensitive(true);
                    warn!("Failed to create calendar: {e:?}");
                    // TODO: Show toast
                }
            }

            self.obj().close();
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
