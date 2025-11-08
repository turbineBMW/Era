mod calendar_manager_dialog;
mod components;
mod create_event_dialog;
mod event_details_dialog;
mod qr_code_dialog;
mod search_dialog;
mod views;
mod window;

pub use self::{
    calendar_manager_dialog::CalendarManagerDialog, create_event_dialog::CreateEventDialog,
    qr_code_dialog::QrCodeDialog, search_dialog::SearchDialog, window::Window,
};
