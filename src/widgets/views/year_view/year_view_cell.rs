use std::{cell::Cell, cmp};

use adw::{prelude::*, subclass::prelude::*};
use glib::clone;

use crate::{Application, utils::TemplateCallbacks, utils::WeekDay, widgets::window::Styling};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Era/year_view_cell.ui")]
    #[properties(wrapper_type = super::YearViewCell)]
    pub struct YearViewCell {
        #[property(get, set)]
        year: Cell<i32>,
        #[property(get, construct_only)]
        month: Cell<i32>,
        #[property(get, set, construct, builder(Styling::default()))]
        styling: Cell<Styling>,

        #[template_child]
        month_label: TemplateChild<gtk::Label>,
        #[template_child]
        days_grid: TemplateChild<gtk::Grid>,

        spacing: Cell<i32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for YearViewCell {
        const NAME: &'static str = "YearViewCell";
        type Type = super::YearViewCell;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
            TemplateCallbacks::bind_template_callbacks(klass);

            klass.set_css_name("year-view-cell");
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for YearViewCell {
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

            obj.connect_styling_notify(|obj| {
                obj.imp().update_styling();
            });

            for cell in 0..42 {
                let label = gtk::Label::new(None);
                label.add_css_class("numeric");
                self.days_grid.attach(&label, cell % 7, cell / 7, 1, 1);
            }

            let application = Application::default();
            let dt = application.current_datetime();
            self.update_month_label_color(dt.year(), dt.month());

            obj.connect_year_notify(clone!(
                #[weak]
                application,
                move |obj| {
                    let dt = application.current_datetime();
                    obj.imp().update_month_label_color(dt.year(), dt.month());
                    obj.imp()
                        .update_day_label_color(dt.year(), dt.month(), dt.day_of_month());
                }
            ));

            obj.connect_month_notify(clone!(
                #[weak]
                application,
                move |obj| {
                    let dt = application.current_datetime();
                    obj.imp().update_month_label_color(dt.year(), dt.month());
                    obj.imp()
                        .update_day_label_color(dt.year(), dt.month(), dt.day_of_month());
                }
            ));

            application.connect_current_datetime_notify(clone!(
                #[weak(rename_to = imp)]
                self,
                move |application| {
                    let dt = application.current_datetime();
                    imp.update_month_label_color(dt.year(), dt.month());
                    imp.update_day_label_color(dt.year(), dt.month(), dt.day_of_month());
                }
            ));

            let system = application.system();
            system.connect_first_day_of_week_notify(clone!(
                #[weak(rename_to = imp)]
                self,
                #[weak]
                application,
                move |_| {
                    let dt = application.current_datetime();
                    imp.set_days_grid();
                    imp.update_day_label_color(dt.year(), dt.month(), dt.day_of_month());
                }
            ));
        }

        fn dispose(&self) {
            self.month_label.unparent();
            self.days_grid.unparent();
        }
    }

    impl WidgetImpl for YearViewCell {
        fn measure(&self, orientation: gtk::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
            match orientation {
                gtk::Orientation::Horizontal => {
                    let (month_label_minimum_width, ..) = self
                        .month_label
                        .measure(gtk::Orientation::Horizontal, for_size);
                    let (days_grid_minimum_width, days_grid_natural_width, ..) =
                        self.days_grid.layout_manager().unwrap().measure(
                            &self.days_grid.get(),
                            gtk::Orientation::Horizontal,
                            for_size,
                        );

                    let minimum_width =
                        cmp::max(month_label_minimum_width, days_grid_minimum_width);
                    let natural_width = days_grid_natural_width;

                    (minimum_width, natural_width, -1, -1)
                }
                gtk::Orientation::Vertical => {
                    let spacing = self.spacing.get();

                    let (month_label_minimum_height, month_label_natural_height, ..) = self
                        .month_label
                        .measure(gtk::Orientation::Vertical, for_size);
                    let (days_grid_minimum_height, days_grid_natural_height, ..) = self
                        .days_grid
                        .layout_manager()
                        .unwrap()
                        .measure(&self.days_grid.get(), gtk::Orientation::Vertical, for_size);

                    let minimum_height =
                        month_label_minimum_height + spacing + days_grid_minimum_height;
                    let natural_height =
                        month_label_natural_height + spacing + days_grid_natural_height;

                    (minimum_height, natural_height, -1, -1)
                }
                _ => unreachable!(),
            }
        }

        fn size_allocate(&self, width: i32, _height: i32, baseline: i32) {
            let spacing = self.spacing.get();

            let (label_height, ..) = self.month_label.measure(gtk::Orientation::Vertical, width);

            let (days_grid_height, ..) = self.days_grid.layout_manager().unwrap().measure(
                &self.days_grid.get(),
                gtk::Orientation::Vertical,
                -1,
            );
            let (days_grid_width, ..) = self.days_grid.layout_manager().unwrap().measure(
                &self.days_grid.get(),
                gtk::Orientation::Horizontal,
                -1,
            );

            let label_width = days_grid_width;

            let side_spacing = (width - label_width) / 2;
            let month_label_allocation =
                gtk::Allocation::new(side_spacing, 0, label_width, label_height);
            self.month_label
                .size_allocate(&month_label_allocation, baseline);
            let days_grid_allocation = gtk::Allocation::new(
                side_spacing,
                label_height + spacing,
                days_grid_width,
                days_grid_height,
            );
            self.days_grid
                .size_allocate(&days_grid_allocation, baseline);
        }
    }

    #[gtk::template_callbacks]
    impl YearViewCell {
        /// Convert WeekDay to a numeric offset (0 = Monday, 6 = Sunday).
        fn first_day_of_week_offset(first_day_of_week: WeekDay) -> usize {
            match first_day_of_week {
                WeekDay::Monday => 0,
                WeekDay::Tuesday => 1,
                WeekDay::Wednesday => 2,
                WeekDay::Thursday => 3,
                WeekDay::Friday => 4,
                WeekDay::Saturday => 5,
                WeekDay::Sunday => 6,
            }
        }

        fn set_days_grid(&self) {
            let year = self.year.get();
            let month = self.month.get();
            let first_day = jiff::civil::date(year as i16, month as i8, 1);
            let days_in_month = first_day.days_in_month() as usize;

            // Get the first day of week from system state
            let application = Application::default();
            let system = application.system();
            let first_day_of_week = system.first_day_of_week();
            let first_day_offset = Self::first_day_of_week_offset(first_day_of_week);

            // Calculate the position of the first day of the month in the grid
            let weekday_of_first_day = first_day.weekday().to_monday_zero_offset() as usize;
            let weekday_of_first_day = (weekday_of_first_day + 7 - first_day_offset) % 7;

            let mut cells = 0..42;

            for cell in cells.by_ref().take(weekday_of_first_day) {
                let label = self
                    .days_grid
                    .child_at(cell % 7, cell / 7)
                    .expect("Grid should be initialized")
                    .downcast::<gtk::Label>()
                    .expect("Widget should be a label");
                label.set_label("");
            }

            for (day_number, cell) in cells.by_ref().take(days_in_month).enumerate() {
                let label = self
                    .days_grid
                    .child_at(cell % 7, cell / 7)
                    .expect("Grid should be initialized")
                    .downcast::<gtk::Label>()
                    .expect("Widget should be a label");
                label.set_label(&format!("{}", day_number + 1));
            }

            for cell in cells {
                let label = self
                    .days_grid
                    .child_at(cell % 7, cell / 7)
                    .expect("Grid should be initialized")
                    .downcast::<gtk::Label>()
                    .expect("Widget should be a label");
                label.set_label("");
            }
        }

        fn update_styling(&self) {
            self.month_label.remove_css_class("caption-heading");
            self.month_label.remove_css_class("title-4");
            for cell in 0..42 {
                if let Some(label) = self.days_grid.child_at(cell % 7, cell / 7) {
                    label.remove_css_class("narrow");
                    label.remove_css_class("medium");
                    label.remove_css_class("wide");
                };
            }

            match self.styling.get() {
                Styling::Narrow => {
                    self.month_label.add_css_class("caption-heading");
                    self.days_grid.set_column_spacing(3);
                    self.days_grid.set_row_spacing(3);
                    self.spacing.set(6);
                    for cell in 0..42 {
                        if let Some(label) = self.days_grid.child_at(cell % 7, cell / 7) {
                            label.add_css_class("narrow");
                        };
                    }
                }
                Styling::Medium => {
                    self.month_label.add_css_class("title-4");
                    self.days_grid.set_column_spacing(12);
                    self.days_grid.set_row_spacing(12);
                    self.spacing.set(12);
                    for cell in 0..42 {
                        if let Some(label) = self.days_grid.child_at(cell % 7, cell / 7) {
                            label.add_css_class("medium");
                        };
                    }
                }
                Styling::Wide => {
                    self.month_label.add_css_class("title-4");
                    self.days_grid.set_column_spacing(12);
                    self.days_grid.set_row_spacing(12);
                    self.spacing.set(12);
                    for cell in 0..42 {
                        if let Some(label) = self.days_grid.child_at(cell % 7, cell / 7) {
                            label.add_css_class("wide");
                        }
                    }
                }
            }
        }

        fn update_month_label_color(&self, current_year: i32, current_month: i32) {
            if self.year.get() == current_year && self.month.get() == current_month {
                self.month_label.add_css_class("accent");
            } else {
                self.month_label.remove_css_class("accent");
            }
        }

        fn update_day_label_color(&self, current_year: i32, current_month: i32, current_day: i32) {
            let year = self.year.get();
            let month = self.month.get();

            for cell in 0..42 {
                let label = self
                    .days_grid
                    .child_at(cell % 7, cell / 7)
                    .expect("Grid should be initialized");
                label.remove_css_class("accent");
            }

            if year == current_year && month == current_month {
                let first_day = jiff::civil::date(year as i16, month as i8, 1);

                // Get the first day of week from system state
                let application = Application::default();
                let system = application.system();
                let first_day_of_week = system.first_day_of_week();
                let first_day_offset = Self::first_day_of_week_offset(first_day_of_week);

                // Calculate the position accounting for first day of week setting
                let weekday_of_first_day = first_day.weekday().to_monday_zero_offset() as usize;
                let weekday_of_first_day = (weekday_of_first_day + 7 - first_day_offset) % 7;

                let current_day_cell_number = weekday_of_first_day as i32 + current_day - 1;
                let current_day_label = self
                    .days_grid
                    .child_at(current_day_cell_number % 7, current_day_cell_number / 7)
                    .expect("Grid should be initialized");
                current_day_label.add_css_class("accent");
            }
        }
    }
}

glib::wrapper! {
    pub struct YearViewCell(ObjectSubclass<imp::YearViewCell>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl YearViewCell {
    pub fn new(year: i32, month: i32) -> Self {
        glib::Object::builder()
            .property("year", year)
            .property("month", month)
            .build()
    }
}
