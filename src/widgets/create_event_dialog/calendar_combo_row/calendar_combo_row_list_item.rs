use std::cell::{Cell, RefCell};

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
    #[template(resource = "/io/gitlab/TitouanReal/CalendarManager/calendar_combo_row_list_item.ui")]
    #[properties(wrapper_type = super::CalendarComboRowListItem)]
    pub struct CalendarComboRowListItem {
        // TODO: keep pub?
        #[property(get, set)]
        pub calendar: RefCell<Option<Calendar>>,
        #[property(get, set)]
        pub selected: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CalendarComboRowListItem {
        const NAME: &'static str = "CalendarComboRowListItem";
        type Type = super::CalendarComboRowListItem;
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
    impl ObjectImpl for CalendarComboRowListItem {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }
    impl WidgetImpl for CalendarComboRowListItem {}
    impl BoxImpl for CalendarComboRowListItem {}

    #[gtk::template_callbacks]
    impl CalendarComboRowListItem {
        #[template_callback]
        fn get_color_image(&self, color: RGBA) -> Paintable {
            get_circle_paintable_from_color(&color, 16.)
        }
    }
}

glib::wrapper! {
    pub struct CalendarComboRowListItem(ObjectSubclass<imp::CalendarComboRowListItem>)
    @extends gtk::Widget, gtk::Box,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl CalendarComboRowListItem {
    pub fn new(calendar: &Calendar, selected: bool) -> Self {
        glib::Object::builder()
            .property("calendar", calendar)
            .property("selected", selected)
            .build()
    }
}
