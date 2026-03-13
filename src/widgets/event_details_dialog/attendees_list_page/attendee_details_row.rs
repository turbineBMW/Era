use std::cell::RefCell;

use adw::{prelude::*, subclass::prelude::*};
use clepsydre::Attendee;

use crate::utils::TemplateCallbacks;

use super::attendee_role_badge::AttendeeRoleBadge;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Kalendasom/attendee_details_row.ui")]
    #[properties(wrapper_type = super::AttendeeDetailsRow)]
    pub struct AttendeeDetailsRow {
        #[property(get, set, nullable)]
        attendee: RefCell<Option<Attendee>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AttendeeDetailsRow {
        const NAME: &'static str = "AttendeeDetailsRow";
        type Type = super::AttendeeDetailsRow;
        type ParentType = adw::PreferencesRow;

        fn class_init(klass: &mut Self::Class) {
            AttendeeRoleBadge::ensure_type();

            klass.bind_template();
            klass.bind_template_callbacks();
            TemplateCallbacks::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for AttendeeDetailsRow {}
    impl WidgetImpl for AttendeeDetailsRow {}
    impl ListBoxRowImpl for AttendeeDetailsRow {}
    impl PreferencesRowImpl for AttendeeDetailsRow {}

    #[gtk::template_callbacks]
    impl AttendeeDetailsRow {}
}

glib::wrapper! {
    pub struct AttendeeDetailsRow(ObjectSubclass<imp::AttendeeDetailsRow>)
        @extends gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Actionable;
}

impl AttendeeDetailsRow {
    pub fn new(attendee: Option<&Attendee>) -> Self {
        glib::Object::builder()
            .property("attendee", attendee)
            .build()
    }
}
