use std::cell::RefCell;

use adw::{prelude::*, subclass::prelude::*};
use ashpd::desktop::{
    file_chooser::{FileFilter, SelectedFiles},
    open_uri::OpenFileRequest,
};
use clepsydre::Event;
use gdk::gdk_pixbuf::{Colorspace, Pixbuf};
use image::{ImageBuffer, Rgb};
use qrcodegen::{QrCode, QrCodeEcc};
use tracing::warn;

use crate::{
    utils::{PaintableCallbacks, TemplateCallbacks},
    widgets::components::LoadingButton,
};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Kalendasom/event_details_dialog.ui")]
    #[properties(wrapper_type = super::EventDetailsDialog)]
    pub struct EventDetailsDialog {
        #[property(get, construct_only)]
        pub event: RefCell<Option<Event>>,
        #[template_child]
        navigation_view: TemplateChild<adw::NavigationView>,
        #[template_child]
        details_toast_overlay: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        name_label: TemplateChild<gtk::Label>,
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
    }

    #[glib::object_subclass]
    impl ObjectSubclass for EventDetailsDialog {
        const NAME: &'static str = "EventDetailsDialog";
        type Type = super::EventDetailsDialog;
        type ParentType = adw::Dialog;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
            TemplateCallbacks::bind_template_callbacks(klass);
            PaintableCallbacks::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for EventDetailsDialog {
        fn dispose(&self) {
            self.name_label.unparent();
        }
    }

    impl WidgetImpl for EventDetailsDialog {}
    impl AdwDialogImpl for EventDetailsDialog {}

    #[gtk::template_callbacks]
    impl EventDetailsDialog {
        #[template_callback]
        fn timeframe_label(&self) -> String {
            if let Some(event) = self.obj().event() {
                let start = event.timeframe().unwrap().start().to_string();
                let end = event.timeframe().unwrap().end().to_string();
                format!("{} - {}", start, end)
            } else {
                String::new()
            }
        }

        #[template_callback]
        fn location_or_video_conference(&self) -> bool {
            if let Some(event) = self.obj().event() {
                !event.location().is_empty() || !event.video_conference().is_empty()
            } else {
                false
            }
        }

        #[template_callback]
        fn video_conference_is_a_uri(&self) -> bool {
            if let Some(event) = self.obj().event() {
                url::Url::parse(&event.video_conference()).is_ok()
            } else {
                false
            }
        }

        #[template_callback]
        async fn share(&self) {
            let event = self.obj().event().expect("event should be initialized");
            let ics_content = event.to_string_for_ics();

            let request = match SelectedFiles::save_file()
                .title("Export Event")
                .accept_label("Export")
                .current_name(format!("{}.ics", event.name()).as_str())
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
                let file = gio::File::for_uri(file_uri.as_ref());
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
            pub fn to_qr_code_pixbuf(event: &Event) -> Pixbuf {
                let module_size = 8;
                let url = event.to_string_for_qr_code();

                // Generate the raw QR code structure
                let qr = QrCode::encode_text(&url, QrCodeEcc::Medium).unwrap();

                let size = qr.size() as u32;
                let img_size = size * module_size;

                // Create an image buffer from the raw QR data
                let mut img = ImageBuffer::new(img_size, img_size);

                let white = Rgb([255u8, 255u8, 255u8]);
                let black = Rgb([0u8, 0u8, 0u8]);

                // Iterate through the raw QR code modules
                for y in 0..size {
                    for x in 0..size {
                        let color = if qr.get_module(x as i32, y as i32) {
                            black
                        } else {
                            white
                        };

                        // Scale the module to the desired pixel size (module_size x module_size)
                        for i in 0..module_size {
                            for j in 0..module_size {
                                img.put_pixel(x * module_size + j, y * module_size + i, color);
                            }
                        }
                    }
                }

                // Transform into a pixbuf
                let data = glib::Bytes::from_owned(img.into_raw());
                Pixbuf::from_bytes(
                    &data,
                    Colorspace::Rgb,
                    false,
                    8,
                    img_size as i32,
                    img_size as i32,
                    img_size as i32 * 3,
                )
            }

            let event = self.obj().event().expect("event should be initialized");
            let dialog = adw::Dialog::new();
            let pixbuf = to_qr_code_pixbuf(&event);
            // TODO: Remove deprecated function
            #[allow(deprecated)]
            let image = gtk::Image::from_pixbuf(Some(&pixbuf));
            image.set_pixel_size(400);
            dialog.set_child(Some(&image));
            dialog.present(Some(&*self.obj()));
        }

        #[template_callback]
        async fn join(&self) {
            let event = self.obj().event().expect("event should be initialized");
            let uri = url::Url::parse(&event.video_conference())
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
        fn edit(&self) {
            let event = self.obj().event().expect("event should be initialized");

            self.navigation_view.push_by_tag("editor");

            self.name_entry.set_text(&event.name());
            self.location_entry.set_text(&event.location());
            self.video_conference_entry
                .set_text(&event.video_conference());
            self.description_entry.set_text(&event.description());
        }

        #[template_callback]
        async fn save(&self) {
            dbg!("todo");
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
