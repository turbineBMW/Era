use std::cell::Cell;

use adw::{prelude::*, subclass::prelude::*};
use glib::clone;
use jiff::ToSpan;

use crate::{Application, system_settings::FirstDayOfWeek};

use super::{MonthViewStyling, month_view_day_cell::MonthViewDayCell};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Kalendasom/month_view_week_row.ui")]
    #[properties(wrapper_type = super::MonthViewWeekRow)]
    pub struct MonthViewWeekRow {
        #[property(get, set = Self::set_year)]
        year: Cell<i32>,
        #[property(get, set = Self::set_month)]
        month: Cell<i32>,
        #[property(get, set = Self::set_day)]
        day: Cell<i32>,
        #[property(get, set, builder(MonthViewStyling::default()))]
        styling: Cell<MonthViewStyling>,
        #[template_child]
        day_1: TemplateChild<MonthViewDayCell>,
        #[template_child]
        day_2: TemplateChild<MonthViewDayCell>,
        #[template_child]
        day_3: TemplateChild<MonthViewDayCell>,
        #[template_child]
        day_4: TemplateChild<MonthViewDayCell>,
        #[template_child]
        day_5: TemplateChild<MonthViewDayCell>,
        #[template_child]
        day_6: TemplateChild<MonthViewDayCell>,
        #[template_child]
        day_7: TemplateChild<MonthViewDayCell>,
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
        fn constructed(&self) {
            self.parent_constructed();

            // TODO: Validate year/month/row?

            self.update_cells();

            Application::default()
                .system_settings()
                .connect_first_day_of_week_notify(clone!(
                    #[weak(rename_to = imp)]
                    self,
                    move |_| {
                        imp.update_cells();
                    }
                ));
        }
    }

    impl WidgetImpl for MonthViewWeekRow {}
    impl BoxImpl for MonthViewWeekRow {}

    #[gtk::template_callbacks]
    impl MonthViewWeekRow {
        // /// Sets the triplet year-month-day.
        // pub(super) fn set_year_month_day(&self, year: i32, month: i32, day: i32) {
        //     if self.year.get() != year {
        //         self.year.set(year);
        //         self.obj().notify_year();
        //     }

        //     if self.month.get() != month {
        //         self.month.set(month);
        //         self.obj().notify_month();
        //     }

        //     if self.day.get() != day {
        //         self.day.set(day);
        //         self.obj().notify_day();
        //     }

        //     self.update_cells();
        // }

        fn set_year(&self, year: i32) {
            if self.year.get() == year {
                return;
            }

            self.year.set(year);
            self.obj().notify_year();
            self.update_cells();
        }

        fn set_month(&self, month: i32) {
            if self.month.get() == month {
                return;
            }

            self.month.set(month);
            self.obj().notify_month();
            self.update_cells();
        }

        fn set_day(&self, day: i32) {
            if self.day.get() == day {
                return;
            }

            self.day.set(day);
            self.obj().notify_day();
            self.update_cells();
        }

        /// Updates the days of each cell in the row.
        fn update_cells(&self) {
            let obj = self.obj();

            let year = obj.year() as i16;
            let month = obj.month() as i8;
            let day = obj.day() as i8;

            let Ok(date) = jiff::civil::Date::new(year, month, day) else {
                return;
            };

            let base = match Application::default().system_settings().first_day_of_week() {
                FirstDayOfWeek::Monday => 1,
                FirstDayOfWeek::Tuesday => 2,
                FirstDayOfWeek::Wednesday => 3,
                FirstDayOfWeek::Thursday => 4,
                FirstDayOfWeek::Friday => 5,
                FirstDayOfWeek::Saturday => 6,
                FirstDayOfWeek::Sunday => 7,
            };
            let offset = match date.weekday() {
                jiff::civil::Weekday::Monday => 1,
                jiff::civil::Weekday::Tuesday => 2,
                jiff::civil::Weekday::Wednesday => 3,
                jiff::civil::Weekday::Thursday => 4,
                jiff::civil::Weekday::Friday => 5,
                jiff::civil::Weekday::Saturday => 6,
                jiff::civil::Weekday::Sunday => 7,
            };

            let go_back_by = offset - base;

            let date_1 = date.checked_sub(go_back_by.days()).unwrap();
            self.day_1.set_year(date_1.year() as i32);
            self.day_1.set_month(date_1.month() as i32);
            self.day_1.set_day(date_1.day() as i32);
            // TODO: Use this instead
            // self.day_1.set_year_month_day(
            //     date_1.year() as i32,
            //     date_1.month() as i32,
            //     date_1.day() as i32,
            // );

            let date_2 = date_1.checked_add(1.day()).unwrap();
            self.day_2.set_year(date_2.year() as i32);
            self.day_2.set_month(date_2.month() as i32);
            self.day_2.set_day(date_2.day() as i32);
            // TODO: Use this instead
            // self.day_2.set_year_month_day(
            //     date_2.year() as i32,
            //     date_2.month() as i32,
            //     date_2.day() as i32,
            // );

            let date_3 = date_1.checked_add(2.days()).unwrap();
            self.day_3.set_year(date_3.year() as i32);
            self.day_3.set_month(date_3.month() as i32);
            self.day_3.set_day(date_3.day() as i32);
            // TODO: Use this instead
            // self.day_3.set_year_month_day(
            //     date_3.year() as i32,
            //     date_3.month() as i32,
            //     date_3.day() as i32,
            // );

            let date_4 = date_1.checked_add(3.days()).unwrap();
            self.day_4.set_year(date_4.year() as i32);
            self.day_4.set_month(date_4.month() as i32);
            self.day_4.set_day(date_4.day() as i32);
            // TODO: Use this instead
            // self.day_4.set_year_month_day(
            //     date_4.year() as i32,
            //     date_4.month() as i32,
            //     date_4.day() as i32,
            // );

            let date_5 = date_1.checked_add(4.days()).unwrap();
            self.day_5.set_year(date_5.year() as i32);
            self.day_5.set_month(date_5.month() as i32);
            self.day_5.set_day(date_5.day() as i32);
            // TODO: Use this instead
            // self.day_5.set_year_month_day(
            //     date_5.year() as i32,
            //     date_5.month() as i32,
            //     date_5.day() as i32,
            // );

            let date_6 = date_1.checked_add(5.days()).unwrap();
            self.day_6.set_year(date_6.year() as i32);
            self.day_6.set_month(date_6.month() as i32);
            self.day_6.set_day(date_6.day() as i32);
            // TODO: Use this instead
            // self.day_6.set_year_month_day(
            //     date_6.year() as i32,
            //     date_6.month() as i32,
            //     date_6.day() as i32,
            // );

            let date_7 = date_1.checked_add(6.days()).unwrap();
            self.day_7.set_year(date_7.year() as i32);
            self.day_7.set_month(date_7.month() as i32);
            self.day_7.set_day(date_7.day() as i32);
            // TODO: Use this instead
            // self.day_7.set_year_month_day(
            //     date_7.year() as i32,
            //     date_7.month() as i32,
            //     date_7.day() as i32,
            // );
        }
    }
}

glib::wrapper! {
    pub struct MonthViewWeekRow(ObjectSubclass<imp::MonthViewWeekRow>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl MonthViewWeekRow {
    pub fn new(year: i32, month: i32, day: i32) -> Self {
        glib::Object::builder()
            .property("year", year)
            .property("month", month)
            .property("day", day)
            .build()
    }

    // /// Sets the triplet year-month-day.
    // pub fn set_year_month_day(&self, year: i32, month: i32, day: i32) {
    //     self.imp().set_year_month_day(year, month, day);
    // }
}
