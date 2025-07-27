use std::cell::Cell;

use adw::{prelude::*, subclass::prelude::*};
use ccm::jiff;
use gettextrs::gettext;
use gtk::glib::{self, clone};

pub(crate) mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/CalendarManager/year_view_month_cell.ui")]
    #[properties(wrapper_type = super::MonthViewDayCell)]
    pub struct MonthViewDayCell {
        #[property(get, set)]
        year: Cell<i32>,
        #[property(get, set)]
        month: Cell<i32>,
        #[template_child]
        days_grid: TemplateChild<gtk::Grid>,
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
    impl ObjectImpl for MonthViewDayCell {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            obj.connect_year_notify(clone!(
                #[weak(rename_to = imp)]
                self,
                move |_| {
                    imp.set_days_grid();
                }
            ));
            obj.connect_month_notify(clone!(
                #[weak(rename_to = imp)]
                self,
                move |_| {
                    imp.set_days_grid();
                }
            ));
        }
    }

    impl WidgetImpl for MonthViewDayCell {}
    impl ButtonImpl for MonthViewDayCell {}

    #[gtk::template_callbacks]
    impl MonthViewDayCell {
        fn set_days_grid(&self) {
            let year = self.year.get();
            let month = self.month.get();
            let first_day = jiff::civil::date(year as i16, month as i8, 1);
            let days_in_month = first_day.days_in_month() as usize;
            let weekday_of_first_day = first_day.weekday() as usize - 1;

            for cell in 0..42 {
                if let Some(label) = self.days_grid.child_at(cell % 7, cell / 7) {
                    self.days_grid.remove(&label);
                }
            }

            let mut cells = 0..42;

            for cell in cells.by_ref().take(weekday_of_first_day) {
                let label = gtk::Label::new(None);
                self.days_grid.attach(&label, cell % 7, cell / 7, 1, 1);
            }

            for (day_number, cell) in cells.by_ref().take(days_in_month).enumerate() {
                let label = gtk::Label::new(Some(&format!("{}", day_number + 1)));
                label.add_css_class("year-view-month-cell-day-label");
                self.days_grid.attach(&label, cell % 7, cell / 7, 1, 1);
            }

            for cell in cells {
                let label = gtk::Label::new(None);
                label.add_css_class("year-view-month-cell-day-label");
                self.days_grid.attach(&label, cell % 7, cell / 7, 1, 1);
            }
        }

        #[template_callback]
        fn get_month_name(&self) -> String {
            let month = self.obj().month();
            match month {
                1 => gettext("January"),
                2 => gettext("February"),
                3 => gettext("March"),
                4 => gettext("April"),
                5 => gettext("May"),
                6 => gettext("June"),
                7 => gettext("July"),
                8 => gettext("August"),
                9 => gettext("September"),
                10 => gettext("October"),
                11 => gettext("November"),
                12 => gettext("December"),
                _ => "".to_string(),
            }
        }
    }
}

glib::wrapper! {
    pub struct MonthViewDayCell(ObjectSubclass<imp::MonthViewDayCell>)
        @extends gtk::Widget, gtk::Button,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl MonthViewDayCell {
    pub fn new(year: i32, month: i32) -> Self {
        glib::Object::builder()
            .property("year", year)
            .property("month", month)
            .build()
    }
}
