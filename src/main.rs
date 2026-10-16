use gettextrs::{bind_textdomain_codeset, bindtextdomain, textdomain};
use gtk::prelude::*;
use tracing_subscriber::{EnvFilter, prelude::*};

mod application;
mod config;
mod resources;
mod system;
mod utils;
mod widgets;

use self::{
    application::Application,
    config::{APP_NAME, GETTEXT_PACKAGE, LOCALEDIR, PROJECT_NAME},
    resources::GRESOURCE_BYTES,
};

// Verify Clepsydre Backend
#[cfg(not(any(
    feature = "backend-android",
    feature = "backend-eds",
    feature = "backend-mock",
    feature = "backend-p2panda",
)))]
compile_error!(
    "You must enable EXACTLY ONE backend feature: `backend-android`, `backend-eds`, `backend-mock`, or `backend-p2panda`."
);

#[cfg(any(
    all(feature = "backend-eds", feature = "backend-mock"),
    all(feature = "backend-eds", feature = "backend-p2panda"),
    all(feature = "backend-eds", feature = "backend-android"),
    all(feature = "backend-mock", feature = "backend-p2panda"),
    all(feature = "backend-mock", feature = "backend-android"),
    all(feature = "backend-p2panda", feature = "backend-android"),
))]
compile_error!(
    "Multiple backend features enabled! Choose only ONE: `backend-eds`, `backend-mock`, `backend-p2panda`, or `backend-android`."
);

// Verify Platform
#[cfg(not(any(feature = "platform-flatpak", feature = "platform-android")))]
compile_error!(
    "You must enable EXACTLY ONE platform feature: `platform-flatpak` or `platform-android`."
);

#[cfg(all(feature = "platform-flatpak", feature = "platform-android"))]
compile_error!(
    "Multiple platform features enabled! Choose only ONE: `platform-flatpak` or `platform-android`."
);

fn make_application() -> gio::Application {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new(format!(
            "{PROJECT_NAME}=info,clepsydre=info,clepsydre-eds=info,clepsydre-mock=info,clepsydre-p2panda=info,warn"
        ))
    });

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_filter(env_filter))
        .init();

    glib::log_set_writer_func(|level, fields| {
        let level = match level {
            glib::LogLevel::Error | glib::LogLevel::Critical => tracing_log::log::Level::Error,
            glib::LogLevel::Warning => tracing_log::log::Level::Warn,
            glib::LogLevel::Message | glib::LogLevel::Info => tracing_log::log::Level::Info,
            glib::LogLevel::Debug => tracing_log::log::Level::Debug,
        };

        let domain = fields
            .iter()
            .find(|field| field.key() == "GLIB_DOMAIN")
            .unwrap()
            .value_str()
            .unwrap();
        let message = fields
            .iter()
            .find(|field| field.key() == "MESSAGE")
            .unwrap()
            .value_str()
            .unwrap();

        tracing_log::log::log!(target: domain, level, "{message}");

        glib::LogWriterOutput::Handled
    });

    glib::set_application_name(APP_NAME);

    bindtextdomain(GETTEXT_PACKAGE, LOCALEDIR).expect("Unable to bind the text domain");
    bind_textdomain_codeset(GETTEXT_PACKAGE, "UTF-8")
        .expect("Unable to set the text domain encoding");
    textdomain(GETTEXT_PACKAGE).expect("Unable to switch to the text domain");

    let resources = gio::Resource::from_data(&glib::Bytes::from_static(GRESOURCE_BYTES))
        .expect("Could not load resources");
    gio::resources_register(&resources);

    let application = Application::new(&gio::ApplicationFlags::empty());

    application.upcast()
}

#[cfg(not(target_os = "android"))]
fn main() -> glib::ExitCode {
    let application = make_application();
    application.run()
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "C" fn main(argc: i32, argv: *mut *mut std::ffi::c_char) -> i32 {
    let application = make_application();
    unsafe { gio::ffi::g_application_run(application.as_ptr(), argc, argv) }
}
