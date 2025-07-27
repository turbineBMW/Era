use std::{cell::Cell, cmp, sync::LazyLock};

use adw::{prelude::*, subclass::prelude::*};
use gtk::glib::{self, clone, closure_local, subclass::Signal};

use crate::CalendarManagerApplication;

use super::{YearViewMonthCell, YearViewStyling};

const SIDE_MARGIN: i32 = 12;
const SPACING: i32 = 6;

pub(crate) mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/CalendarManager/year_view_year_row.ui")]
    #[properties(wrapper_type = super::YearViewYearRow)]
    pub struct YearViewYearRow {
        #[property(get, set)]
        year: Cell<i32>,
        #[property(get, set, builder(YearViewStyling::default()))]
        styling: Cell<YearViewStyling>,
        #[template_child]
        year_label: TemplateChild<gtk::Label>,
        #[template_child]
        separator: TemplateChild<gtk::Separator>,
        #[template_child]
        month_flow_box: TemplateChild<gtk::FlowBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for YearViewYearRow {
        const NAME: &'static str = "YearViewYearRow";
        type Type = super::YearViewYearRow;
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
    impl ObjectImpl for YearViewYearRow {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            let year = self.year.get();
            for month in 1..=12 {
                let cell = YearViewMonthCell::new(year, month);
                obj.bind_property("styling", &cell, "styling")
                    .sync_create()
                    .build();
                obj.bind_property("year", &cell, "year")
                    .sync_create()
                    .build();
                self.month_flow_box.append(&cell);
            }

            obj.connect_styling_notify(|obj| {
                obj.imp().update_styling();
            });

            let application = CalendarManagerApplication::default();
            let current_year = application.current_year();
            self.update_year_label_color(current_year);

            obj.connect_year_notify(clone!(
                #[weak]
                application,
                move |obj| {
                    let current_year = application.current_year();
                    obj.imp().update_year_label_color(current_year);
                }
            ));

            application.connect_current_year_notify(clone!(
                #[weak(rename_to = imp)]
                self,
                move |application| {
                    let current_year = application.current_year();
                    imp.update_year_label_color(current_year);
                }
            ));
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

    impl WidgetImpl for YearViewYearRow {
        fn request_mode(&self) -> gtk::SizeRequestMode {
            gtk::SizeRequestMode::HeightForWidth
        }

        fn measure(&self, orientation: gtk::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
            match orientation {
                gtk::Orientation::Horizontal => {
                    let (year_label_minimum_width, year_label_natural_width, ..) = self
                        .year_label
                        .measure(gtk::Orientation::Horizontal, for_size);
                    let (separator_minimum_width, separator_natural_width, ..) = self
                        .separator
                        .measure(gtk::Orientation::Horizontal, for_size);
                    let (month_flow_box_minimum_width, month_flow_box_natural_width, ..) = self
                        .month_flow_box
                        .measure(gtk::Orientation::Horizontal, for_size);

                    let minimum_width = cmp::max(
                        year_label_minimum_width,
                        cmp::max(separator_minimum_width, month_flow_box_minimum_width),
                    );
                    let natural_width = cmp::max(
                        year_label_natural_width,
                        cmp::max(separator_natural_width, month_flow_box_natural_width),
                    );

                    (minimum_width, natural_width, -1, -1)
                }
                gtk::Orientation::Vertical => {
                    let for_size = for_size - SIDE_MARGIN * 2;
                    let (year_label_minimum_height, year_label_natural_height, ..) = self
                        .year_label
                        .measure(gtk::Orientation::Vertical, for_size);
                    let (separator_minimum_height, separator_natural_height, ..) =
                        self.separator.measure(gtk::Orientation::Vertical, for_size);
                    let (month_flow_box_minimum_height, month_flow_box_natural_height, ..) = self
                        .month_flow_box
                        .measure(gtk::Orientation::Vertical, for_size);

                    let minimum_height = year_label_minimum_height
                        + SPACING
                        + separator_minimum_height
                        + SPACING
                        + month_flow_box_minimum_height;
                    let natural_height = year_label_natural_height
                        + SPACING
                        + separator_natural_height
                        + SPACING
                        + month_flow_box_natural_height;

                    (minimum_height, natural_height, -1, -1)
                }
                _ => unreachable!(),
            }
        }

        // TODO: check if i have been allocated enough space
        fn size_allocate(&self, width: i32, _height: i32, baseline: i32) {
            let width = width - SIDE_MARGIN * 2;

            let (year_label_height, ..) =
                self.year_label.measure(gtk::Orientation::Vertical, width);
            let (separator_height, ..) = self.separator.measure(gtk::Orientation::Vertical, width);

            let (month_flow_box_height, ..) = self
                .month_flow_box
                .measure(gtk::Orientation::Vertical, width);

            let x_start = SIDE_MARGIN;

            let y_separator_start = year_label_height + SPACING;
            let y_grid_start = y_separator_start + separator_height + SPACING;

            self.year_label.size_allocate(
                &gtk::Allocation::new(x_start, 0, width, year_label_height),
                baseline,
            );
            self.separator.size_allocate(
                &gtk::Allocation::new(x_start, y_separator_start, width, separator_height),
                baseline,
            );
            self.month_flow_box.size_allocate(
                &gtk::Allocation::new(x_start, y_grid_start, width, month_flow_box_height),
                baseline,
            );
        }
    }

    #[gtk::template_callbacks]
    impl YearViewYearRow {
        fn update_styling(&self) {
            self.year_label.remove_css_class("title-3");
            self.year_label.remove_css_class("title-1");

            match self.styling.get() {
                YearViewStyling::Narrow => {
                    self.year_label.add_css_class("title-3");
                    self.month_flow_box.set_column_spacing(6);
                    self.month_flow_box.set_row_spacing(6);
                }
                YearViewStyling::Medium => {
                    self.year_label.add_css_class("title-1");
                    self.month_flow_box.set_column_spacing(12);
                    self.month_flow_box.set_row_spacing(12);
                }
                YearViewStyling::Wide => {
                    self.year_label.add_css_class("title-1");
                    self.month_flow_box.set_column_spacing(12);
                    self.month_flow_box.set_row_spacing(12);
                }
            }
        }

        fn update_year_label_color(&self, current_year: i32) {
            if self.year.get() == current_year {
                self.year_label.add_css_class("accent");
            } else {
                self.year_label.remove_css_class("accent");
            }
        }

        #[template_callback]
        fn get_year_label_narrow(&self) -> String {
            self.obj().year().to_string()
        }

        #[template_callback]
        fn month_cell_clicked(&self, _cell: gtk::FlowBoxChild) {
            // let cell = cell
            //     .upcast::<gtk::Widget>()
            //     .downcast::<YearViewMonthCell>()
            //     .unwrap();
            // self.obj()
            //     .emit_by_name::<()>("month-clicked", &[&cell.year(), &cell.month()]);
        }
    }
}

glib::wrapper! {
    pub struct YearViewYearRow(ObjectSubclass<imp::YearViewYearRow>)
        @extends gtk::Widget;
}

impl YearViewYearRow {
    pub fn new(year: i32) -> Self {
        glib::Object::builder().property("year", year).build()
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
