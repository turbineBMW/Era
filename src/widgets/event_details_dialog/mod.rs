use std::cell::RefCell;

use adw::{prelude::*, subclass::prelude::*};
use ashpd::Uri;
use ashpd::desktop::{
    file_chooser::{FileFilter, SelectedFiles},
    open_uri::OpenFileRequest,
};
use clepsydre::Event;
use glib::clone;
use tracing::{debug, warn};

use crate::{
    utils::{AttendeeTypeFilter, PaintableCallbacks, TemplateCallbacks},
    widgets::{
        QrCodeDialog,
        components::{ErrorDialog, LoadingButton},
    },
};

mod attendee_list_row;
mod attendees_list_page;

use self::{attendee_list_row::AttendeeListRow, attendees_list_page::AttendeeListPage};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Kalendasom/event_details_dialog.ui")]
    #[properties(wrapper_type = super::EventDetailsDialog)]
    pub struct EventDetailsDialog {
        #[property(get, construct_only)]
        event: RefCell<Option<Event>>,
        #[template_child]
        navigation_view: TemplateChild<adw::NavigationView>,
        #[template_child]
        details_toast_overlay: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        edit: TemplateChild<gtk::Button>,
        #[template_child]
        remove: TemplateChild<LoadingButton>,
        #[template_child]
        editor_toast_overlay: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        save: TemplateChild<LoadingButton>,
        #[template_child]
        name_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        location_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        video_conference_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        description_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        qr_code_dialog: TemplateChild<QrCodeDialog>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for EventDetailsDialog {
        const NAME: &'static str = "EventDetailsDialog";
        type Type = super::EventDetailsDialog;
        type ParentType = adw::Dialog;

        fn class_init(klass: &mut Self::Class) {
            AttendeeListRow::ensure_type();
            AttendeeTypeFilter::ensure_type();
            AttendeeListPage::ensure_type();

            klass.bind_template();
            klass.bind_template_callbacks();
            TemplateCallbacks::bind_template_callbacks(klass);
            PaintableCallbacks::bind_template_callbacks(klass);

            klass.install_action(
                "event-details-dialog.show-error",
                Some(&String::static_variant_type()),
                |obj, _, param| {
                    let error_message = &param
                        .and_then(glib::Variant::get::<String>)
                        .expect("The parameter should be a string");
                    let dialog = ErrorDialog::new(error_message);
                    dialog.present(Some(obj));
                },
            );
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for EventDetailsDialog {
        fn constructed(&self) {
            let event = self.obj().event().unwrap();

            // TODO: Doesn't work if a subdialog is shown
            event.connect_removed(clone!(
                #[weak(rename_to = imp)]
                self,
                move |_| {
                    let _ = imp.obj().activate_action("window.close", None);
                }
            ));
        }
    }

    impl WidgetImpl for EventDetailsDialog {}
    impl AdwDialogImpl for EventDetailsDialog {}

    #[gtk::template_callbacks]
    impl EventDetailsDialog {
        #[template_callback]
        fn timeframe_label(&self) -> String {
            if let Some(event) = self.obj().event() {
                let start = event
                    .timeframe()
                    .unwrap()
                    .start()
                    .unwrap()
                    .format_iso8601()
                    .unwrap();
                let end = event
                    .timeframe()
                    .unwrap()
                    .end()
                    .unwrap()
                    .format_iso8601()
                    .unwrap();
                format!("{} - {}", start, end)
            } else {
                String::new()
            }
        }

        #[template_callback]
        fn location_or_video_conference(&self) -> bool {
            let Some(event) = self.obj().event() else {
                return false;
            };
            let Some(location) = event.location() else {
                return false;
            };
            let Some(video_conference) = event.video_conference() else {
                return false;
            };
            !location.is_empty() || !video_conference.is_empty()
        }

        #[template_callback]
        fn video_conference_is_a_uri(&self) -> bool {
            let Some(event) = self.obj().event() else {
                return false;
            };
            let Some(video_conference) = event.video_conference() else {
                return false;
            };
            url::Url::parse(&video_conference).is_ok()
        }

        #[template_callback]
        async fn share(&self) {
            let event = self.obj().event().expect("event should be initialized");
            let ics_content = event.to_string_for_ics().unwrap();

            let request = match SelectedFiles::save_file()
                .title("Export Event")
                .accept_label("Export")
                .current_name(format!("{}.ics", event.name().unwrap_or_default()).as_str())
                .modal(true)
                .filter(FileFilter::new("iCalendar").glob("*.ics"))
                .send()
                .await
            {
                Ok(request) => request,
                Err(e) => {
                    warn!("Failed to save file: {}", e);
                    return;
                }
            };

            let Ok(files) = request.response() else {
                return;
            };

            if let Some(file_uri) = files.uris().first() {
                let file = gio::File::for_uri(file_uri.as_str());
                file.replace_contents_future(
                    ics_content,
                    None,
                    false,
                    gio::FileCreateFlags::REPLACE_DESTINATION,
                )
                .await
                .unwrap();
            }
        }

        #[template_callback]
        fn show_qr_code(&self) {
            let event = self.obj().event().expect("event should be initialized");
            let url = event.to_string_for_qr_code().unwrap();

            self.qr_code_dialog.set_url(url);
            self.qr_code_dialog.present(Some(&*self.obj()));
        }

        #[template_callback]
        async fn open_map(&self) {
            let location = self
                .event
                .borrow()
                .clone()
                .expect("event should be set")
                .location()
                .expect("Map button should not be available if no location is set");
            // TODO: Use correct geo URI once ASHPD removes this limitation
            // See https://github.com/bilelmoussaoui/ashpd/issues/385
            // Use "geo://0,0?q={}" instead
            let uri = Uri::parse(&location)
                .or_else(|_error| Uri::parse(&format!("geo://0,0?q={}", location)))
                .expect("should be a geo URI");
            match OpenFileRequest::default().send_uri(&uri).await {
                Ok(_) => {
                    debug!("Map opened with URL: {uri}");
                }
                Err(error) => {
                    warn!("Failed to open map: {error}");
                    self.editor_toast_overlay.dismiss_all();
                    let toast = adw::Toast::new("An error occurred");
                    toast.set_button_label(Some("Details"));
                    toast.set_action_name(Some("event-details-dialog.show-error"));
                    toast.set_action_target(Some(&error.to_string()));
                    self.editor_toast_overlay.add_toast(toast);
                }
            }
        }

        #[template_callback]
        async fn join(&self) {
            let event = self.obj().event().expect("event should be initialized");
            let uri = Uri::parse(
                &event
                    .video_conference()
                    .expect("join should not be callable if video_conference is not a URI"),
            )
            .expect("join should not be callable if video_conference is not a URI");
            match OpenFileRequest::default().send_uri(&uri).await {
                Ok(_) => {}
                Err(err) => {
                    warn!("Failed to open file: {err}");
                    self.details_toast_overlay.dismiss_all();
                    let toast = adw::Toast::new("The video conference could not be opened");
                    self.details_toast_overlay.add_toast(toast);
                }
            }
        }

        #[template_callback]
        fn open_attendees_details(&self, row: &AttendeeListRow) {
            let attendees_list_page =
                AttendeeListPage::new(&self.obj().event(), row.attendee_type_selection());
            attendees_list_page.set_title(&row.title());
            self.navigation_view.push(&attendees_list_page);
        }

        #[template_callback]
        fn edit(&self) {
            let event = self.obj().event().expect("event should be initialized");

            self.navigation_view.push_by_tag("editor");

            self.name_entry.set_text(&event.name().unwrap());
            self.location_entry.set_text(&event.location().unwrap());
            self.video_conference_entry
                .set_text(&event.video_conference().unwrap());
            self.description_entry
                .set_text(&event.description().unwrap());
        }

        #[template_callback]
        async fn save(&self) {
            dbg!("todo");
        }

        #[template_callback]
        async fn remove(&self) {
            self.remove.set_is_loading(true);
            self.edit.set_sensitive(false);
            let event = self.obj().event().expect("event should be initialized");
            match event.try_remove_future().await {
                Ok(()) => {
                    debug!("Event removed: {}", event.uri().unwrap());
                }
                Err(error) => {
                    warn!("Failed to remove event: {}", error);
                    self.editor_toast_overlay.dismiss_all();
                    let toast = adw::Toast::new("An error occurred");
                    toast.set_button_label(Some("Details"));
                    toast.set_action_name(Some("event-details-dialog.show-error"));
                    toast.set_action_target(Some(&error.message()));
                    self.editor_toast_overlay.add_toast(toast);

                    self.remove.set_is_loading(false);
                }
            }
        }
    }
}

glib::wrapper! {
    pub struct EventDetailsDialog(ObjectSubclass<imp::EventDetailsDialog>)
        @extends gtk::Widget, adw::Dialog,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::ShortcutManager;
}

impl EventDetailsDialog {
    pub fn new(event: &Event) -> Self {
        glib::Object::builder().property("event", event).build()
    }
}
