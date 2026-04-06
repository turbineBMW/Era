use std::cell::Cell;

use adw::{prelude::*, subclass::prelude::*};
use glib::clone;
use jiff::ToSpan;

use crate::{Application, system_settings::FirstDayOfWeek};

use super::{MonthViewStyling, month_view_header::MonthViewHeader};

pub const MINIMUM_WIDTH: i32 = 0;
pub const NATURAL_WIDTH: i32 = 0;
pub const MINIMUM_HEIGHT: i32 = 60;
pub const NATURAL_HEIGHT: i32 = MINIMUM_HEIGHT * 3;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Kalendasom/month_view_row.ui")]
    #[properties(wrapper_type = super::MonthViewRow)]
    pub struct MonthViewRow {
        #[property(get)]
        year: Cell<i32>,
        #[property(get)]
        month: Cell<i32>,
        #[property(get)]
        day: Cell<i32>,
        #[property(get, set, builder(MonthViewStyling::default()))]
        styling: Cell<MonthViewStyling>,
        #[template_child]
        overlay: TemplateChild<gtk::Overlay>,
        #[template_child]
        header_1: TemplateChild<MonthViewHeader>,
        #[template_child]
        header_2: TemplateChild<MonthViewHeader>,
        #[template_child]
        header_3: TemplateChild<MonthViewHeader>,
        #[template_child]
        header_4: TemplateChild<MonthViewHeader>,
        #[template_child]
        header_5: TemplateChild<MonthViewHeader>,
        #[template_child]
        header_6: TemplateChild<MonthViewHeader>,
        #[template_child]
        header_7: TemplateChild<MonthViewHeader>,
        #[template_child]
        above_1: TemplateChild<gtk::Separator>,
        #[template_child]
        above_2: TemplateChild<gtk::Separator>,
        #[template_child]
        above_3: TemplateChild<gtk::Separator>,
        #[template_child]
        above_4: TemplateChild<gtk::Separator>,
        #[template_child]
        above_5: TemplateChild<gtk::Separator>,
        #[template_child]
        above_6: TemplateChild<gtk::Separator>,
        #[template_child]
        above_7: TemplateChild<gtk::Separator>,
        #[template_child]
        between_1_and_2: TemplateChild<gtk::Separator>,
        #[template_child]
        between_2_and_3: TemplateChild<gtk::Separator>,
        #[template_child]
        between_3_and_4: TemplateChild<gtk::Separator>,
        #[template_child]
        between_4_and_5: TemplateChild<gtk::Separator>,
        #[template_child]
        between_5_and_6: TemplateChild<gtk::Separator>,
        #[template_child]
        between_6_and_7: TemplateChild<gtk::Separator>,
        #[template_child]
        corner_between_1_and_2: TemplateChild<gtk::Separator>,
        #[template_child]
        corner_between_2_and_3: TemplateChild<gtk::Separator>,
        #[template_child]
        corner_between_3_and_4: TemplateChild<gtk::Separator>,
        #[template_child]
        corner_between_4_and_5: TemplateChild<gtk::Separator>,
        #[template_child]
        corner_between_5_and_6: TemplateChild<gtk::Separator>,
        #[template_child]
        corner_between_6_and_7: TemplateChild<gtk::Separator>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MonthViewRow {
        const NAME: &'static str = "MonthViewRow";
        type Type = super::MonthViewRow;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();

            klass.set_css_name("month-view-row");
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MonthViewRow {
        fn constructed(&self) {
            self.parent_constructed();

            // TODO: Validate year/month/row?

            self.update_days();

            Application::default()
                .system_settings()
                .connect_first_day_of_week_notify(clone!(
                    #[weak(rename_to = imp)]
                    self,
                    move |_| {
                        imp.update_days();
                    }
                ));
        }

        fn dispose(&self) {
            self.overlay.unparent();
        }
    }

    impl WidgetImpl for MonthViewRow {
        fn request_mode(&self) -> gtk::SizeRequestMode {
            gtk::SizeRequestMode::ConstantSize
        }

        fn measure(&self, orientation: gtk::Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
            match orientation {
                gtk::Orientation::Horizontal => (MINIMUM_WIDTH, NATURAL_WIDTH, -1, -1),
                gtk::Orientation::Vertical => (MINIMUM_HEIGHT, NATURAL_HEIGHT, -1, -1),
                _ => unreachable!(),
            }
        }

        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            let allocation = gtk::Allocation::new(0, 0, width, height);
            self.overlay.size_allocate(&allocation, baseline);
        }
    }

    #[gtk::template_callbacks]
    impl MonthViewRow {
        /// Sets the triplet year-month-day.
        pub(super) fn set_year_month_day(&self, year: i32, month: i32, day: i32) {
            if self.year.get() != year {
                self.year.set(year);
                self.obj().notify_year();
            }

            if self.month.get() != month {
                self.month.set(month);
                self.obj().notify_month();
            }

            if self.day.get() != day {
                self.day.set(day);
                self.obj().notify_day();
            }

            self.update_days();
        }

        /// Updates the days of each cell in the row.
        fn update_days(&self) {
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
            self.header_1.set_year_month_day(
                date_1.year() as i32,
                date_1.month() as i32,
                date_1.day() as i32,
            );
            if date_1.day() <= 7 {
                self.above_1.add_css_class("month-separator");
            } else {
                self.above_1.remove_css_class("month-separator");
            }

            let date_2 = date_1.checked_add(1.day()).unwrap();
            self.header_2.set_year_month_day(
                date_2.year() as i32,
                date_2.month() as i32,
                date_2.day() as i32,
            );
            if date_2.day() == 1 {
                self.between_1_and_2.add_css_class("month-separator");
            } else {
                self.between_1_and_2.remove_css_class("month-separator");
            }
            if date_2.day() <= 8 {
                self.corner_between_1_and_2.add_css_class("month-separator");
            } else {
                self.corner_between_1_and_2
                    .remove_css_class("month-separator");
            }
            if date_2.day() <= 7 {
                self.above_2.add_css_class("month-separator");
            } else {
                self.above_2.remove_css_class("month-separator");
            }

            let date_3 = date_1.checked_add(2.days()).unwrap();
            self.header_3.set_year_month_day(
                date_3.year() as i32,
                date_3.month() as i32,
                date_3.day() as i32,
            );
            if date_3.day() == 1 {
                self.between_2_and_3.add_css_class("month-separator");
            } else {
                self.between_2_and_3.remove_css_class("month-separator");
            }
            if date_3.day() <= 8 {
                self.corner_between_2_and_3.add_css_class("month-separator");
            } else {
                self.corner_between_2_and_3
                    .remove_css_class("month-separator");
            }
            if date_3.day() <= 7 {
                self.above_3.add_css_class("month-separator");
            } else {
                self.above_3.remove_css_class("month-separator");
            }

            let date_4 = date_1.checked_add(3.days()).unwrap();
            self.header_4.set_year_month_day(
                date_4.year() as i32,
                date_4.month() as i32,
                date_4.day() as i32,
            );
            if date_4.day() == 1 {
                self.between_3_and_4.add_css_class("month-separator");
            } else {
                self.between_3_and_4.remove_css_class("month-separator");
            }
            if date_4.day() <= 8 {
                self.corner_between_3_and_4.add_css_class("month-separator");
            } else {
                self.corner_between_3_and_4
                    .remove_css_class("month-separator");
            }
            if date_4.day() <= 7 {
                self.above_4.add_css_class("month-separator");
            } else {
                self.above_4.remove_css_class("month-separator");
            }

            let date_5 = date_1.checked_add(4.days()).unwrap();
            self.header_5.set_year_month_day(
                date_5.year() as i32,
                date_5.month() as i32,
                date_5.day() as i32,
            );
            if date_5.day() == 1 {
                self.between_4_and_5.add_css_class("month-separator");
            } else {
                self.between_4_and_5.remove_css_class("month-separator");
            }
            if date_5.day() <= 8 {
                self.corner_between_4_and_5.add_css_class("month-separator");
            } else {
                self.corner_between_4_and_5
                    .remove_css_class("month-separator");
            }
            if date_5.day() <= 7 {
                self.above_5.add_css_class("month-separator");
            } else {
                self.above_5.remove_css_class("month-separator");
            }

            let date_6 = date_1.checked_add(5.days()).unwrap();
            self.header_6.set_year_month_day(
                date_6.year() as i32,
                date_6.month() as i32,
                date_6.day() as i32,
            );
            if date_6.day() == 1 {
                self.between_5_and_6.add_css_class("month-separator");
            } else {
                self.between_5_and_6.remove_css_class("month-separator");
            }
            if date_6.day() <= 8 {
                self.corner_between_5_and_6.add_css_class("month-separator");
            } else {
                self.corner_between_5_and_6
                    .remove_css_class("month-separator");
            }
            if date_6.day() <= 7 {
                self.above_6.add_css_class("month-separator");
            } else {
                self.above_6.remove_css_class("month-separator");
            }

            let date_7 = date_1.checked_add(6.days()).unwrap();
            self.header_7.set_year_month_day(
                date_7.year() as i32,
                date_7.month() as i32,
                date_7.day() as i32,
            );
            if date_7.day() == 1 {
                self.between_6_and_7.add_css_class("month-separator");
            } else {
                self.between_6_and_7.remove_css_class("month-separator");
            }
            if date_7.day() <= 8 {
                self.corner_between_6_and_7.add_css_class("month-separator");
            } else {
                self.corner_between_6_and_7
                    .remove_css_class("month-separator");
            }
            if date_7.day() <= 7 {
                self.above_7.add_css_class("month-separator");
            } else {
                self.above_7.remove_css_class("month-separator");
            }
        }
    }
}

glib::wrapper! {
    pub struct MonthViewRow(ObjectSubclass<imp::MonthViewRow>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl MonthViewRow {
    pub fn new() -> Self {
        glib::Object::new()
    }

    /// Sets the triplet year-month-day.
    pub fn set_year_month_day(&self, year: i32, month: i32, day: i32) {
        self.imp().set_year_month_day(year, month, day);
    }
}
