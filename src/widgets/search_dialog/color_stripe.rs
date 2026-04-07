use std::cell::RefCell;

use gtk::{graphene, gsk, prelude::*, subclass::prelude::*};

mod imp {
    use super::*;

    #[derive(Debug, glib::Properties)]
    #[properties(wrapper_type = super::ColorStripe)]
    pub struct ColorStripe {
        #[property(get, set = Self::set_color)]
        color: RefCell<gdk::RGBA>,
    }

    impl Default for ColorStripe {
        fn default() -> Self {
            Self {
                color: RefCell::new(gdk::RGBA::WHITE),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ColorStripe {
        const NAME: &'static str = "ColorStripe";
        type Type = super::ColorStripe;
        type ParentType = gtk::Widget;
    }

    #[glib::derived_properties]
    impl ObjectImpl for ColorStripe {}

    impl WidgetImpl for ColorStripe {
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let obj = self.obj();
            let width = obj.width() as f32;
            let height = obj.height() as f32;

            let rect = graphene::Rect::new(0., 0., width, height);
            let rounded_rect = gsk::RoundedRect::from_rect(rect, height);
            snapshot.push_rounded_clip(&rounded_rect);
            snapshot.append_color(&self.color.borrow(), &rect);
            snapshot.pop();
        }
    }

    impl ColorStripe {
        fn set_color(&self, color: gdk::RGBA) {
            self.color.replace(color);
            self.obj().queue_draw();
        }
    }
}

glib::wrapper! {
    pub struct ColorStripe(ObjectSubclass<imp::ColorStripe>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}
