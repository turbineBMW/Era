use std::cell::Cell;

use adw::{prelude::*, subclass::prelude::*};
use clepsydre::{Attendee, ParticipationStatus};

// TODO: upstream this in some way, either to Clepsydre or as a macro in gtk-rs.
// This has a dependency on GTK though

#[glib::flags(name = "ParticipationStatusSelection")]
pub enum ParticipationStatusSelection {
    NeedsAction = (1 << 0),
    Accepted = (1 << 1),
    Declined = (1 << 2),
    Tentative = (1 << 3),
    Delegated = (1 << 4),
}

impl Default for ParticipationStatusSelection {
    fn default() -> Self {
        Self::empty()
    }
}

mod imp {
    use super::*;

    // TODO: Add an `expression` property to get the attendee_type from the item?
    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::ParticipationStatusFilter)]
    pub struct ParticipationStatusFilter {
        #[property(get, set = Self::set_participation_status_selection)]
        participation_status_selection: Cell<ParticipationStatusSelection>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ParticipationStatusFilter {
        const NAME: &'static str = "ParticipationStatusFilter";
        type Type = super::ParticipationStatusFilter;
        type ParentType = gtk::Filter;
    }

    #[glib::derived_properties]
    impl ObjectImpl for ParticipationStatusFilter {}

    impl FilterImpl for ParticipationStatusFilter {
        fn match_(&self, item: &glib::Object) -> bool {
            let Some(attendee) = item.downcast_ref::<Attendee>() else {
                return false;
            };

            let selection = self.obj().participation_status_selection();

            match attendee.participation_status() {
                ParticipationStatus::NeedsAction => {
                    selection.contains(ParticipationStatusSelection::NeedsAction)
                }
                ParticipationStatus::Accepted => {
                    selection.contains(ParticipationStatusSelection::Accepted)
                }
                ParticipationStatus::Declined => {
                    selection.contains(ParticipationStatusSelection::Declined)
                }
                ParticipationStatus::Tentative => {
                    selection.contains(ParticipationStatusSelection::Tentative)
                }
                ParticipationStatus::Delegated => {
                    selection.contains(ParticipationStatusSelection::Delegated)
                }
                _ => false,
            }
        }

        fn strictness(&self) -> gtk::FilterMatch {
            let selection = self.obj().participation_status_selection();

            if selection.is_empty() {
                gtk::FilterMatch::None
            } else if selection.is_all() {
                gtk::FilterMatch::All
            } else {
                gtk::FilterMatch::Some
            }
        }
    }

    impl ParticipationStatusFilter {
        fn set_participation_status_selection(
            &self,
            participation_status_selection: ParticipationStatusSelection,
        ) {
            let old_selection = self.participation_status_selection.get();
            if old_selection == participation_status_selection {
                return;
            }

            self.participation_status_selection
                .set(participation_status_selection);

            self.obj().notify_participation_status_selection();

            // TODO: What are the rewatch variants?
            if old_selection.contains(participation_status_selection) {
                self.obj()
                    .emit_by_name::<()>("changed", &[&gtk::FilterChange::LessStrict]);
            } else if participation_status_selection.contains(old_selection) {
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
    pub struct ParticipationStatusFilter(ObjectSubclass<imp::ParticipationStatusFilter>)
        @extends gtk::Filter;
}

impl ParticipationStatusFilter {
    pub fn new(participation_status_selection: ParticipationStatusSelection) -> Self {
        glib::Object::builder()
            .property(
                "participation-status-selection",
                participation_status_selection,
            )
            .build()
    }
}
