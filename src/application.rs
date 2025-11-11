use std::cell::{Cell, OnceCell};

use adw::{prelude::*, subclass::prelude::*};
use clepsydre::{EdsManager, Manager, jiff};
use gettextrs::gettext;

use crate::{
    config::{APP_ID, APP_NAME, BASE_RESOURCE_PATH, VERSION},
    widgets::Window,
};

mod imp {
    use super::*;

    #[derive(Debug, Default, glib::Properties)]
    #[properties(wrapper_type = super::Application)]
    pub struct Application {
        // TODO: Monitor the system to update those
        // TODO: Use i16 for year ; this requires support from gtk-rs
        #[property(get, set)]
        current_year: Cell<i32>,
        #[property(get, set)]
        current_month: Cell<i8>,
        #[property(get, set)]
        current_day: Cell<i8>,
        #[property(get, set)]
        manager: OnceCell<Manager>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Application {
        const NAME: &'static str = "CalendarManagerApplication";
        type Type = super::Application;
        type ParentType = adw::Application;
    }

    #[glib::derived_properties]
    impl ObjectImpl for Application {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            obj.setup_gactions();
            obj.set_accels_for_action("app.quit", &["<primary>q"]);

            self.manager.set(EdsManager::new().upcast()).unwrap();
        }
    }

    impl ApplicationImpl for Application {
        fn activate(&self) {
            let application = self.obj();
            let window = application.active_window().unwrap_or_else(|| {
                let window = Window::new(&*application);
                window.upcast()
            });

            window.present();
        }
    }

    impl GtkApplicationImpl for Application {}
    impl AdwApplicationImpl for Application {}
}

glib::wrapper! {
    pub struct Application(ObjectSubclass<imp::Application>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl Application {
    pub fn new(flags: &gio::ApplicationFlags) -> Self {
        let now = jiff::Zoned::now();
        let current_year = now.year();
        let current_month = now.month();
        let current_day = now.day();

        glib::Object::builder()
            .property("application-id", APP_ID)
            .property("flags", flags)
            .property("resource-base-path", BASE_RESOURCE_PATH)
            .property("current-year", current_year as i32)
            .property("current-month", current_month)
            .property("current-day", current_day)
            .build()
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
            .application_name(APP_NAME)
            .application_icon(APP_ID)
            .developer_name("Titouan Real")
            .version(VERSION)
            .developers(vec!["Titouan Real"])
            .designers(vec!["Philipp Sauberz"])
            // Translators: Replace "translator-credits" with your name/username, and optionally an email or URL.
            .translator_credits(gettext("translator-credits"))
            .website("https://gitlab.gnome.org/TitouanReal/kalendasom")
            .issue_url("https://gitlab.gnome.org/TitouanReal/kalendasom/-/issues")
            .license_type(gtk::License::Gpl30)
            .copyright("© 2025 Titouan Real")
            .build();

        about.present(Some(&window));
    }
}

impl Default for Application {
    fn default() -> Self {
        gio::Application::default()
            .and_downcast()
            .expect("Application should always be available")
    }
}
