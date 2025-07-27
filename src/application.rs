use std::cell::Cell;

use adw::{prelude::*, subclass::prelude::*};
use ccm::{Manager, jiff};
use gettextrs::gettext;
use gtk::{gio, glib};

use crate::config::VERSION;
use crate::widgets::CalendarManagerWindow;

mod imp {
    use super::*;

    #[derive(Debug, Default, glib::Properties)]
    #[properties(wrapper_type = super::CalendarManagerApplication)]
    pub struct CalendarManagerApplication {
        // TODO: Monitor the system to update those
        #[property(get, set)]
        current_year: Cell<i32>,
        #[property(get, set)]
        current_month: Cell<i32>,
        #[property(get, set)]
        current_day: Cell<i32>,
        pub manager: Manager,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CalendarManagerApplication {
        const NAME: &'static str = "CalendarManagerApplication";
        type Type = super::CalendarManagerApplication;
        type ParentType = adw::Application;
    }

    #[glib::derived_properties]
    impl ObjectImpl for CalendarManagerApplication {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.setup_gactions();
            obj.set_accels_for_action("app.quit", &["<primary>q"]);
        }
    }

    impl ApplicationImpl for CalendarManagerApplication {
        fn activate(&self) {
            let application = self.obj();
            // Get the current window or create one if necessary
            let window = application.active_window().unwrap_or_else(|| {
                let window = CalendarManagerWindow::new(&*application);
                window.upcast()
            });

            window.present();
        }
    }

    impl GtkApplicationImpl for CalendarManagerApplication {}
    impl AdwApplicationImpl for CalendarManagerApplication {}
}

glib::wrapper! {
    pub struct CalendarManagerApplication(ObjectSubclass<imp::CalendarManagerApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl CalendarManagerApplication {
    pub fn new(application_id: &str, flags: &gio::ApplicationFlags) -> Self {
        let now = jiff::Zoned::now();
        let current_year = now.year();
        let current_month = now.month();
        let current_day = now.day();

        glib::Object::builder()
            .property("application-id", application_id)
            .property("flags", flags)
            .property(
                "resource-base-path",
                "/io/gitlab/TitouanReal/CalendarManager",
            )
            .property("current-year", current_year as i32)
            .property("current-month", current_month as i32)
            .property("current-day", current_day as i32)
            .build()
    }

    pub fn manager(&self) -> Manager {
        self.imp().manager.clone()
    }

    fn setup_gactions(&self) {
        let quit_action = gio::ActionEntry::builder("quit")
            .activate(move |app: &Self, _, _| app.quit())
            .build();
        let about_action = gio::ActionEntry::builder("about")
            .activate(move |app: &Self, _, _| app.show_about())
            .build();
        self.add_action_entries([quit_action, about_action]);
    }

    fn show_about(&self) {
        let window = self.active_window().unwrap();
        let about = adw::AboutDialog::builder()
            .application_name("calendar-manager")
            .application_icon("io.gitlab.TitouanReal.CalendarManager")
            .developer_name("Titouan Real")
            .version(VERSION)
            .developers(vec!["Titouan Real"])
            // Translators: Replace "translator-credits" with your name/username, and optionally an email or URL.
            .translator_credits(gettext("translator-credits"))
            .copyright("© 2025 Titouan Real")
            .build();

        about.present(Some(&window));
    }
}

impl Default for CalendarManagerApplication {
    fn default() -> Self {
        gio::Application::default()
            .and_downcast()
            .expect("Application should always be available")
    }
}
