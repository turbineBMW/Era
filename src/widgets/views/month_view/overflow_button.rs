use std::cell::{Cell, RefCell};

use adw::{prelude::*, subclass::prelude::*};

use crate::utils::TemplateCallbacks;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Era/overflow_button.ui")]
    #[properties(wrapper_type = super::OverflowButton)]
    pub struct OverflowButton {
        #[property(get, set)]
        text: RefCell<String>,

        #[template_child]
        inscription: TemplateChild<gtk::Inscription>,
        #[template_child]
        image: TemplateChild<gtk::Image>,

        small_mode: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for OverflowButton {
        const NAME: &'static str = "OverflowButton";
        type Type = super::OverflowButton;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("overflow-button");

            klass.bind_template();
            klass.bind_template_callbacks();
            TemplateCallbacks::bind_template_callbacks(klass);

            klass.install_action("event-widget.activate", None, |widget, _, _| {
                widget.imp().open_day_list();
            });

            klass.add_binding_action(
                gdk::Key::Return,
                gdk::ModifierType::empty(),
                "event-widget.activate",
            );
            klass.add_binding_action(
                gdk::Key::KP_Enter,
                gdk::ModifierType::empty(),
                "event-widget.activate",
            );
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for OverflowButton {
        fn dispose(&self) {
            self.inscription.unparent();
            self.image.unparent();
        }
    }

    impl WidgetImpl for OverflowButton {
        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            let allocation = gtk::Allocation::new(0, 0, width, height);
            if self.small_mode.get() {
                self.image.size_allocate(&allocation, baseline);
                self.inscription.set_child_visible(false);
                self.image.set_child_visible(true);
            } else {
                self.inscription.size_allocate(&allocation, baseline);
                self.inscription.set_child_visible(true);
                self.image.set_child_visible(false);
            }
        }
    }

    #[gtk::template_callbacks]
    impl OverflowButton {
        pub(super) fn set_small_mode(&self, small_mode: bool) {
            self.small_mode.set(small_mode);
        }

        #[template_callback]
        fn open_day_list(&self) {
            self.obj().grab_focus();

            let dialog = adw::Dialog::new();
            dialog.present(Some(&*self.obj()));
        }
    }
}

glib::wrapper! {
    pub struct OverflowButton(ObjectSubclass<imp::OverflowButton>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl OverflowButton {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn set_small_mode(&self, small_mode: bool) {
        self.imp().set_small_mode(small_mode);
    }
}

impl Default for OverflowButton {
    fn default() -> Self {
        Self::new()
    }
}
