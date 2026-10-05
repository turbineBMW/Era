mod calendar_management_dialog;
mod components;
mod event_creation_dialog;
mod event_details_dialog;
#[cfg(feature = "phone")]
mod phone; // fork: the phone build
mod qr_code_dialog;
mod search_dialog;
mod sidebar;
mod views;
mod window;

pub use self::{
    calendar_management_dialog::CalendarManagementDialog,
    event_creation_dialog::EventCreationDialog, qr_code_dialog::QrCodeDialog,
    search_dialog::SearchDialog, sidebar::Sidebar, window::Window,
};
