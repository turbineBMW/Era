use std::cell::Cell;

use adw::{prelude::*, subclass::prelude::*};

pub(crate) mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Kalendasom/year_view_month_cell.ui")]
    #[properties(wrapper_type = super::MonthViewDayCell)]
    pub struct MonthViewDayCell {
        #[property(get, set)]
        year: Cell<i32>,
        #[property(get, set)]
        month: Cell<i8>,
        #[property(get, set)]
        day: Cell<i8>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MonthViewDayCell {
        const NAME: &'static str = "MonthViewDayCell";
        type Type = super::MonthViewDayCell;
        type ParentType = gtk::Button;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MonthViewDayCell {}

    impl WidgetImpl for MonthViewDayCell {}
    impl ButtonImpl for MonthViewDayCell {}

    #[gtk::template_callbacks]
    impl MonthViewDayCell {}
}

glib::wrapper! {
    pub struct MonthViewDayCell(ObjectSubclass<imp::MonthViewDayCell>)
        @extends gtk::Widget, gtk::Button,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl MonthViewDayCell {
    pub fn new(year: i32, month: i8, day: i8) -> Self {
        glib::Object::builder()
            .property("year", year)
            .property("month", month)
            .property("day", day)
            .build()
    }
}
