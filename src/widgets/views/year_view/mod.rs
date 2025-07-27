use std::{
    cell::{Cell, OnceCell},
    sync::{LazyLock, Mutex},
};

use adw::{prelude::*, subclass::prelude::*};
use gtk::{
    Allocation,
    glib::{self, clone, subclass::Signal},
};

mod year_view_month_cell;
mod year_view_year_row;

use crate::CalendarManagerApplication;

use self::{year_view_month_cell::*, year_view_year_row::*};

const SPACING: i32 = 12;

#[derive(Debug, Default, Hash, Eq, PartialEq, Clone, Copy, glib::Enum)]
#[enum_type(name = "YearViewStyling")]
pub enum YearViewStyling {
    #[enum_value(name = "Narrow", nick = "narrow")]
    Narrow,
    #[default]
    #[enum_value(name = "Medium", nick = "medium")]
    Medium,
    #[enum_value(name = "Wide", nick = "wide")]
    Wide,
}

pub(crate) mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/CalendarManager/year_view.ui")]
    #[properties(wrapper_type = super::YearView)]
    pub struct YearView {
        #[property(get, set)]
        year: Cell<i32>,
        #[property(get, set, builder(YearViewStyling::default()))]
        styling: Cell<YearViewStyling>,
        // TODO: I should remove the OnceCell? Should I use Cell instead of Mutex?
        year_rows: OnceCell<Mutex<Vec<YearViewYearRow>>>,
        scroll_offset: Cell<f64>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for YearView {
        const NAME: &'static str = "YearView";
        type Type = super::YearView;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            YearViewMonthCell::ensure_type();
            YearViewYearRow::ensure_type();

            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for YearView {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            let application = CalendarManagerApplication::default();
            let current_year = application.current_year();
            obj.set_year(current_year);

            let first_row = YearViewYearRow::new(current_year - 1);
            obj.bind_property("styling", &first_row, "styling")
                .sync_create()
                .build();
            first_row.connect_month_clicked(clone!(
                #[weak(rename_to = imp)]
                self,
                move |_row, year, month| {
                    imp.obj()
                        .emit_by_name::<()>("month-clicked", &[&year, &month]);
                }
            ));
            first_row.insert_before(&*self.obj(), None::<&gtk::Widget>);

            let (row_height, ..) = first_row.measure(gtk::Orientation::Vertical, 800);
            let offset = row_height as f64;
            self.scroll_offset.set(offset);
            let nb_rows = self.obj().height() / row_height + 1;

            let mut year_rows = vec![first_row];
            for year in current_year..current_year + nb_rows + 1 {
                let row = YearViewYearRow::new(year);
                obj.bind_property("styling", &row, "styling")
                    .sync_create()
                    .build();
                row.insert_before(&*self.obj(), None::<&gtk::Widget>);
                row.connect_month_clicked(clone!(
                    #[weak(rename_to = imp)]
                    self,
                    move |_row, year, month| {
                        imp.obj()
                            .emit_by_name::<()>("month-clicked", &[&year, &month]);
                    }
                ));
                year_rows.push(row);
            }
            self.year_rows.get_or_init(|| Mutex::new(year_rows));
        }

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

    impl WidgetImpl for YearView {
        // TODO: check if i have been allocated enough space
        fn size_allocate(&self, width: i32, _height: i32, baseline: i32) {
            let year_rows = self.year_rows.get().unwrap().lock().unwrap();
            let last_row = year_rows
                .last()
                .expect("There should be at least one year row")
                .to_owned();
            let (minimum_row_height, ..) = last_row.measure(gtk::Orientation::Vertical, width);

            for (i, row) in year_rows.iter().enumerate() {
                let allocation = Allocation::new(
                    0,
                    -self.scroll_offset.get() as i32 + (i as i32 * (minimum_row_height + SPACING)),
                    width,
                    minimum_row_height,
                );
                row.size_allocate(&allocation, baseline);
            }
        }
    }

    #[gtk::template_callbacks]
    impl YearView {
        #[template_callback]
        fn get_year_label_narrow(&self) -> String {
            self.obj().year().to_string()
        }

        #[template_callback]
        fn month_cell_clicked(&self, year: i32, month: i32) {
            self.obj()
                .emit_by_name::<()>("month-clicked", &[&year, &month]);
        }

        #[template_callback]
        fn scroll(&self, _dx: f64, dy: f64) -> bool {
            let mut year_rows = self.year_rows.get().unwrap().lock().unwrap();
            let width = self.obj().width();
            let height = self.obj().height();
            let last_row = year_rows
                .last()
                .expect("There should be at least one year row")
                .to_owned();
            let (row_height, ..) = last_row.measure(gtk::Orientation::Vertical, width);

            // The y offset of the top of the first row
            let top_offset = self.scroll_offset.get() + dy;
            // The y offset of the bottom of the last row
            let bottom_offset = top_offset + height as f64;

            let first_row = year_rows
                .first()
                .expect("There should be at least one year row")
                .to_owned();
            let first_year = first_row.year();
            self.obj()
                .set_year(first_year + top_offset as i32 / row_height);

            let top_threshold = row_height as f64 / 2.;
            let bottom_threshold = (year_rows.len() as f64 - 0.5) * row_height as f64;

            if top_offset < top_threshold {
                self.scroll_offset.set(top_offset + row_height as f64);

                let first_row = year_rows
                    .first()
                    .expect("There should be at least one year row")
                    .to_owned();
                let last_row = year_rows.pop().unwrap();
                last_row.set_year(first_row.year() - 1);

                year_rows.insert(0, last_row);
            } else if bottom_offset > bottom_threshold {
                self.scroll_offset.set(top_offset - row_height as f64);

                let first_row = year_rows.remove(0);
                let last_row = year_rows
                    .last()
                    .expect("There should be at least one year row")
                    .clone();
                first_row.set_year(last_row.year() + 1);

                year_rows.push(first_row);
            } else {
                self.scroll_offset.set(top_offset);
            }

            self.obj().queue_allocate();
            true
        }

        #[template_callback]
        fn decelerate(&self, _velocity_x: f64, _velocity_y: f64) {
            // let duration =
            // self.scroll_offset.decelerate(template);
        }
    }
}

glib::wrapper! {
    pub struct YearView(ObjectSubclass<imp::YearView>)
        @extends gtk::Widget;
}
