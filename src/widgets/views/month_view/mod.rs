use std::{
    cell::{Cell, OnceCell},
    sync::{LazyLock, Mutex},
};

use adw::{prelude::*, subclass::prelude::*};
use ccm::jiff;
use gtk::{
    Allocation,
    glib::{self, subclass::Signal},
};

mod month_view_day_cell;
mod month_view_week_row;

use self::{month_view_day_cell::*, month_view_week_row::*};

pub(crate) mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/CalendarManager/month_view.ui")]
    #[properties(wrapper_type = super::MonthView)]
    pub struct MonthView {
        #[property(get, set)]
        year: Cell<i32>,
        // month will not change by itself. Create setters for year and week, and emit notifies
        #[property(get = Self::get_month)]
        month: Cell<i32>,
        #[property(get, set)]
        week: Cell<i8>,
        week_rows: OnceCell<Mutex<Vec<MonthViewWeekRow>>>,
        scroll_offset: Cell<f64>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MonthView {
        const NAME: &'static str = "MonthView";
        type Type = super::MonthView;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MonthView {
        fn constructed(&self) {
            let obj = self.obj();

            let now = jiff::Zoned::now();
            let current_year = now.year() as i32;
            let current_week = now.iso_week_date().week();
            obj.set_year(current_year);
            obj.set_week(current_week);

            let first_row = if current_week == 0 {
                let last_day = jiff::civil::Date::new(current_year as i16 - 1, 12, 31).unwrap();
                let last_week = last_day.iso_week_date().week();
                MonthViewWeekRow::new(current_year - 1, last_week)
            } else {
                MonthViewWeekRow::new(current_year, current_week)
            };
            // first_row.connect_month_clicked(clone!(
            //     #[weak(rename_to = imp)]
            //     self,
            //     move |_row, year, month| {
            //         imp.obj()
            //             .emit_by_name::<()>("month-clicked", &[&year, &month]);
            //     }
            // ));
            first_row.insert_before(&*self.obj(), None::<&gtk::Widget>);

            let (row_height, ..) = first_row.measure(gtk::Orientation::Vertical, 400);
            let offset = row_height as f64;
            self.scroll_offset.set(offset);
            let nb_rows = (self.obj().height() / row_height + 1) as u8;

            let mut week_rows = vec![first_row];
            let mut first_day_of_new_week = jiff::civil::ISOWeekDate::new(
                current_year as i16,
                current_week,
                jiff::civil::Weekday::Monday,
            )
            .unwrap()
            .date();
            for _ in 0..nb_rows {
                first_day_of_new_week = first_day_of_new_week
                    .checked_add(jiff::Span::new().days(7))
                    .unwrap();
                let row = MonthViewWeekRow::new(
                    first_day_of_new_week.year() as i32,
                    first_day_of_new_week.iso_week_date().week(),
                );
                row.insert_before(&*self.obj(), None::<&gtk::Widget>);
                // row.connect_month_clicked(clone!(
                //     #[weak(rename_to = imp)]
                //     self,
                //     move |_row, year, month| {
                //         imp.obj()
                //             .emit_by_name::<()>("month-clicked", &[&year, &month]);
                //     }
                // ));
                week_rows.push(row);
            }
            self.week_rows.get_or_init(|| Mutex::new(week_rows));
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: LazyLock<Vec<Signal>> =
                LazyLock::new(|| vec![Signal::builder("day-clicked").build()]);
            SIGNALS.as_ref()
        }
    }

    impl WidgetImpl for MonthView {
        fn size_allocate(&self, width: i32, _height: i32, baseline: i32) {
            let week_rows = self.week_rows.get().unwrap().lock().unwrap();
            let first_row = week_rows.first().unwrap();
            let (row_height, ..) = first_row.measure(gtk::Orientation::Vertical, width);
            for (i, row) in week_rows.iter().enumerate() {
                let allocation = Allocation::new(
                    0,
                    -self.scroll_offset.get() as i32 + i as i32 * row_height,
                    width,
                    row_height,
                );
                row.size_allocate(&allocation, baseline);
            }
        }
    }

    #[gtk::template_callbacks]
    impl MonthView {
        fn get_month(&self) -> i32 {
            let weekdate = jiff::civil::ISOWeekDate::new(
                self.obj().year() as i16,
                self.obj().week(),
                jiff::civil::Weekday::Monday,
            )
            .expect("Week number should be valid");
            weekdate.date().month() as i32
        }

        #[template_callback]
        fn get_month_label(&self) -> String {
            let year = self.obj().year();
            let month = self.obj().month();
            format!("{year} {month}")
        }

        #[template_callback]
        fn day_cell_clicked(&self) {
            self.obj().emit_by_name::<()>("day-clicked", &[]);
        }

        #[template_callback]
        fn scroll(&self, _dx: f64, _dy: f64) -> bool {
            true
        }
    }
}

glib::wrapper! {
    pub struct MonthView(ObjectSubclass<imp::MonthView>)
        @extends gtk::Widget;
}

impl MonthView {}
