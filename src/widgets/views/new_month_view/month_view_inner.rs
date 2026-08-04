use std::cell::{Cell, OnceCell};

use adw::{prelude::*, subclass::prelude::*};
use glib::clone;
use jiff::ToSpan;

use crate::{application::Application, utils::Date, widgets::window::Styling};

use super::month_view_cell::NewMonthViewCell;

const NB_ROWS: usize = 200;
const NB_ROWS_ABOVE_AT_STARTUP: usize = 10;

/// Minimum height of a cell in pixel. A cell can report a higher minimum and size_allocate will
/// respect it, but will never allocate them less than this.
/// This is a hard minimum.
const MINIMUM_CELL_HEIGHT: i32 = 10;
/// Maximum height of a cell in pixel. Ignored if the cell's own minimum height exceeds this, in
/// which case size_allocate allocates the cell's minimum height instead.
const MAXIMUM_CELL_HEIGHT: i32 = 300;

const _: () = assert!(
    MAXIMUM_CELL_HEIGHT >= MINIMUM_CELL_HEIGHT,
    "MAXIMUM_CELL_HEIGHT must be greater than or equal to MINIMUM_CELL_HEIGHT"
);

const EVENT_GAP: i32 = 2;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Era/new_month_view_inner.ui")]
    #[properties(wrapper_type = super::NewMonthViewInner)]
    pub struct NewMonthViewInner {
        #[property(get, set)]
        date: Cell<Date>,
        #[property(get, set = Self::set_styling, construct, builder(Styling::default()))]
        styling: Cell<Styling>,

        #[template_child]
        scroll_drag: TemplateChild<gtk::GestureDrag>,
        #[template_child]
        scroll_swipe: TemplateChild<gtk::GestureSwipe>,

        /// Number of pixels from the top of the first cell to the top of the widget. Value should
        /// be positive, meaning the top rows are scrolled off-screen upward.
        scroll_offset: Cell<f64>,

        /// The height that was given to each row during the last size_allocate. Should never be
        /// zero after constructed is ran.
        /// The row height doesn't include the separator height.
        cell_height: Cell<i32>,

        /// The desired row height for the next size_allocate. It shouldn't be set to a value
        /// bigger than MAXIMUM_ROW_HEIGHT. size_allocate might give the rows more height than
        /// this, to respect the rows measurements and MINIMUM_ROW_HEIGHT.
        /// This value is only set in stone once size_allocate is called. Before that point, it can
        /// be set many times.
        desired_cell_height: Cell<i32>,

        cells: OnceCell<[NewMonthViewCell; 7 * NB_ROWS]>,

        column_separators: OnceCell<[gtk::Separator; 6]>,

        row_separators: OnceCell<[gtk::Separator; NB_ROWS]>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NewMonthViewInner {
        const NAME: &'static str = "NewMonthViewInner";
        type Type = super::NewMonthViewInner;
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
    impl ObjectImpl for NewMonthViewInner {
        fn constructed(&self) {
            self.parent_constructed();

            let application = Application::default();
            let first_week_day = application.system().first_week_day();
            let today = application.system().date();

            let first_cell_date = today
                .previous_occurrence_of_weekday(first_week_day)
                .to_jiff()
                - (NB_ROWS_ABOVE_AT_STARTUP as i32 * 7).days();

            let cells = std::array::from_fn(|i| {
                let date = (first_cell_date + (i as i32).days()).into();
                let cell = NewMonthViewCell::new(date);
                cell.insert_before(&*self.obj(), None::<&gtk::Widget>);
                cell
            });

            let column_separators = std::array::from_fn(|_| {
                let separator = gtk::Separator::new(gtk::Orientation::Vertical);
                separator.insert_before(&*self.obj(), None::<&gtk::Widget>);
                separator
            });

            let row_separators = std::array::from_fn(|_| {
                let separator = gtk::Separator::new(gtk::Orientation::Horizontal);
                separator.insert_before(&*self.obj(), None::<&gtk::Widget>);
                separator
            });

            // TODO: use a real event widget instead
            let event_height = 30;
            let initial_cell_height = 3 * event_height + 2 * EVENT_GAP;
            self.cell_height.set(initial_cell_height);
            self.desired_cell_height.set(initial_cell_height);

            let (separator_height, ..) = row_separators[0].measure(gtk::Orientation::Vertical, 200);

            self.scroll_offset.set(
                (NB_ROWS_ABOVE_AT_STARTUP as i32 * (initial_cell_height + separator_height)) as f64,
            );

            self.cells.set(cells).unwrap();
            self.column_separators.set(column_separators).unwrap();
            self.row_separators.set(row_separators).unwrap();

            self.scroll_swipe.group_with(&*self.scroll_drag);

            application.system().connect_first_week_day_notify(clone!(
                #[weak(rename_to = _imp)]
                self,
                move |_system| unimplemented!()
            ));
        }

        fn dispose(&self) {
            for cell in self.cells.get().unwrap() {
                cell.unparent();
            }
            for separator in self.column_separators.get().unwrap() {
                separator.unparent();
            }
            for separator in self.row_separators.get().unwrap() {
                separator.unparent();
            }
        }
    }

    impl WidgetImpl for NewMonthViewInner {
        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            let cells = self.cells.get().unwrap();
            let column_separators = self.column_separators.get().unwrap();
            let row_separators = self.row_separators.get().unwrap();

            let (separator_width, ..) =
                column_separators[0].measure(gtk::Orientation::Horizontal, height);

            let (separator_height, ..) =
                row_separators[0].measure(gtk::Orientation::Vertical, width);

            // Width is distributed evenly. Any remainder pixels are given to the leftmost columns,
            // so the first (width % 7) columns are one pixel ider than the rest.
            let column_widths: [i32; 7] = {
                let width = width - 6 * separator_width;
                let base = width / 7;
                let remainder = width % 7;

                std::array::from_fn(|i| base + i32::from((i as i32) < remainder))
            };

            let column_xs: [i32; 7] = {
                std::array::from_fn(|i| {
                    column_widths[..i].iter().sum::<i32>() + i as i32 * separator_width
                })
            };

            let column_separator_xs: [i32; 6] = {
                std::array::from_fn(|i| {
                    column_widths[..i + 1].iter().sum::<i32>() + i as i32 * separator_width
                })
            };

            let cell_height = {
                // Measures against the narrowest column (the last one) to get a conservative
                // minimum height that holds for all cells regardless of their width.
                let (minimum_cell_height, ..) =
                    cells[0].measure(gtk::Orientation::Horizontal, column_widths[6]);

                let floored_minimum_cell_height = minimum_cell_height.max(MINIMUM_CELL_HEIGHT);

                if floored_minimum_cell_height > MAXIMUM_CELL_HEIGHT {
                    floored_minimum_cell_height
                } else {
                    self.desired_cell_height
                        .get()
                        .clamp(floored_minimum_cell_height, MAXIMUM_CELL_HEIGHT)
                }
            };

            self.cell_height.set(cell_height);
            self.desired_cell_height.set(cell_height);

            let scroll_offset = self.scroll_offset.get() as i32;

            for row_index in 0..NB_ROWS {
                let row_height = cell_height + separator_height;
                let cell_y = -scroll_offset + row_index as i32 * row_height;
                let separator_y = cell_y + cell_height;

                let row_visible = (cell_y + row_height) > 0;

                for column_index in 0..7 {
                    let cell = &cells[row_index * 7 + column_index];

                    if row_visible {
                        cell.set_child_visible(true);
                        let cell_x = column_xs[column_index];
                        let cell_width = column_widths[column_index];
                        let cell_allocation =
                            gtk::Allocation::new(cell_x, cell_y, cell_width, cell_height);
                        cell.size_allocate(&cell_allocation, baseline);
                    } else {
                        cell.set_child_visible(false);
                    }
                }

                let row_separator = &row_separators[row_index];
                if row_visible {
                    row_separator.set_child_visible(true);
                    let separator_allocation =
                        gtk::Allocation::new(0, separator_y, width, separator_height);
                    row_separator.size_allocate(&separator_allocation, baseline);
                } else {
                    row_separator.set_child_visible(false);
                }
            }

            for (i, column_separator) in column_separators.iter().enumerate() {
                let separator_x = column_separator_xs[i];
                let separator_allocation =
                    gtk::Allocation::new(separator_x, 0, separator_width, height);
                column_separator.size_allocate(&separator_allocation, baseline);
            }
        }
    }

    #[gtk::template_callbacks]
    impl NewMonthViewInner {
        /// Sets the styling used for the view.
        fn set_styling(&self, styling: Styling) {
            if self.styling.get() == styling {
                return;
            }

            self.styling.set(styling);
            for cell in self.cells.get().unwrap() {
                cell.set_styling(styling);
            }
            self.obj().notify_styling();
        }

        #[template_callback]
        fn kinetic_scroll_begin(&self) {}

        #[template_callback]
        fn kinetic_scroll(
            &self,
            _dx: f64,
            _dy: f64,
            _controller: gtk::EventControllerScroll,
        ) -> bool {
            true
        }

        #[template_callback]
        fn kinetic_scroll_decelerate(&self, _dx: f64, _dy: f64) {}

        #[template_callback]
        fn discrete_scroll(
            &self,
            _dx: f64,
            _dy: f64,
            _controller: gtk::EventControllerScroll,
        ) -> bool {
            true
        }

        #[template_callback]
        fn scroll_drag_begin(&self, _start_x: f64, _start_y: f64, _gesture_drag: gtk::GestureDrag) {
        }

        #[template_callback]
        fn scroll_drag_update(
            &self,
            _offset_x: f64,
            _offset_y: f64,
            _gesture_drag: gtk::GestureDrag,
        ) {
        }

        #[template_callback]
        fn swipe(&self, _dx: f64, _dy: f64) {}
    }
}

glib::wrapper! {
    pub struct NewMonthViewInner(ObjectSubclass<imp::NewMonthViewInner>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl NewMonthViewInner {}
