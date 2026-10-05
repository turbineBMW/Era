use std::{
    ffi::{CString, c_char},
    ptr,
};

use gettextrs::{bind_textdomain_codeset, bindtextdomain, textdomain};
use glib::ffi::g_log_writer_default_set_debug_domains;
use gtk::prelude::*;
use tracing_subscriber::{EnvFilter, prelude::*};

mod application;
mod config;
mod omarchy; // fork: Omarchy theme support
mod preferences; // fork: settings popover
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
    // TODO: Debug - scrollwheel after scrolling down with pad is bugged
    unsafe {
        //     std::env::set_var("GDK_DEBUG", "events");
        std::env::set_var("RUST_BACKTRACE", "full");
    }
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new(format!(
            "{PROJECT_NAME}=trace,clepsydre=trace,clepsydre-eds=trace,clepsydre-mock=trace,clepsydre-p2panda=trace,warn"
        ))
    });

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_filter(env_filter))
        .init();

    {
        // Inside your init function:
        let domains = [
            PROJECT_NAME,
            "clepsydre",
            "clepsydre-eds",
            "clepsydre-mock",
            "clepsydre-p2panda",
        ];

        // 1. Convert &str to CString (adds \0)
        let c_strings: Vec<CString> = domains.iter().map(|&s| CString::new(s).unwrap()).collect();

        // 2. Create a list of raw pointers to those CStrings
        let mut ptrs: Vec<*const c_char> = c_strings.iter().map(|cs| cs.as_ptr()).collect();

        // 3. Add a null terminator at the end so C knows where to stop
        ptrs.push(ptr::null());

        unsafe {
            // 4. Pass the pointer to the start of the pointer array
            g_log_writer_default_set_debug_domains(ptrs.as_ptr());
        }
    }

    glib::set_application_name(APP_NAME);

    bindtextdomain(GETTEXT_PACKAGE, LOCALEDIR).expect("Unable to bind the text domain");
    bind_textdomain_codeset(GETTEXT_PACKAGE, "UTF-8")
        .expect("Unable to set the text domain encoding");
    textdomain(GETTEXT_PACKAGE).expect("Unable to switch to the text domain");

    let resources = gio::Resource::from_data(&glib::Bytes::from_static(GRESOURCE_BYTES))
        .expect("Could not load resources");
    gio::resources_register(&resources);

    let application = Application::new(&gio::ApplicationFlags::empty());
    application.connect_startup(|app| omarchy::install(app.upcast_ref())); // fork
    application.connect_startup(preferences::install); // fork

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
