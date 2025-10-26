use adw::{prelude::*, subclass::prelude::*};

mod imp {
    use std::cell::RefCell;

    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Kalendasom/error_dialog.ui")]
    #[properties(wrapper_type = super::ErrorDialog)]
    pub struct ErrorDialog {
        #[property(get, set, construct_only)]
        error_message: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ErrorDialog {
        const NAME: &'static str = "ErrorDialog";
        type Type = super::ErrorDialog;
        type ParentType = adw::Dialog;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for ErrorDialog {}
    impl WidgetImpl for ErrorDialog {}
    impl AdwDialogImpl for ErrorDialog {}
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
