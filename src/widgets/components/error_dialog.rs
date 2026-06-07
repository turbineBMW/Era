use std::cell::RefCell;

use adw::{prelude::*, subclass::prelude::*};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Era/error_dialog.ui")]
    #[properties(wrapper_type = super::ErrorDialog)]
    pub struct ErrorDialog {
        #[property(get, construct_only)]
        error_message: RefCell<String>,
        #[template_child]
        toast_overlay: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        text_view: TemplateChild<gtk::TextView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ErrorDialog {
        const NAME: &'static str = "ErrorDialog";
        type Type = super::ErrorDialog;
        type ParentType = adw::Dialog;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for ErrorDialog {
        fn constructed(&self) {
            self.parent_constructed();

            self.text_view
                .buffer()
                .set_text(&self.error_message.borrow());
        }
    }

    impl WidgetImpl for ErrorDialog {}
    impl AdwDialogImpl for ErrorDialog {}

    #[gtk::template_callbacks]
    impl ErrorDialog {
        #[template_callback]
        fn copy(&self) {
            let clipboard = self.obj().clipboard();
            let text = self.text_view.buffer().property::<String>("text");
            clipboard.set_text(&text);

            self.toast_overlay.dismiss_all();
            let toast = adw::Toast::new("Error message copied to clipboard");
            self.toast_overlay.add_toast(toast);
        }
    }
}

glib::wrapper! {
    pub struct ErrorDialog(ObjectSubclass<imp::ErrorDialog>)
        @extends gtk::Widget, adw::Dialog,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::ShortcutManager;
}

impl ErrorDialog {
    pub fn new(error_message: &str) -> Self {
        glib::Object::builder()
            .property("error-message", error_message)
            .build()
    }
}
