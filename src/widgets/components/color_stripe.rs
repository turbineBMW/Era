use std::cell::RefCell;

use adw::{prelude::*, subclass::prelude::*};
use glib::translate::*;

mod imp {
    use super::*;

    #[derive(Debug, glib::Properties)]
    #[properties(wrapper_type = super::ColorStripe)]
    pub struct ColorStripe {
        #[property(get, set = Self::set_color)]
        color: RefCell<gdk::RGBA>,

        css_class: RefCell<Option<String>>,
    }

    impl Default for ColorStripe {
        fn default() -> Self {
            Self {
                color: RefCell::new(gdk::RGBA::WHITE),
                css_class: RefCell::new(None),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ColorStripe {
        const NAME: &'static str = "ColorStripe";
        type Type = super::ColorStripe;
        type ParentType = adw::Bin;
    }

    #[glib::derived_properties]
    impl ObjectImpl for ColorStripe {
        fn constructed(&self) {
            self.parent_constructed();

            self.obj().add_css_class("color-stripe");
            self.update_color();
        }
    }

    impl WidgetImpl for ColorStripe {}
    impl BinImpl for ColorStripe {}

    impl ColorStripe {
        fn set_color(&self, color: gdk::RGBA) {
            if color == *self.color.borrow() {
                return;
            }

            self.color.replace(color);

            self.update_color();

            self.obj().notify_color();

            self.obj().queue_draw();
        }

        fn update_color(&self) {
            let obj = self.obj();

            if let Some(old_class) = self.css_class.borrow_mut().take() {
                obj.remove_css_class(&old_class);
            }

            let color = obj.color();

            let color_str = color.to_string();
            let color_id = glib::Quark::from_str(&color_str);
            let css_class = format!("color-{}", color_id.into_glib());

            obj.add_css_class(&css_class);
            self.css_class.replace(Some(css_class));
        }
    }
}

glib::wrapper! {
    pub struct ColorStripe(ObjectSubclass<imp::ColorStripe>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}
