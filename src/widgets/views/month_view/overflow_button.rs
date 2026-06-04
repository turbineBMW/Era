use adw::subclass::prelude::*;

use crate::utils::TemplateCallbacks;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Kalendasom/overflow_button.ui")]
    #[properties(wrapper_type = super::OverflowButton)]
    pub struct OverflowButton {}

    #[glib::object_subclass]
    impl ObjectSubclass for OverflowButton {
        const NAME: &'static str = "OverflowButton";
        type Type = super::OverflowButton;
        type ParentType = gtk::Button;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
            TemplateCallbacks::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for OverflowButton {}
    impl WidgetImpl for OverflowButton {}
    impl ButtonImpl for OverflowButton {}

    #[gtk::template_callbacks]
    impl OverflowButton {}
}

glib::wrapper! {
    pub struct OverflowButton(ObjectSubclass<imp::OverflowButton>)
        @extends gtk::Widget, gtk::Button,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl OverflowButton {
    pub fn new() -> Self {
        glib::Object::new()
    }
}

impl Default for OverflowButton {
    fn default() -> Self {
        Self::new()
    }
}
