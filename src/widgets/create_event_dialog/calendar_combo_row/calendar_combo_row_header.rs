use std::cell::RefCell;

use adw::{prelude::*, subclass::prelude::*};
use clepsydre::Collection;
use gtk::glib;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Kalendasom/calendar_combo_row_header.ui")]
    #[properties(wrapper_type = super::CalendarComboRowHeader)]
    pub struct CalendarComboRowHeader {
        #[property(get, set, construct_only)]
        pub collection: RefCell<Option<Collection>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CalendarComboRowHeader {
        const NAME: &'static str = "CalendarComboRowHeader";
        type Type = super::CalendarComboRowHeader;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for CalendarComboRowHeader {}
    impl WidgetImpl for CalendarComboRowHeader {}
    impl BoxImpl for CalendarComboRowHeader {}

    #[gtk::template_callbacks]
    impl CalendarComboRowHeader {}
}

glib::wrapper! {
    pub struct CalendarComboRowHeader(ObjectSubclass<imp::CalendarComboRowHeader>)
    @extends gtk::Widget, gtk::Box,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl CalendarComboRowHeader {
    pub fn new(collection: &Collection) -> Self {
        glib::Object::builder()
            .property("collection", collection)
            .build()
    }
}
