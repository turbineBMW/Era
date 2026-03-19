use std::cell::{Cell, RefCell};

use adw::{prelude::*, subclass::prelude::*};
use clepsydre::{Attendee, Event};

use crate::utils::{AttendeeTypeFilter, AttendeeTypeSelection, TemplateCallbacks};

mod attendee_details_row;
mod attendee_role_badge;

use self::attendee_details_row::AttendeeDetailsRow;

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
        search_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        accepted_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        tentative_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        declined_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        needs_action_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        delegated_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        attendee_sorter: TemplateChild<gtk::CustomSorter>,
        #[template_child]
        accepted_model: TemplateChild<gtk::FilterListModel>,
        #[template_child]
        tentative_model: TemplateChild<gtk::FilterListModel>,
        #[template_child]
        declined_model: TemplateChild<gtk::FilterListModel>,
        #[template_child]
        needs_action_model: TemplateChild<gtk::FilterListModel>,
        #[template_child]
        delegated_model: TemplateChild<gtk::FilterListModel>,
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

            klass.install_action("attendees-list-page.focus-search", None, |obj, _, _| {
                obj.imp().search_entry.grab_focus();
            });
            klass.add_binding_action(
                gdk::Key::F,
                gdk::ModifierType::CONTROL_MASK,
                "attendees-list-page.focus-search",
            );
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

                left_name.cmp(&right_name).into()
            });

            self.accepted_group
                .bind_model(Some(&*self.accepted_model), Self::attendee_row);
            self.tentative_group
                .bind_model(Some(&*self.tentative_model), Self::attendee_row);
            self.declined_group
                .bind_model(Some(&*self.declined_model), Self::attendee_row);
            self.needs_action_group
                .bind_model(Some(&*self.needs_action_model), Self::attendee_row);
            self.delegated_group
                .bind_model(Some(&*self.delegated_model), Self::attendee_row);
        }
    }

    impl WidgetImpl for AttendeesListPage {}
    impl NavigationPageImpl for AttendeesListPage {}

    #[gtk::template_callbacks]
    impl AttendeesListPage {
        fn attendee_row(attendee: &glib::Object) -> gtk::Widget {
            let attendee = attendee
                .downcast_ref::<Attendee>()
                .expect("item should be an attendee");
            AttendeeDetailsRow::new(Some(attendee)).upcast()
        }
    }
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
