use std::cell::{Cell, RefCell};
use std::cmp;

use adw::{prelude::*, subclass::prelude::*};
use clepsydre::{Attendee, Event, ParticipationStatus};

use crate::utils::{AttendeeTypeFilter, AttendeeTypeSelection, TemplateCallbacks};

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Kalendasom/attendees_list_page.ui")]
    #[properties(wrapper_type = super::AttendeeListPage)]
    pub struct AttendeesListPage {
        #[property(get, set, nullable)]
        event: RefCell<Option<Event>>,
        #[property(get, set)]
        attendee_type_selection: Cell<AttendeeTypeSelection>,
        #[template_child]
        attendee_sorter: TemplateChild<gtk::CustomSorter>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AttendeesListPage {
        const NAME: &'static str = "AttendeeDetailsPage";
        type Type = super::AttendeeListPage;
        type ParentType = adw::NavigationPage;

        fn class_init(klass: &mut Self::Class) {
            AttendeeTypeFilter::ensure_type();

            klass.bind_template();
            klass.bind_template_callbacks();
            TemplateCallbacks::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for AttendeesListPage {
        fn constructed(&self) {
            self.parent_constructed();

            self.attendee_sorter.set_sort_func(|left, right| {
                let left_attendee = left
                    .downcast_ref::<Attendee>()
                    .expect("Item should be an Attendee");
                let right_attendee = right
                    .downcast_ref::<Attendee>()
                    .expect("Item should be an Attendee");
                let left_name = left_attendee.name();
                let right_name = right_attendee.name();

                match (
                    left_attendee.participation_status(),
                    right_attendee.participation_status(),
                ) {
                    (left, right) if left == right => left_name.cmp(&right_name),
                    (ParticipationStatus::Accepted, _) => cmp::Ordering::Less,
                    (_, ParticipationStatus::Accepted) => cmp::Ordering::Greater,
                    (ParticipationStatus::Tentative, _) => cmp::Ordering::Less,
                    (_, ParticipationStatus::Tentative) => cmp::Ordering::Greater,
                    (ParticipationStatus::Declined, _) => cmp::Ordering::Less,
                    (_, ParticipationStatus::Declined) => cmp::Ordering::Greater,
                    _ => left_name.cmp(&right_name),
                }
                .into()
            });
        }
    }

    impl WidgetImpl for AttendeesListPage {}
    impl NavigationPageImpl for AttendeesListPage {}

    #[gtk::template_callbacks]
    impl AttendeesListPage {}
}

impl AttendeeListPage {
    pub fn new(event: &Option<Event>, attendee_type_selection: AttendeeTypeSelection) -> Self {
        glib::Object::builder()
            .property("event", event)
            .property("attendee-type-selection", attendee_type_selection)
            .build()
    }
}

glib::wrapper! {
    pub struct AttendeeListPage(ObjectSubclass<imp::AttendeesListPage>)
        @extends gtk::Widget, adw::NavigationPage,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}
