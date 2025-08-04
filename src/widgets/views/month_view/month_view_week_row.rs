use std::{cell::Cell, sync::LazyLock};

use adw::{prelude::*, subclass::prelude::*};
use gtk::glib::{self, closure_local, subclass::Signal};

// use super::MonthViewDayCell;

pub(crate) mod imp {
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
    impl ObjectImpl for MonthViewWeekRow {
        fn signals() -> &'static [Signal] {
            static SIGNALS: LazyLock<Vec<Signal>> = LazyLock::new(|| {
                vec![
                    Signal::builder("month-clicked")
                        // Year, Month
                        // TODO: Should these be something else than i32?
                        .param_types([i32::static_type(), i32::static_type()])
                        .build(),
                ]
            });
            SIGNALS.as_ref()
        }
    }

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

    pub fn connect_month_clicked<F: Fn(&Self, i32, i32) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_closure(
            "month-clicked",
            true,
            closure_local!(move |obj: Self, year: i32, month: i32| {
                f(&obj, year, month);
            }),
        )
    }
}
