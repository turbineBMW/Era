use std::cell::RefCell;

use adw::{prelude::*, subclass::prelude::*};
use ccm::Calendar;
use gtk::{
    gdk::{Paintable, RGBA},
    glib,
};

use crate::utils::get_circle_paintable_from_color;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/CalendarManager/calendar_combo_row_item.ui")]
    #[properties(wrapper_type = super::CalendarComboRowItem)]
    pub struct CalendarComboRowItem {
        #[property(get, set)]
        pub calendar: RefCell<Option<Calendar>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CalendarComboRowItem {
        const NAME: &'static str = "CalendarComboRowItem";
        type Type = super::CalendarComboRowItem;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for CalendarComboRowItem {}
    impl WidgetImpl for CalendarComboRowItem {}
    impl BoxImpl for CalendarComboRowItem {}

    #[gtk::template_callbacks]
    impl CalendarComboRowItem {
        #[template_callback]
        fn get_color_image(&self, color: RGBA) -> Paintable {
            get_circle_paintable_from_color(&color, 16.)
        }
    }
}

glib::wrapper! {
    pub struct CalendarComboRowItem(ObjectSubclass<imp::CalendarComboRowItem>)
    @extends gtk::Widget, gtk::Box,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl CalendarComboRowItem {
    pub fn new(calendar: &Calendar) -> Self {
        glib::Object::builder()
            .property("calendar", calendar)
            .build()
    }
}
