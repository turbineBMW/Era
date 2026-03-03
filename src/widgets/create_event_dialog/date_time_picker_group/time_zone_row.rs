use std::cell::RefCell;

use adw::{prelude::*, subclass::prelude::*};

use super::object_time_zone::ObjectTimeZone;

mod imp {

    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Kalendasom/time_zone_row.ui")]
    #[properties(wrapper_type = super::TimeZoneRow)]
    pub struct TimeZoneRow {
        #[property(get, construct_only)]
        time_zone: RefCell<Option<ObjectTimeZone>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TimeZoneRow {
        const NAME: &'static str = "TimeZoneRow";
        type Type = super::TimeZoneRow;
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
    impl ObjectImpl for TimeZoneRow {
        fn constructed(&self) {}
    }

    impl WidgetImpl for TimeZoneRow {}
    impl BoxImpl for TimeZoneRow {}

    #[gtk::template_callbacks]
    impl TimeZoneRow {}
}

glib::wrapper! {
    pub struct TimeZoneRow(ObjectSubclass<imp::TimeZoneRow>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl TimeZoneRow {
    pub fn new(time_zone: &ObjectTimeZone) -> Self {
        glib::Object::builder()
            .property("time-zone", time_zone)
            .build()
    }
}
