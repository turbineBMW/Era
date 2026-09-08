use std::cell::{Cell, OnceCell};

use adw::{prelude::*, subclass::prelude::*};
use clepsydre::Manager;
use gettextrs::gettext;

use crate::{
    config::{APP_ID, APP_NAME, BASE_RESOURCE_PATH, VERSION},
    system::System,
    widgets::Window,
};

mod imp {
    use super::*;

    #[derive(Debug, glib::Properties)]
    #[properties(wrapper_type = super::Application)]
    pub struct Application {
        #[property(get, set)]
        system: OnceCell<System>,
        #[property(get, set)]
        manager: OnceCell<Manager>,
        #[property(get, set)]
        debug: Cell<bool>,
    }

    impl Default for Application {
        fn default() -> Self {
            Self {
                system: OnceCell::default(),
                manager: OnceCell::default(),
                debug: Cell::new(false),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Application {
        const NAME: &'static str = "EraApplication";
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

            #[cfg(feature = "backend-android")]
            let manager = clepsydre_android::AndroidManager::new().upcast();
            #[cfg(feature = "backend-eds")]
            let manager = clepsydre_eds::Manager::new().upcast();
            #[cfg(feature = "backend-mock")]
            let manager = clepsydre_mock::MockManager::new().upcast();
            #[cfg(feature = "backend-p2panda")]
            let manager = clepsydre_p2panda::P2pandaManager::new().upcast();

            self.system
                .set(System::new())
                .expect("System should not already be initialized");
            self.manager
                .set(manager)
                .expect("Manager should not already be initialized");
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

    impl Application {}
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
            .artists(vec!["Hylke Bons https://planetpeanut.studio"])
            // Translators: Replace "translator-credits" with your name/username, and optionally an
            // email or URL.
            .translator_credits(gettext("translator-credits"))
            .website("https://gitlab.gnome.org/TitouanReal/era")
            .issue_url("https://gitlab.gnome.org/TitouanReal/era/-/issues")
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
