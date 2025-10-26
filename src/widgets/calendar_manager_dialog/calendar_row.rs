use std::cell::RefCell;

use adw::{prelude::*, subclass::prelude::*};
use clepsydre::Calendar;
use tracing::{debug, warn};

use crate::utils::{PaintableCallbacks, TemplateCallbacks};

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
            PaintableCallbacks::bind_template_callbacks(klass);
            TemplateCallbacks::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for CalendarRow {}
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
        async fn toggle_calendar_visible(&self) {
            let calendar = self
                .calendar
                .borrow()
                .as_ref()
                .expect("Calendar should be initialized")
                .clone();
            let visible = !calendar.visible();

            match calendar.try_set_visible_future(visible).await {
                Ok(()) => {
                    debug!("Calendar name updated: {}", calendar.uri());
                }
                Err(error) => {
                    warn!("Failed to update calendar name: {}", error);
                    let toast = adw::Toast::new("An error occurred");
                    toast.set_button_label(Some("Details"));
                    toast.set_action_name(Some("collections-list-page.show-error"));
                    toast.set_action_target(Some(&error.message()));

                    let toast_overlay = self
                        .obj()
                        .ancestor(adw::ToastOverlay::static_type())
                        .expect("Toast overlay should be present")
                        .downcast::<adw::ToastOverlay>()
                        .expect("Ancestor should be a ToastOverlay");
                    toast_overlay.add_toast(toast);
                }
            }
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
