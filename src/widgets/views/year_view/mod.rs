use std::{
    cell::{Cell, OnceCell, RefCell},
    mem,
    sync::{LazyLock, Mutex},
};

use adw::{prelude::*, subclass::prelude::*};
use glib::{clone, subclass::Signal};
use gtk::Allocation;

use crate::Application;

mod year_view_cell;
mod year_view_row;

use self::{year_view_cell::YearViewCell, year_view_row::YearViewRow};

const NB_ROWS: i32 = 30;
const MINIMUM_NB_ROWS_ABOVE: i32 = 5;
const MINIMUM_NB_ROWS_BELOW: i32 = 5;

const VELOCITY_THRESHOLD_TO_RETURN: f64 = 300.;
const VELOCITY_THRESHOLD_TO_SNAP: f64 = 400.;
const VELOCITY_THRESHOLD_TO_SKIP: f64 = 2.;
const FIRST_STAGE_DIVISOR: f64 = 1.5;
const FIRST_TO_SECOND_STAGE_THRESHOLD: f64 = 300.;
const SECOND_STAGE_DIVISOR: f64 = 2.5;
const DISCRETE_SCROLL_DISTANCE_THRESHOLD_TO_SNAP: f64 = 100.;
const DISCRETE_SCROLL_DISTANCE_THRESHOLD_TO_ROW: f64 = 50.;

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

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Era/year_view.ui")]
    #[properties(wrapper_type = super::YearView)]
    pub struct YearView {
        #[property(get, set = Self::set_year)]
        year: Cell<i32>,
        #[property(get, set, builder(YearViewStyling::default()))]
        styling: Cell<YearViewStyling>,

        /// Rows contained in the view.
        rows: OnceCell<Mutex<Vec<YearViewRow>>>,
        /// The current height of a row.
        row_height: Cell<i32>,

        /// Offset of the top of the first row. This is also the number of pixels above the
        /// view that are not visible.
        scroll_offset: Cell<i32>,

        /// Currently running scroll animation, if any.
        scroll_animation: RefCell<Option<adw::TimedAnimation>>,
        /// Number of pixels the view should have moved by already, but because it is less than 1,
        /// it is still pending.
        pixels_waiting: Cell<f64>,
        /// Number of pixels to move by still to go.
        pixels_left: Cell<f64>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for YearView {
        const NAME: &'static str = "YearView";
        type Type = super::YearView;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();

            klass.set_css_name("year-view");
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

            let application = Application::default();
            let current_year = application.current_datetime().year();
            self.year.set(current_year);
            obj.notify_year();

            let mut rows = Vec::new();
            for year in
                current_year - MINIMUM_NB_ROWS_ABOVE..current_year - MINIMUM_NB_ROWS_ABOVE + NB_ROWS
            {
                let row = YearViewRow::new(year);
                obj.bind_property("styling", &row, "styling")
                    .sync_create()
                    .build();
                row.insert_before(&*self.obj(), None::<&gtk::Widget>);
                row.connect_month_clicked(clone!(
                    #[weak]
                    obj,
                    move |_row, year, month| {
                        obj.emit_by_name::<()>("month-clicked", &[&year, &month]);
                    }
                ));
                rows.push(row);
            }
            self.rows.get_or_init(|| Mutex::new(rows));
        }

        fn dispose(&self) {
            for row in self.rows.get().unwrap().lock().unwrap().iter() {
                row.unparent();
            }
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: LazyLock<Vec<Signal>> = LazyLock::new(|| {
                vec![
                    Signal::builder("month-clicked")
                        // Year, Month
                        .param_types([i32::static_type(), i32::static_type()])
                        .build(),
                ]
            });
            SIGNALS.as_ref()
        }
    }

    impl WidgetImpl for YearView {
        fn size_allocate(&self, width: i32, _height: i32, baseline: i32) {
            let rows = self.rows.get().unwrap().lock().unwrap();
            let first_row = rows
                .first()
                .expect("There should be at least one year row")
                .to_owned();
            let (row_height, ..) = first_row.measure(gtk::Orientation::Vertical, width);

            // If the row height changes, the scroll offset is meaningless. As a consequence we snap
            // to the row of the current displayed year.
            if self.row_height.get() != row_height {
                let first_row_year = first_row.year();
                let nb_rows_above = self.year.get() - first_row_year;

                for (i, row) in rows.iter().enumerate() {
                    let allocation = Allocation::new(
                        0,
                        (-nb_rows_above + i as i32) * row_height,
                        width,
                        row_height,
                    );
                    row.size_allocate(&allocation, baseline);
                }

                self.scroll_offset.set(nb_rows_above * row_height);
            } else {
                let scroll_offset = self.scroll_offset.get();

                for (i, row) in rows.iter().enumerate() {
                    let allocation = Allocation::new(
                        0,
                        -scroll_offset + i as i32 * row_height,
                        width,
                        row_height,
                    );
                    row.size_allocate(&allocation, baseline);
                }
            }

            self.row_height.set(row_height);
        }
    }

    #[gtk::template_callbacks]
    impl YearView {
        /// Sets the year.
        fn set_year(&self, year: i32) {
            self.cancel_scroll_animation();

            if self.year.get() != year {
                self.year.set(year);
                self.obj().notify_year();
            }

            self.update_view_to_stored_date();
        }

        /// Gets the rows.
        fn rows(&self) -> &Mutex<Vec<YearViewRow>> {
            self.rows.get().expect("Rows should be initialized")
        }

        /// Updates the view to the currently stored date.
        fn update_view_to_stored_date(&self) {
            let base_year = self.year.get() - MINIMUM_NB_ROWS_ABOVE;

            for (i, row) in self.rows().lock().unwrap().iter().enumerate() {
                row.set_year(base_year + i as i32);
            }

            let row_height = self.row_height.get();
            let current_offset = self.scroll_offset.get();
            self.scroll_offset_add(MINIMUM_NB_ROWS_ABOVE * row_height - current_offset);
        }

        /// Instantly moves the view by the given number of pixels.
        ///
        /// If necessary, rows will be recycled and the offset will get adjusted. Year/month/day
        /// properties will be updated.
        fn scroll_offset_add(&self, dy: i32) {
            // The desired offset of the top of the first row
            let top_offset = self.scroll_offset.get() + dy;
            self.set_scroll_offset(top_offset);
        }

        /// Move the view by the given number of pixels.
        fn set_scroll_offset(&self, scroll_offset: i32) {
            let height = self.obj().height();
            let mut rows = self.rows.get().unwrap().lock().unwrap();

            let row_height = self.row_height.get();
            // The limit of the top offset before recycling happens
            let top_threshold = row_height * MINIMUM_NB_ROWS_ABOVE;
            // The limit of the bottom offset before recycling happens
            let bottom_threshold = (NB_ROWS - MINIMUM_NB_ROWS_BELOW) * row_height;

            // Recycle a row if necessary and set the new scroll offset
            // TODO: Recycle multiple rows if needed
            if scroll_offset < top_threshold {
                self.scroll_offset.set(scroll_offset + row_height);

                let first_row = rows
                    .first()
                    .expect("There should be at least one year row")
                    .to_owned();
                let last_row = rows.pop().unwrap();
                last_row.set_year(first_row.year() - 1);

                rows.insert(0, last_row);
            } else if scroll_offset + height > bottom_threshold {
                self.scroll_offset.set(scroll_offset - row_height);

                let first_row = rows.remove(0);
                let last_row = rows
                    .last()
                    .expect("There should be at least one year row")
                    .clone();
                first_row.set_year(last_row.year() + 1);

                rows.push(first_row);
            } else {
                self.scroll_offset.set(scroll_offset);
            }

            mem::drop(rows);

            self.update_date();

            self.obj().queue_allocate();
        }

        /// Updates the view's date to the one of the most top visible row.
        fn update_date(&self) {
            let rows = self.rows().lock().unwrap();
            let row_height = self.row_height.get();

            let highest_visible_row = rows
                .get((self.scroll_offset.get() / row_height) as usize)
                .unwrap()
                .clone();

            let year = highest_visible_row.year();

            if self.year.get() != year {
                self.year.set(year);
                self.obj().notify_year();
            }
        }

        /// Start a scroll animation.
        ///
        /// The animation will move the view by the given amount of pixels over a certain amount of
        /// time.
        fn start_scroll_animation(&self, pixels: f64) {
            let obj = self.obj();

            self.cancel_scroll_animation();

            self.pixels_left.set(pixels);

            let duration_ms = pixels.abs() / obj.height() as f64 * 1000.;
            let animation_target = adw::CallbackAnimationTarget::new(clone!(
                #[weak(rename_to = imp)]
                self,
                move |pixels_to_leave| {
                    let pixels_left = imp.pixels_left.get();
                    let pixels_waiting = imp.pixels_waiting.get();

                    // If there are not enough pixels to leave, leave no pixel for later and skip
                    // the animation.
                    // There is still a little jump left to fix at the end of the animation.
                    // TODO: Fix the little jump - maybe switch to how scroll view does inertia
                    if pixels_to_leave.abs() < VELOCITY_THRESHOLD_TO_SKIP
                        && let Some(animation) = imp.scroll_animation.borrow().as_ref()
                    {
                        imp.pixels_left.set(0.);
                        imp.pixels_waiting.set(0.);
                        imp.scroll_offset_add((pixels_left + pixels_waiting) as i32);

                        animation.skip();
                        return;
                    }

                    let dy = pixels_left - pixels_to_leave + pixels_waiting;
                    imp.pixels_left.set(pixels_to_leave);

                    let move_by = dy.trunc() as i32;
                    imp.pixels_waiting.set(dy.fract());
                    imp.scroll_offset_add(move_by);
                }
            ));
            let kinetic_scroll_animation =
                adw::TimedAnimation::new(&*obj, pixels, 0., duration_ms as u32, animation_target);
            kinetic_scroll_animation.set_easing(adw::Easing::EaseOutExpo);
            kinetic_scroll_animation.set_follow_enable_animations_setting(false);

            self.scroll_animation
                .replace(Some(kinetic_scroll_animation.clone()));
            kinetic_scroll_animation.play();
        }

        /// Cancel the current scroll animation, if any.
        fn cancel_scroll_animation(&self) {
            if let Some(kinetic_scroll_animation) = self.scroll_animation.borrow().as_ref() {
                kinetic_scroll_animation.pause();
            }
            self.scroll_animation.replace(None);
        }

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
        fn kinetic_scroll_begin(&self) {
            // TODO: Don't cancel if scrolling should not be handled?
            self.cancel_scroll_animation();
        }

        #[template_callback]
        fn kinetic_scroll(
            &self,
            _dx: f64,
            dy: f64,
            controller: gtk::EventControllerScroll,
        ) -> bool {
            // For kinetic scrolling, we only want to handle smooth events. Don't handle discrete
            // events.
            match controller
                .current_event()
                .expect("Controller should have a current event")
                .downcast::<gdk::ScrollEvent>()
                .expect("A scroll controller should only have a scroll event")
                .direction()
            {
                gdk::ScrollDirection::Up | gdk::ScrollDirection::Down => return false,
                gdk::ScrollDirection::Smooth => (),
                gdk::ScrollDirection::Left | gdk::ScrollDirection::Right => {
                    panic!("Vertical scroll controller should not signal horizontal scroll events")
                }
                direction => panic!("Unknown scroll direction: {direction:?}"),
            }

            self.cancel_scroll_animation();
            self.scroll_offset_add(dy as i32);

            true
        }

        #[template_callback]
        fn kinetic_decelerate(&self, _dx: f64, dy: f64) {
            // TODO: Do we have to check again if the event is smooth?
            if dy.abs() < VELOCITY_THRESHOLD_TO_RETURN {
                return;
            }

            // Apply a function to the speed so that the animation feels more natural
            let dy = if dy > FIRST_TO_SECOND_STAGE_THRESHOLD {
                FIRST_TO_SECOND_STAGE_THRESHOLD / FIRST_STAGE_DIVISOR
                    + (dy - FIRST_TO_SECOND_STAGE_THRESHOLD) / SECOND_STAGE_DIVISOR
            } else if dy < -FIRST_TO_SECOND_STAGE_THRESHOLD {
                -FIRST_TO_SECOND_STAGE_THRESHOLD / FIRST_STAGE_DIVISOR
                    + (dy + FIRST_TO_SECOND_STAGE_THRESHOLD) / SECOND_STAGE_DIVISOR
            } else {
                dy / FIRST_STAGE_DIVISOR
            };

            let row_height = self.row_height.get();
            let scroll_offset = self.scroll_offset.get();
            let offset_from_a_row = scroll_offset % row_height;

            // Adjust the scroll to snap to the start of a row if close enough
            let pixels = if dy > VELOCITY_THRESHOLD_TO_SNAP {
                (dy / row_height as f64).floor() * row_height as f64 + row_height as f64
                    - offset_from_a_row as f64
            } else if dy < -VELOCITY_THRESHOLD_TO_SNAP {
                (dy / row_height as f64).ceil() * row_height as f64 - offset_from_a_row as f64
            } else {
                dy
            };
            self.start_scroll_animation(pixels);
        }

        #[template_callback]
        fn discrete_scroll(
            &self,
            _dx: f64,
            dy: f64,
            controller: gtk::EventControllerScroll,
        ) -> bool {
            // TODO: Add up to already ongoing discrete originating scroll animation?
            // Maybe we could make so that 5 ticks on a mouse always move the view by 5 units, even
            // if they are done fast

            match controller
                .current_event()
                .expect("Controller should have a current event")
                .downcast::<gdk::ScrollEvent>()
                .expect("A scroll controller should only have a scroll event")
                .direction()
            {
                gdk::ScrollDirection::Up | gdk::ScrollDirection::Down => (),
                gdk::ScrollDirection::Smooth => return false,
                gdk::ScrollDirection::Left | gdk::ScrollDirection::Right => {
                    panic!("Vertical scroll controller should not signal horizontal scroll events")
                }
                direction => panic!("Unknown scroll direction: {direction:?}"),
            }

            let height = self.obj().height();
            let row_height = self.row_height.get();
            let scroll_offset = self.scroll_offset.get();
            let number_of_scroll_steps = dy;
            let distance_to_previous_row_start = scroll_offset % row_height;
            let distance_to_next_row_start = row_height - distance_to_previous_row_start;

            // If one year row does not fit in the view, scroll by the biggest slice of a row that
            // fits in the view
            if row_height > height {
                let number_of_slices_per_row = (row_height as f64 / height as f64).ceil() as i32;
                let slice_height = row_height as f64 / number_of_slices_per_row as f64;
                let scroll_request = number_of_scroll_steps * slice_height;
                let upcoming_distance_to_previous_row_start =
                    (scroll_offset as f64 + scroll_request) % row_height as f64;
                let upcoming_distance_to_next_row_start =
                    row_height as f64 - upcoming_distance_to_previous_row_start;

                // Adjust the scroll to snap to the start of a row if close enough
                let pixels = if (upcoming_distance_to_previous_row_start)
                    < DISCRETE_SCROLL_DISTANCE_THRESHOLD_TO_SNAP
                {
                    scroll_request - upcoming_distance_to_previous_row_start
                } else if (upcoming_distance_to_next_row_start)
                    < DISCRETE_SCROLL_DISTANCE_THRESHOLD_TO_SNAP
                {
                    scroll_request + upcoming_distance_to_next_row_start
                } else {
                    scroll_request
                };
                self.start_scroll_animation(pixels);

                return true;
            }

            // Else, scroll to the next year row start
            let pixels = if number_of_scroll_steps < 0. {
                let mut pixels = number_of_scroll_steps * row_height as f64
                    - distance_to_previous_row_start as f64;
                if distance_to_previous_row_start as f64 > DISCRETE_SCROLL_DISTANCE_THRESHOLD_TO_ROW
                {
                    pixels += row_height as f64;
                }
                pixels
            } else {
                let mut pixels =
                    number_of_scroll_steps * row_height as f64 + distance_to_next_row_start as f64;
                if distance_to_next_row_start as f64 > DISCRETE_SCROLL_DISTANCE_THRESHOLD_TO_ROW {
                    pixels -= row_height as f64;
                }
                pixels
            };
            self.start_scroll_animation(pixels);

            true
        }

        #[template_callback]
        fn swipe(&self, _dx: f64, dy: f64) {
            let pixels = -dy;
            self.start_scroll_animation(pixels);
        }
    }
}

glib::wrapper! {
    pub struct YearView(ObjectSubclass<imp::YearView>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}
