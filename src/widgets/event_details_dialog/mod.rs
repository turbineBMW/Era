use std::cell::RefCell;

use adw::{prelude::*, subclass::prelude::*};
use ashpd::{
    Uri,
    desktop::{
        file_chooser::{FileFilter, SelectedFiles},
        open_uri::OpenFileRequest,
    },
};
use clepsydre::{Calendar, Event, prelude::*};
use gettextrs::gettext;
use glib::{DateTime, clone};
use tracing::{debug, warn};

use crate::{
    spawn,
    utils::{PaintableCallbacks, TemplateCallbacks},
    widgets::{
        QrCodeDialog,
        components::{ErrorDialog, LoadingButton, TimeframePicker},
    },
};

mod attendee_list_row;
mod attendees_list_page;

use self::{attendee_list_row::AttendeeListRow, attendees_list_page::AttendeeListPage};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Era/event_details_dialog.ui")]
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
        cancel: TemplateChild<gtk::Button>,
        #[template_child]
        save: TemplateChild<LoadingButton>,
        #[template_child]
        name_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        location_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        conference_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        timeframe_picker: TemplateChild<TimeframePicker>,
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

            klass.bind_template();
            klass.bind_template_callbacks();
            TemplateCallbacks::bind_template_callbacks(klass);
            PaintableCallbacks::bind_template_callbacks(klass);

            klass.set_css_name("event-details-dialog");

            klass.install_action("event-details-dialog.export", None, |obj, _, _| {
                let imp = obj.imp();
                spawn!(clone!(
                    #[weak]
                    imp,
                    async move {
                        imp.export().await;
                    }
                ));
            });

            klass.install_action("event-details-dialog.show-qr-code", None, |obj, _, _| {
                obj.imp().qr_code_dialog.present(Some(obj));
            });

            klass.install_action("event-details-dialog.save", None, |obj, _, _| {
                let imp = obj.imp();
                spawn!(clone!(
                    #[weak]
                    imp,
                    async move {
                        imp.update_event().await;
                    }
                ));
            });
            klass.add_binding_action(
                gdk::Key::S,
                gdk::ModifierType::CONTROL_MASK,
                "event-details-dialog.save",
            );

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

            self.name_entry.connect_changed(clone!(
                #[weak(rename_to = imp)]
                self,
                move |_| imp.update_save_action()
            ));
            self.timeframe_picker.connect_timeframe_notify(clone!(
                #[weak(rename_to = imp)]
                self,
                move |_| imp.update_save_action()
            ));

            self.update_save_action();
        }
    }

    impl WidgetImpl for EventDetailsDialog {}
    impl AdwDialogImpl for EventDetailsDialog {}

    #[gtk::template_callbacks]
    impl EventDetailsDialog {
        #[template_callback]
        fn timeframe_label(
            &self,
            all_day: bool,
            start: DateTime,
            end: DateTime,
            today: DateTime,
        ) -> String {
            let yesterday = today.add_days(-1).unwrap();
            let tomorrow = today.add_days(1).unwrap();

            if all_day {
                if start.add_days(1).unwrap() == end {
                    if start.year() == yesterday.year()
                        && start.month() == yesterday.month()
                        && start.day_of_month() == yesterday.day_of_month()
                    {
                        gettext("Yesterday")
                    } else if start.year() == today.year()
                        && start.month() == today.month()
                        && start.day_of_month() == today.day_of_month()
                    {
                        gettext("Today")
                    } else if start.year() == tomorrow.year()
                        && start.month() == tomorrow.month()
                        && start.day_of_month() == tomorrow.day_of_month()
                    {
                        gettext("Tomorrow")
                    } else {
                        start.format("%Y-%m-%d").unwrap().to_string()
                    }
                } else {
                    format!(
                        "{} - {}",
                        start.format("%Y-%m-%d").unwrap(),
                        end.add_days(-1).unwrap().format("%Y-%m-%d").unwrap(),
                    )
                }
            } else {
                format!(
                    "{} - {}",
                    start.format_iso8601().unwrap(),
                    end.format_iso8601().unwrap()
                )
            }
        }

        #[template_callback]
        fn conference_is_a_uri(&self, conference: &str) -> bool {
            url::Url::parse(conference).is_ok()
        }

        async fn export(&self) {
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
        async fn open_map(&self) {
            let location = self
                .event
                .borrow()
                .clone()
                .expect("event should be set")
                .location()
                .expect("Map button should not be available if no location is set");
            let uri = Uri::parse(&location)
                .or_else(|_error| Uri::parse(&format!("geo:0,0?q={}", location)))
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
                    .conference()
                    .expect("join should not be callable if conference is not a URI"),
            )
            .expect("join should not be callable if conference is not a URI");
            match OpenFileRequest::default().send_uri(&uri).await {
                Ok(_) => {}
                Err(err) => {
                    warn!("Failed to open file: {err}");
                    self.details_toast_overlay.dismiss_all();
                    let toast = adw::Toast::new("The conference could not be opened");
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

        fn update_save_action(&self) {
            let name = self.name_entry.text();
            let timeframe = self.timeframe_picker.timeframe();

            let name_is_empty = name.trim().is_empty();
            let timeframe_is_invalid = timeframe.is_none();

            let is_invalid = name_is_empty || timeframe_is_invalid;
            let enabled = !is_invalid;
            self.obj()
                .action_set_enabled("event-details-dialog.save", enabled);
        }

        #[template_callback]
        fn edit(&self) {
            let event = self.obj().event().expect("event should be initialized");

            self.navigation_view.push_by_tag("editor");

            self.name_entry.set_text(&event.name().unwrap());
            self.location_entry.set_text(&event.location().unwrap());
            self.conference_entry.set_text(&event.conference().unwrap());
            self.timeframe_picker
                .set_timeframe(event.timeframe().unwrap());
            self.description_entry
                .set_text(&event.description().unwrap());
        }

        async fn update_event(&self) {
            self.save.grab_focus();
            self.cancel.set_sensitive(false);
            self.save.set_is_loading(true);
            self.name_entry.set_sensitive(false);
            self.location_entry.set_sensitive(false);
            self.conference_entry.set_sensitive(false);
            self.description_entry.set_sensitive(false);

            let event = self.obj().event().expect("event should be initialized");

            let name = self.name_entry.text();
            let description = self.description_entry.text();
            let location = self.location_entry.text();
            let conference = self.conference_entry.text();
            let timeframe = self.timeframe_picker.timeframe().expect(
                "A timeframe should be set if the user was able to activate the event update",
            );

            match event
                .try_update_future(
                    None::<&Calendar>,
                    Some(&name),
                    Some(&description),
                    Some(&location),
                    Some(&conference),
                    Some(&timeframe),
                )
                .await
            {
                Ok(()) => {
                    debug!("Event updated: {}", event.uri().unwrap());
                    let _ = self.navigation_view.pop();
                }
                Err(error) => {
                    warn!("Failed to create event: {error}");
                    self.editor_toast_overlay.dismiss_all();
                    let toast = adw::Toast::new("An error occurred");
                    toast.set_button_label(Some("Details"));
                    toast.set_action_name(Some("event-details-dialog.show-error"));
                    toast.set_action_target(Some(&error.message()));
                    self.editor_toast_overlay.add_toast(toast);
                }
            }

            self.cancel.set_sensitive(true);
            self.save.set_is_loading(false);
            self.name_entry.set_sensitive(true);
            self.location_entry.set_sensitive(true);
            self.conference_entry.set_sensitive(true);
            self.description_entry.set_sensitive(true);
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
