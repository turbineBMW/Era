use std::{
    cell::{Cell, RefCell},
    sync::LazyLock,
};

use adw::{prelude::*, subclass::prelude::*};
use clepsydre::{Event, prelude::*};
use glib::{clone, closure_local, subclass::Signal};

use crate::utils::{AttendeeTypeFilter, AttendeeTypeSelection, TemplateCallbacks};

mod attendee_row;
mod attendee_status_section;

use self::attendee_status_section::AttendeeStatusSection;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Kalendasom/attendee_list_row.ui")]
    #[properties(wrapper_type = super::AttendeeListRow)]
    pub struct AttendeeListRow {
        #[property(get, set, nullable)]
        event: RefCell<Option<Event>>,
        #[property(get, set)]
        attendee_type_selection: Cell<AttendeeTypeSelection>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AttendeeListRow {
        const NAME: &'static str = "AttendeeListRow";
        type Type = super::AttendeeListRow;
        type ParentType = adw::PreferencesRow;

        fn class_init(klass: &mut Self::Class) {
            AttendeeStatusSection::ensure_type();

            klass.bind_template();
            klass.bind_template_callbacks();
            TemplateCallbacks::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for AttendeeListRow {
        fn signals() -> &'static [Signal] {
            static SIGNALS: LazyLock<Vec<Signal>> =
                LazyLock::new(|| vec![Signal::builder("activated").build()]);
            SIGNALS.as_ref()
        }

        fn constructed(&self) {
            self.parent_constructed();

            self.obj().connect_parent_notify(|obj| {
                if let Some(listbox) = obj.parent().and_downcast_ref::<gtk::ListBox>() {
                    listbox.connect_row_activated(clone!(
                        #[weak]
                        obj,
                        move |_, row| {
                            if *row == obj {
                                obj.emit_by_name::<()>("activated", &[]);
                            }
                        }
                    ));
                }
            });
        }
    }

    impl WidgetImpl for AttendeeListRow {}
    impl ListBoxRowImpl for AttendeeListRow {}
    impl PreferencesRowImpl for AttendeeListRow {}

    #[gtk::template_callbacks]
    impl AttendeeListRow {
        #[template_callback]
        fn number_of_attendees(&self) -> String {
            let Some(event) = self.obj().event() else {
                return String::new();
            };

            let Some(attendees) = event.attendees() else {
                return "0".to_string();
            };

            let filter = AttendeeTypeFilter::new(self.obj().attendee_type_selection());
            let filtered_attendees = gtk::FilterListModel::new(Some(attendees), Some(filter));

            format!("{}", filtered_attendees.n_items())
        }
    }
}

glib::wrapper! {
    pub struct AttendeeListRow(ObjectSubclass<imp::AttendeeListRow>)
        @extends gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Actionable;
}

impl AttendeeListRow {
    pub fn new(event: &Event, attendee_type_selection: AttendeeTypeSelection) -> Self {
        glib::Object::builder()
            .property("event", event)
            .property("attendee-type-selection", attendee_type_selection)
            .build()
    }

    /// Connect to the signal emitted when the row is activated.
    pub fn connect_activated<F: Fn(&Self) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            "activated",
            true,
            closure_local!(move |obj: Self| {
                f(&obj);
            }),
        )
    }
}
