use std::cell::RefCell;

use adw::{prelude::*, subclass::prelude::*};
use clepsydre::Attendee;
use glib::clone;
use gtk::FilterListModel;

use super::attendee_row::AttendeeRow;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Kalendasom/participants_row.ui")]
    #[properties(wrapper_type = super::ParticipantsRow)]
    pub struct ParticipantsRow {
        #[property(get, set)]
        attendees: RefCell<Option<FilterListModel>>,
        #[template_child]
        n_attendees: TemplateChild<gtk::Label>,
        #[template_child]
        attendee_rows: TemplateChild<gtk::Box>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ParticipantsRow {
        const NAME: &'static str = "ParticipantsRow";
        type Type = super::ParticipantsRow;
        type ParentType = adw::PreferencesRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for ParticipantsRow {
        // TODO: Find a way to clean up this mess
        fn constructed(&self) {
            self.obj().connect_attendees_notify(clone!(
                #[weak(rename_to = imp)]
                self,
                move |_| {
                    imp.set_attendee_rows();
                    if let Some(attendees) = imp.obj().attendees() {
                        attendees.connect_items_changed(clone!(
                            #[weak]
                            imp,
                            move |_, _, _, _| {
                                imp.set_attendee_rows();
                            }
                        ));
                    }
                }
            ));
        }
    }

    impl WidgetImpl for ParticipantsRow {}
    impl ListBoxRowImpl for ParticipantsRow {}
    impl PreferencesRowImpl for ParticipantsRow {}

    #[gtk::template_callbacks]
    impl ParticipantsRow {
        fn set_attendee_rows(&self) {
            while let Some(child) = self.attendee_rows.first_child() {
                child.unparent();
            }

            let Some(attendees) = self.obj().attendees() else {
                return;
            };

            self.n_attendees.set_label(&attendees.n_items().to_string());

            for i in 0..5 {
                let Some(attendee) = attendees.item(i) else {
                    break;
                };
                let attendee = attendee
                    .downcast::<Attendee>()
                    .expect("Item should be an Attendee");
                let attendee_row = AttendeeRow::new(&attendee);
                self.attendee_rows.append(&attendee_row);
            }
            if attendees.n_items() <= 6 {
                if let Some(attendee) = attendees.item(5) {
                    let attendee = attendee
                        .downcast::<Attendee>()
                        .expect("Item should be an Attendee");
                    let attendee_row = AttendeeRow::new(&attendee);
                    self.attendee_rows.append(&attendee_row);
                }
            } else {
                let more = attendees.n_items() - 5;
                let label = gtk::Label::new(Some(&format!("{more} more…")));
                label.set_xalign(0.);
                label.set_margin_start(22);
                label.set_margin_top(3);
                label.add_css_class("dimmed");
                label.add_css_class("attendee-row");
                self.attendee_rows.append(&label);
            }
        }

        #[template_callback]
        fn attendee_item_bind(_factory: gtk::SignalListItemFactory, item: gtk::ListItem) {
            let attendee: Attendee = item
                .item()
                .expect("item should be bound")
                .downcast()
                .expect("item should be a Collection");
            let attendee_row = AttendeeRow::new(&attendee);
            item.set_child(Some(&attendee_row));
            item.set_activatable(false);
            item.set_focusable(false);
        }

        #[template_callback]
        fn open_details(&self) {
            tracing::warn!("here");
            let _ = self.obj().activate_action(
                "navigation.push",
                Some(&"participants-details".to_variant()),
            );
        }
    }
}

glib::wrapper! {
    pub struct ParticipantsRow(ObjectSubclass<imp::ParticipantsRow>)
        @extends gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Actionable;
}

impl ParticipantsRow {
    pub fn new(title: &str, attendees: FilterListModel) -> Self {
        glib::Object::builder()
            .property("title", title)
            .property("attendees", attendees)
            .build()
    }
}
