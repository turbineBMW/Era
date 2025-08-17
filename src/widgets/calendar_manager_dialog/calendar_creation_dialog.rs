use std::cell::RefCell;

use adw::{prelude::*, subclass::prelude::*};
use clepsydre::Collection;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Kalendasom/calendar_creation_dialog.ui")]
    #[properties(wrapper_type = super::CalendarCreationDialog)]
    pub struct CalendarCreationDialog {
        #[property(get, set, construct_only)]
        pub collection: RefCell<Option<Collection>>,
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
        fn create_calendar(&self) {
            self.obj()
                .collection()
                .expect("collection should be initialized")
                .create_calendar(&self.name.text(), self.color.rgba());
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
