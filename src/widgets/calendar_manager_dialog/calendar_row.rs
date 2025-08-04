use std::cell::RefCell;

use adw::{prelude::*, subclass::prelude::*};
use clepsydre::Calendar;
use gtk::{
    gdk::{Paintable, RGBA},
    glib,
};

use crate::utils::get_circle_paintable_from_color;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Kalendasom/calendar_row.ui")]
    #[properties(wrapper_type = super::CalendarRow)]
    pub struct CalendarRow {
        #[property(get, set, construct_only)]
        pub calendar: RefCell<Option<Calendar>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CalendarRow {
        const NAME: &'static str = "CalendarRow";
        type Type = super::CalendarRow;
        type ParentType = adw::ActionRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for CalendarRow {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }
    impl WidgetImpl for CalendarRow {}
    impl ListBoxRowImpl for CalendarRow {}
    impl PreferencesRowImpl for CalendarRow {}
    impl ActionRowImpl for CalendarRow {}

    #[gtk::template_callbacks]
    impl CalendarRow {
        /// Show the session subpage.
        #[template_callback]
        fn show_calendar_subpage(&self) {
            let obj = self.obj();

            let _ = obj.activate_action(
                "calendar-manager.show-calendar-subpage",
                Some(
                    &obj.calendar()
                        .expect("Calendar should be initialized")
                        .uri()
                        .to_variant(),
                ),
            );
        }

        /// Toggle the visibility of the calendar.
        #[template_callback]
        fn toggle_calendar_visible(&self) {
            dbg!("todo");
        }

        #[template_callback]
        fn get_color_image(&self, color: RGBA) -> Paintable {
            get_circle_paintable_from_color(&color, 16.)
        }
    }
}

glib::wrapper! {
    pub struct CalendarRow(ObjectSubclass<imp::CalendarRow>)
    @extends gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow, adw::ActionRow,
    @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl CalendarRow {
    pub fn new(calendar: &Calendar) -> Self {
        glib::Object::builder()
            .property("calendar", calendar)
            .build()
    }
}
