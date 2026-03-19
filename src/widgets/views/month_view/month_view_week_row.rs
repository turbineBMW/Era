use std::cell::Cell;

use adw::{prelude::*, subclass::prelude::*};

// use super::MonthViewDayCell;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Kalendasom/month_view_week_row.ui")]
    #[properties(wrapper_type = super::MonthViewWeekRow)]
    pub struct MonthViewWeekRow {
        #[property(get, set)]
        year: Cell<i32>,
        #[property(get, set)]
        week: Cell<i8>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MonthViewWeekRow {
        const NAME: &'static str = "MonthViewWeekRow";
        type Type = super::MonthViewWeekRow;
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
    impl ObjectImpl for MonthViewWeekRow {}

    impl WidgetImpl for MonthViewWeekRow {}
    impl BoxImpl for MonthViewWeekRow {}

    #[gtk::template_callbacks]
    impl MonthViewWeekRow {}
}

glib::wrapper! {
    pub struct MonthViewWeekRow(ObjectSubclass<imp::MonthViewWeekRow>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl MonthViewWeekRow {
    pub fn new(year: i32, week: i8) -> Self {
        glib::Object::builder()
            .property("year", year)
            .property("week", week)
            .build()
    }
}
