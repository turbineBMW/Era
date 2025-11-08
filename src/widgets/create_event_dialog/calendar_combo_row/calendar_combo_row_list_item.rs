use std::cell::{Cell, RefCell};

use adw::{prelude::*, subclass::prelude::*};
use clepsydre::Calendar;

use crate::utils::PaintableCallbacks;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Kalendasom/calendar_combo_row_list_item.ui")]
    #[properties(wrapper_type = super::CalendarComboRowListItem)]
    pub struct CalendarComboRowListItem {
        #[property(get, set)]
        calendar: RefCell<Option<Calendar>>,
        #[property(get, set)]
        selected: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CalendarComboRowListItem {
        const NAME: &'static str = "CalendarComboRowListItem";
        type Type = super::CalendarComboRowListItem;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
            PaintableCallbacks::bind_template_callbacks(klass);
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
    impl CalendarComboRowListItem {}
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
