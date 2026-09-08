use std::cell::RefCell;

use adw::{prelude::*, subclass::prelude::*};
use clepsydre::Attendee;

use crate::utils::TemplateCallbacks;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(file = "data/resources/ui/event_details_dialog/attendee_list_row/attendee_row.blp")]
    #[properties(wrapper_type = super::AttendeeRow)]
    pub struct AttendeeRow {
        #[property(get, construct_only)]
        attendee: RefCell<Option<Attendee>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AttendeeRow {
        const NAME: &'static str = "AttendeeRow";
        type Type = super::AttendeeRow;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            TemplateCallbacks::bind_template_callbacks(klass);

            klass.set_css_name("attendee-row");
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for AttendeeRow {}
    impl WidgetImpl for AttendeeRow {}
    impl BoxImpl for AttendeeRow {}
}

glib::wrapper! {
    pub struct AttendeeRow(ObjectSubclass<imp::AttendeeRow>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl AttendeeRow {
    pub fn new(attendee: &Attendee) -> Self {
        glib::Object::builder()
            .property("attendee", attendee)
            .build()
    }
}
