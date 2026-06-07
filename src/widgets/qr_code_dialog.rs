use std::cell::RefCell;

use adw::{prelude::*, subclass::prelude::*};

use crate::utils::{PaintableCallbacks, TemplateCallbacks};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Era/qr_code_dialog.ui")]
    #[properties(wrapper_type = super::QrCodeDialog)]
    pub struct QrCodeDialog {
        #[property(get, set = Self::set_url, explicit_notify)]
        url: RefCell<String>,
        #[template_child]
        qr_code: TemplateChild<gtk::Picture>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for QrCodeDialog {
        const NAME: &'static str = "QrCodeDialog";
        type Type = super::QrCodeDialog;
        type ParentType = adw::Dialog;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            TemplateCallbacks::bind_template_callbacks(klass);
            PaintableCallbacks::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for QrCodeDialog {}

    impl WidgetImpl for QrCodeDialog {}
    impl AdwDialogImpl for QrCodeDialog {}

    impl QrCodeDialog {
        fn set_url(&self, url: &str) {
            if url == self.url.borrow().as_str() {
                return;
            }
            self.url.replace(url.to_string());
            let qr = qrcode::QrCode::new(url.as_bytes()).unwrap();
            let svg = qr
                .render::<qrcode::render::svg::Color>()
                .min_dimensions(500, 500)
                .build();
            let paintable: gdk::Paintable = gdk::Texture::from_bytes(&svg.as_bytes().into())
                .unwrap()
                .upcast();
            self.qr_code.set_paintable(Some(&paintable));
            self.obj().notify_url();
        }
    }
}

glib::wrapper! {
    pub struct QrCodeDialog(ObjectSubclass<imp::QrCodeDialog>)
        @extends gtk::Widget, adw::Dialog,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::ShortcutManager;
}

impl QrCodeDialog {
    pub fn new(url: &str) -> Self {
        glib::Object::builder().property("url", url).build()
    }
}
