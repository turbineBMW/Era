use std::cell::{Cell, RefCell};

use adw::subclass::prelude::*;
use clepsydre::{Attendee, Event, prelude::*};
use gtk::{FilterListModel, SortListModel, prelude::*};

use crate::utils::{
    AttendeeTypeFilter, AttendeeTypeSelection, ParticipationStatusFilter,
    ParticipationStatusSelection, TemplateCallbacks,
};

use super::attendee_row::AttendeeRow;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(
        file = "data/resources/ui/event_details_dialog/attendee_list_row/attendee_status_section.blp"
    )]
    #[properties(wrapper_type = super::AttendeeStatusSection)]
    pub struct AttendeeStatusSection {
        #[property(get, set = Self::set_event, nullable)]
        event: RefCell<Option<Event>>,
        #[property(get, set = Self::set_attendee_type_selection)]
        attendee_type_selection: Cell<AttendeeTypeSelection>,
        #[property(get, set = Self::set_participation_status_selection)]
        participation_status_selection: Cell<ParticipationStatusSelection>,
        #[property(get)]
        n_items: Cell<u32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AttendeeStatusSection {
        const NAME: &'static str = "AttendeeStatusSection";
        type Type = super::AttendeeStatusSection;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            TemplateCallbacks::bind_template_callbacks(klass);

            klass.set_css_name("attendee-status-section");
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for AttendeeStatusSection {}

    impl WidgetImpl for AttendeeStatusSection {}
    impl BoxImpl for AttendeeStatusSection {}

    impl AttendeeStatusSection {
        fn set_event(&self, event: Option<Event>) {
            if self.obj().event() == event {
                return;
            }

            self.event.replace(event);
            self.obj().notify_event();

            self.update_attendee_rows_and_compute_n_items();
        }

        fn set_attendee_type_selection(&self, attendee_type_selection: AttendeeTypeSelection) {
            if self.obj().attendee_type_selection() == attendee_type_selection {
                return;
            }

            self.attendee_type_selection.set(attendee_type_selection);
            self.obj().notify_attendee_type_selection();

            self.update_attendee_rows_and_compute_n_items();
        }

        fn set_participation_status_selection(
            &self,
            participation_status_selection: ParticipationStatusSelection,
        ) {
            if self.obj().participation_status_selection() == participation_status_selection {
                return;
            }

            self.participation_status_selection
                .set(participation_status_selection);
            self.obj().notify_participation_status_selection();

            self.update_attendee_rows_and_compute_n_items();
        }

        // TODO: Improve this
        fn update_attendee_rows_and_compute_n_items(&self) {
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }

            let Some(event) = self.obj().event() else {
                self.n_items.set(0);
                self.obj().notify_n_items();
                return;
            };

            let filter = gtk::EveryFilter::new();
            filter.append(AttendeeTypeFilter::new(
                self.obj().attendee_type_selection(),
            ));
            filter.append(ParticipationStatusFilter::new(
                self.obj().participation_status_selection(),
            ));

            let Some(attendees) = event.attendees() else {
                self.n_items.set(0);
                self.obj().notify_n_items();
                return;
            };

            let filtered = FilterListModel::new(Some(attendees), Some(filter));

            let sorter = gtk::StringSorter::new(Some(Attendee::this_expression("name")));
            let sorted = SortListModel::new(Some(filtered), Some(sorter));

            self.n_items.set(sorted.n_items());
            self.obj().notify_n_items();

            for i in 0..5 {
                let Some(attendee) = sorted.item(i) else {
                    break;
                };
                let attendee = attendee
                    .downcast::<Attendee>()
                    .expect("item should be an Attendee");
                self.obj().append(&AttendeeRow::new(&attendee));
            }

            if sorted.n_items() <= 6 {
                if let Some(attendee) = sorted.item(5) {
                    let attendee = attendee
                        .downcast::<Attendee>()
                        .expect("item should be an Attendee");
                    self.obj().append(&AttendeeRow::new(&attendee));
                }
            } else {
                let more = sorted.n_items() - 5;
                // TODO: Might need the context of this being more participants or resources, as eg
                // the gender can be useful for translations
                let label = gtk::Label::new(Some(&format!("{more} more…")));
                label.set_xalign(0.);
                label.set_margin_start(22);
                label.add_css_class("dimmed");
                self.obj().append(&label);
            }
        }
    }
}

glib::wrapper! {
    pub struct AttendeeStatusSection(ObjectSubclass<imp::AttendeeStatusSection>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}
