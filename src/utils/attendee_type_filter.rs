use std::cell::Cell;

use adw::{prelude::*, subclass::prelude::*};
use clepsydre::{Attendee, AttendeeType};

// TODO: upstream this in some way, either to Clepsydre or as a macro in gtk-rs.
// This has a dependency on GTK though

#[glib::flags(name = "AttendeeTypeSelection")]
pub enum AttendeeTypeSelection {
    Individual = (1 << 0),
    Group = (1 << 1),
    Resource = (1 << 2),
    Room = (1 << 3),
    Unknown = (1 << 4),
}

impl Default for AttendeeTypeSelection {
    fn default() -> Self {
        Self::empty()
    }
}

mod imp {
    use super::*;

    // TODO: Add an `expression` property to get the attendee_type from the item?
    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::AttendeeTypeFilter)]
    pub struct AttendeeTypeFilter {
        #[property(get, set = Self::set_attendee_type_selection)]
        attendee_type_selection: Cell<AttendeeTypeSelection>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AttendeeTypeFilter {
        const NAME: &'static str = "AttendeeTypeFilter";
        type Type = super::AttendeeTypeFilter;
        type ParentType = gtk::Filter;
    }

    #[glib::derived_properties]
    impl ObjectImpl for AttendeeTypeFilter {}

    impl FilterImpl for AttendeeTypeFilter {
        fn match_(&self, item: &glib::Object) -> bool {
            let Some(attendee) = item.downcast_ref::<Attendee>() else {
                return false;
            };

            let selection = self.attendee_type_selection.get();

            match attendee.attendee_type() {
                AttendeeType::Individual => selection.contains(AttendeeTypeSelection::Individual),
                AttendeeType::Group => selection.contains(AttendeeTypeSelection::Group),
                AttendeeType::Resource => selection.contains(AttendeeTypeSelection::Resource),
                AttendeeType::Room => selection.contains(AttendeeTypeSelection::Room),
                AttendeeType::Unknown => selection.contains(AttendeeTypeSelection::Unknown),
                _ => false,
            }
        }

        fn strictness(&self) -> gtk::FilterMatch {
            let selection = self.obj().attendee_type_selection();

            if selection.is_empty() {
                gtk::FilterMatch::None
            } else if selection.is_all() {
                gtk::FilterMatch::All
            } else {
                gtk::FilterMatch::Some
            }
        }
    }

    impl AttendeeTypeFilter {
        fn set_attendee_type_selection(&self, attendee_type_selection: AttendeeTypeSelection) {
            let old_selection = self.attendee_type_selection.get();
            if old_selection == attendee_type_selection {
                return;
            }

            self.attendee_type_selection.set(attendee_type_selection);

            self.obj().notify_attendee_type_selection();

            // TODO: What are the rewatch variants?
            if old_selection.contains(attendee_type_selection) {
                self.obj()
                    .emit_by_name::<()>("changed", &[&gtk::FilterChange::LessStrict]);
            } else if attendee_type_selection.contains(old_selection) {
                self.obj()
                    .emit_by_name::<()>("changed", &[&gtk::FilterChange::MoreStrict]);
            } else {
                self.obj()
                    .emit_by_name::<()>("changed", &[&gtk::FilterChange::Different]);
            }
        }
    }
}

glib::wrapper! {
    pub struct AttendeeTypeFilter(ObjectSubclass<imp::AttendeeTypeFilter>)
        @extends gtk::Filter;
}

impl AttendeeTypeFilter {
    pub fn new(attendee_type_selection: AttendeeTypeSelection) -> Self {
        glib::Object::builder()
            .property("attendee-type-selection", attendee_type_selection)
            .build()
    }
}
