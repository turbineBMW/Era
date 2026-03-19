use std::cell::{OnceCell, RefCell};

use adw::{prelude::*, subclass::prelude::*};
use clepsydre::Manager;
use gettextrs::gettext;
use gio::DBusConnection;
use glib::{DateTime, TimeZone, clone};

use crate::{
    config::{APP_ID, APP_NAME, BASE_RESOURCE_PATH, VERSION},
    system_settings::SystemSettings,
    widgets::Window,
};

mod imp {
    use super::*;

    #[derive(Debug, glib::Properties)]
    #[properties(wrapper_type = super::Application)]
    pub struct Application {
        #[property(get, set)]
        system_settings: OnceCell<SystemSettings>,
        #[property(get, set)]
        current_datetime: RefCell<DateTime>,
        #[property(get, set)]
        manager: OnceCell<Manager>,
        system_bus: OnceCell<DBusConnection>,
    }

    impl Default for Application {
        fn default() -> Self {
            Self {
                system_settings: OnceCell::default(),
                current_datetime: RefCell::new(
                    DateTime::new(&TimeZone::utc(), 1, 1, 1, 0, 0, 0.).unwrap(),
                ),
                manager: OnceCell::default(),
                system_bus: OnceCell::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Application {
        const NAME: &'static str = "KalendasomApplication";
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

            self.system_settings
                .set(SystemSettings::new())
                .expect("System settings should not already be initialized");
            self.manager
                .set(clepsydre_eds::Manager::new().upcast())
                .expect("Manager should not already be initialized");

            let conn = gio::bus_get_sync(gio::BusType::System, gio::Cancellable::NONE)
                .expect("Failed to connect to system D-Bus");
            self.system_bus
                .set(conn)
                .expect("System bus should not already be initialized");

            self.update_datetime();
            self.start_clock();
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

    impl Application {
        /// Reads the system timezone from org.freedesktop.timedate1.
        fn read_system_timezone(&self) -> TimeZone {
            let connection = self
                .system_bus
                .get()
                .expect("System bus should be initialized");

            let result = connection.call_sync(
                Some("org.freedesktop.timedate1"),
                "/org/freedesktop/timedate1",
                "org.freedesktop.DBus.Properties",
                "Get",
                Some(&glib::Variant::from((
                    "org.freedesktop.timedate1",
                    "Timezone",
                ))),
                Some(glib::VariantTy::new("(v)").expect("Variant Type should be valid")),
                gio::DBusCallFlags::NONE,
                -1,
                gio::Cancellable::NONE,
            );

            let iana_name = result
                .expect("Failed to read Timezone from timedate1")
                .child_value(0)
                .as_variant()
                .expect("Variant should contain another variant")
                .get::<String>()
                .expect("Variant should contain a string");

            TimeZone::from_identifier(Some(&iana_name)).expect("TimeZone should exist")
        }

        /// Returns the current time in the system timezone.
        fn now(&self) -> DateTime {
            let tz = self.read_system_timezone();
            DateTime::now(&tz).expect("Now should exist in any timezone")
        }

        /// Updates the `current-datetime` property from the system clock.
        fn update_datetime(&self) {
            let new_dt = self.now();

            let old = self.obj().current_datetime();
            let changed = old.year() != new_dt.year()
                || old.month() != new_dt.month()
                || old.day_of_month() != new_dt.day_of_month()
                || old.hour() != new_dt.hour()
                || old.minute() != new_dt.minute();

            if changed {
                self.obj().set_current_datetime(new_dt);
            }
        }

        /// Starts a task that runs every second to keep `current-datetime` in sync with the system
        /// clock.
        fn start_clock(&self) {
            glib::timeout_add_seconds_local(
                1,
                clone!(
                    #[weak(rename_to = imp)]
                    self,
                    #[upgrade_or]
                    glib::ControlFlow::Break,
                    move || {
                        imp.update_datetime();
                        glib::ControlFlow::Continue
                    }
                ),
            );
        }
    }
}

glib::wrapper! {
    pub struct Application(ObjectSubclass<imp::Application>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl Application {
    pub fn new(flags: &gio::ApplicationFlags) -> Self {
        glib::Object::builder()
            .property("application-id", APP_ID)
            .property("flags", flags)
            .property("resource-base-path", BASE_RESOURCE_PATH)
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
            .designers(vec!["Philipp Sauberzweig"])
            // Translators: Replace "translator-credits" with your name/username, and optionally an
            // email or URL.
            .translator_credits(gettext("translator-credits"))
            .website("https://gitlab.gnome.org/TitouanReal/kalendasom")
            .issue_url("https://gitlab.gnome.org/TitouanReal/kalendasom/-/issues")
            .license_type(gtk::License::Gpl30)
            .copyright("© 2026 Titouan Real")
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
