use std::cell::{OnceCell, RefCell};

use adw::{prelude::*, subclass::prelude::*};
use glib::TimeZone;

mod imp {
    use super::*;

    #[derive(Debug, Default, glib::Properties)]
    #[properties(wrapper_type = super::ObjectTimeZone)]
    pub struct ObjectTimeZone {
        #[property(get, construct_only)]
        time_zone: OnceCell<TimeZone>,
        #[property(get, construct_only)]
        identifier: RefCell<Option<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ObjectTimeZone {
        const NAME: &'static str = "ObjectTimeZone";
        type Type = super::ObjectTimeZone;
        type ParentType = glib::Object;

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            unsafe {
                obj.as_ref()
                    .imp()
                    .identifier
                    .borrow_mut()
                    .replace(String::new())
            };
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for ObjectTimeZone {
        fn constructed(&self) {
            self.identifier.borrow_mut().replace(
                self.time_zone
                    .get()
                    .expect("TimeZone should be initialized")
                    .identifier()
                    .to_string(),
            );
        }
    }

    impl WidgetImpl for ObjectTimeZone {}
    impl AdwDialogImpl for ObjectTimeZone {}

    #[gtk::template_callbacks]
    impl ObjectTimeZone {}
}

glib::wrapper! {
    pub struct ObjectTimeZone(ObjectSubclass<imp::ObjectTimeZone>)
        @extends gtk::Widget, adw::Dialog,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::ShortcutManager;
}

impl ObjectTimeZone {
    pub fn new(time_zone: TimeZone) -> Self {
        glib::Object::builder()
            .property("time_zone", time_zone)
            .build()
    }
}
