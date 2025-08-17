use gettextrs::{bind_textdomain_codeset, bindtextdomain, textdomain};
use gtk::prelude::*;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

mod application;
mod config;
mod utils;
mod widgets;

use self::{
    application::Application,
    config::{APP_NAME, GETTEXT_PACKAGE, LOCALEDIR, PROJECT_NAME, RESOURCES_FILE},
};

fn main() -> glib::ExitCode {
    // unsafe {
    //     std::env::set_var("GDK_DEBUG", "events");
    // }
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("{PROJECT_NAME}=debug,clepsydre=debug,warn")));

    tracing_subscriber::registry()
        .with(fmt::layer().with_filter(env_filter))
        .init();

    glib::set_application_name(APP_NAME);

    bindtextdomain(GETTEXT_PACKAGE, LOCALEDIR).expect("Unable to bind the text domain");
    bind_textdomain_codeset(GETTEXT_PACKAGE, "UTF-8")
        .expect("Unable to set the text domain encoding");
    textdomain(GETTEXT_PACKAGE).expect("Unable to switch to the text domain");

    let resources = gio::Resource::load(RESOURCES_FILE).expect("Could not load resources");
    gio::resources_register(&resources);

    let app = Application::new(&gio::ApplicationFlags::empty());

    app.run()
}
