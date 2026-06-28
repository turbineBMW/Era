use std::cell::RefCell;

use adw::{prelude::*, subclass::prelude::*};
use clepsydre::{Calendar, prelude::*};
use glib::{clone, translate::*};
use tracing::{debug, warn};

use crate::utils::{PaintableCallbacks, TemplateCallbacks};

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Era/sidebar_calendar_row.ui")]
    #[properties(wrapper_type = super::SidebarCalendarRow)]
    pub struct SidebarCalendarRow {
        #[property(get, set = Self::set_calendar, nullable)]
        calendar: RefCell<Option<Calendar>>,
        // #[template_child]
        // circle_loading: TemplateChild<gtk::Stack>,
        #[template_child]
        check: TemplateChild<gtk::CheckButton>,

        css_class: RefCell<Option<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SidebarCalendarRow {
        const NAME: &'static str = "SidebarCalendarRow";
        type Type = super::SidebarCalendarRow;
        type ParentType = gtk::Box;

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
    impl ObjectImpl for SidebarCalendarRow {}

    impl WidgetImpl for SidebarCalendarRow {}
    impl BoxImpl for SidebarCalendarRow {}

    #[gtk::template_callbacks]
    impl SidebarCalendarRow {
        fn set_calendar(&self, calendar: Option<Calendar>) {
            if self.obj().calendar() == calendar {
                return;
            }

            self.calendar.replace(calendar.clone());
            self.obj().notify_calendar();

            self.update_check_color();

            if let Some(calendar) = calendar {
                calendar.connect_color_notify(clone!(
                    #[weak(rename_to=imp)]
                    self,
                    move |_| {
                        imp.update_check_color();
                    }
                ));
            }
        }

        fn update_check_color(&self) {
            let obj = self.obj();

            if let Some(old_class) = self.css_class.borrow_mut().take() {
                self.check.remove_css_class(&old_class);
            }
            let Some(calendar) = obj.calendar() else {
                return;
            };

            let color = calendar.color().unwrap();

            let color_str = color.to_string();
            let color_id = glib::Quark::from_str(&color_str);
            let css_class = format!("color-{}", color_id.into_glib());

            self.check.add_css_class(&css_class);
            self.css_class.replace(Some(css_class));
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
            let visible = !calendar.is_visible();

            // self.circle_loading.set_visible_child_name("loading");
            // self.obj().set_activatable(false);

            match calendar.try_set_visible_future(visible).await {
                Ok(()) => {
                    debug!("Calendar visibility updated: {}", calendar.uri().unwrap());
                }
                Err(error) => {
                    warn!("Failed to update calendar visibility: {}", error);
                    let toast = adw::Toast::new("An error occurred");
                    toast.set_button_label(Some("Details"));
                    toast.set_action_name(Some("collections-list-page.show-error"));
                    toast.set_action_target(Some(&error.message()));

                    let Some(toast_overlay) = self.obj().ancestor(adw::ToastOverlay::static_type())
                    else {
                        // The dialog was closed by the user
                        return;
                    };
                    let toast_overlay = toast_overlay
                        .downcast::<adw::ToastOverlay>()
                        .expect("Ancestor should be a ToastOverlay");
                    toast_overlay.add_toast(toast);
                }
            }

            // self.circle_loading.set_visible_child_name("circle");
            // self.obj().set_activatable(true);
        }
    }
}

glib::wrapper! {
    pub struct SidebarCalendarRow(ObjectSubclass<imp::SidebarCalendarRow>)
    @extends gtk::Widget, gtk::Box,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl SidebarCalendarRow {
    pub fn new(calendar: &Calendar) -> Self {
        glib::Object::builder()
            .property("calendar", calendar)
            .build()
    }
}
