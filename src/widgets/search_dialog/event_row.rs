use std::cell::RefCell;

use adw::{prelude::*, subclass::prelude::*};
use clepsydre::{Event, Timeframe};
use gdk::{Paintable, RGBA};

use crate::utils::get_horizontal_bar_paintable_from_color;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Kalendasom/event_row.ui")]
    #[properties(wrapper_type = super::EventRow)]
    pub struct EventRow {
        #[property(get, set)]
        event: RefCell<Option<Event>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for EventRow {
        const NAME: &'static str = "EventRow";
        type Type = super::EventRow;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Timeframe::ensure_type();

            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for EventRow {}
    impl WidgetImpl for EventRow {}
    impl BoxImpl for EventRow {}

    #[gtk::template_callbacks]
    impl EventRow {
        #[template_callback]
        fn get_color_image(&self, color: RGBA) -> Paintable {
            get_horizontal_bar_paintable_from_color(&color, 6., 48.)
        }

        #[template_callback]
        fn get_start_time(&self) -> String {
            let Some(event) = self.obj().event() else {
                return String::new();
            };
            format!(
                "{}",
                &event
                    .timeframe()
                    .expect("Event should have a timeframe")
                    .start()
                    .unwrap()
                    .format_iso8601()
                    .unwrap()
            )
        }

        #[template_callback]
        fn get_end_time(&self) -> String {
            let Some(event) = self.obj().event() else {
                return String::new();
            };
            format!(
                "{}",
                &event
                    .timeframe()
                    .expect("Event should have a timeframe")
                    .end()
                    .unwrap()
                    .format_iso8601()
                    .unwrap()
            )
        }
    }
}

glib::wrapper! {
    pub struct EventRow(ObjectSubclass<imp::EventRow>)
    @extends gtk::Widget, gtk::Box,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl EventRow {
    pub fn new(event: &Event) -> Self {
        glib::Object::builder().property("event", event).build()
    }
}
