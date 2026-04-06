use std::{
    cell::{Cell, OnceCell, RefCell},
    mem,
    sync::Mutex,
};

use adw::{prelude::*, subclass::prelude::*};
use glib::clone;
use gtk::Allocation;
use jiff::ToSpan;

use crate::{Application, system_settings::FirstDayOfWeek};

use super::{
    MonthViewStyling,
    month_view_row::{self, MonthViewRow},
};

const NB_ROWS: i32 = 200;
const MINIMUM_NB_ROWS_ABOVE: i32 = 5;
const MINIMUM_NB_ROWS_BELOW: i32 = 5;

const MINIMUM_ROW_HEIGHT: i32 = month_view_row::MINIMUM_HEIGHT;
const NATURAL_ROW_HEIGHT: i32 = month_view_row::NATURAL_HEIGHT;

const VELOCITY_THRESHOLD_TO_RETURN: f64 = 300.;
const VELOCITY_THRESHOLD_TO_SNAP: f64 = 400.;
const VELOCITY_THRESHOLD_TO_SKIP: f64 = 2.;
const FIRST_STAGE_DIVISOR: f64 = 1.5;
const FIRST_TO_SECOND_STAGE_THRESHOLD: f64 = 300.;
const SECOND_STAGE_DIVISOR: f64 = 2.5;
const DISCRETE_SCROLL_DISTANCE_THRESHOLD_TO_SNAP: f64 = 100.;
const DISCRETE_SCROLL_DISTANCE_THRESHOLD_TO_ROW: f64 = 50.;

mod imp {
    use super::*;

    #[derive(Debug, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Kalendasom/month_view_inner.ui")]
    #[properties(wrapper_type = super::MonthViewInner)]
    pub struct MonthViewInner {
        #[property(get)]
        year: Cell<i32>,
        #[property(get)]
        month: Cell<i32>,
        #[property(get)]
        day: Cell<i32>,
        #[property(get, set, builder(MonthViewStyling::default()))]
        styling: Cell<MonthViewStyling>,

        /// Rows contained in the view.
        rows: OnceCell<Mutex<Vec<MonthViewRow>>>,

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

        /// Y position of the pointer to use for zooming with CTRL+scroll.
        pointer_y: Cell<Option<f64>>,
        /// Zoom level updated dynamically during a zoom operation.
        dynamic_zoom_level: Cell<f64>,
        /// Zoom level updated from the dynamic one when a zoom operation begins. It serves as a
        /// base reference when smooth zooming is occurring.
        static_zoom_level: Cell<f64>,
    }

    impl Default for MonthViewInner {
        fn default() -> Self {
            let rows = OnceCell::new();
            rows.get_or_init(|| Mutex::new((0..NB_ROWS).map(|_i| MonthViewRow::new()).collect()));

            let now = Application::default().current_datetime();
            let year = Cell::new(now.year());
            let month = Cell::new(now.month());
            let day = Cell::new(now.day_of_month());

            Self {
                year,
                month,
                day,
                styling: Default::default(),
                rows,
                scroll_offset: Cell::new(2 * NATURAL_ROW_HEIGHT),
                scroll_animation: Default::default(),
                pixels_waiting: Default::default(),
                pixels_left: Default::default(),
                pointer_y: Default::default(),
                dynamic_zoom_level: Cell::new(1.),
                static_zoom_level: Cell::new(1.),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MonthViewInner {
        const NAME: &'static str = "MonthViewInner";
        type Type = super::MonthViewInner;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();

            klass.set_css_name("month-view-inner");
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MonthViewInner {
        fn constructed(&self) {
            self.parent_constructed();

            // TODO: Validate year/month/row?

            for row in self.rows().lock().unwrap().iter() {
                row.insert_before(&*self.obj(), None::<&gtk::Widget>);
                self.obj()
                    .bind_property("styling", row, "styling")
                    .sync_create()
                    .build();
            }

            self.update_view_to_stored_date();

            Application::default()
                .system_settings()
                .connect_first_day_of_week_notify(clone!(
                    #[weak(rename_to = imp)]
                    self,
                    move |_| {
                        imp.update_date();
                    }
                ));
        }

        fn dispose(&self) {
            for row in self.rows().lock().unwrap().iter() {
                row.unparent();
            }
        }
    }

    impl WidgetImpl for MonthViewInner {
        fn size_allocate(&self, width: i32, _height: i32, baseline: i32) {
            let row_height = self.row_height();

            for (i, row) in self.rows().lock().unwrap().iter().enumerate() {
                let allocation = Allocation::new(
                    0,
                    -self.scroll_offset.get() + i as i32 * row_height,
                    width,
                    row_height,
                );
                row.size_allocate(&allocation, baseline);
            }
        }
    }

    #[gtk::template_callbacks]
    impl MonthViewInner {
        /// Sets the triplet year-month-day.
        pub(super) fn set_year_month_day(&self, year: i32, month: i32, day: i32) {
            self.cancel_scroll_animation();

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

            self.update_view_to_stored_date();
        }

        /// Gets the rows.
        fn rows(&self) -> &Mutex<Vec<MonthViewRow>> {
            self.rows.get().expect("Rows should be initialized")
        }

        /// Gets the row height. We assume all rows have the same height.
        fn row_height(&self) -> i32 {
            (NATURAL_ROW_HEIGHT as f64 * self.dynamic_zoom_level.get()) as i32
        }

        /// Updates the view to the currently stored date.
        fn update_view_to_stored_date(&self) {
            let base_date = jiff::civil::Date::new(
                self.year.get() as i16,
                self.month.get() as i8,
                self.day.get() as i8,
            )
            .unwrap()
            .checked_sub(MINIMUM_NB_ROWS_ABOVE.weeks())
            .unwrap();

            for (i, row) in self.rows().lock().unwrap().iter().enumerate() {
                let date = base_date.checked_add((i as i32).weeks()).unwrap();
                row.set_year_month_day(date.year() as i32, date.month() as i32, date.day() as i32);
            }

            let row_height = self.row_height();
            let current_offset = self.scroll_offset.get();
            self.scroll_offset_add(MINIMUM_NB_ROWS_ABOVE * row_height - current_offset);
        }

        /// Changes the offset to the given one.
        ///
        /// If necessary, rows will be recycled and the offset will get adjusted. Year/month/day
        /// properties will be updated.
        fn set_scroll_offset(&self, scroll_offset: i32) {
            let height = self.obj().height();
            let mut rows = self.rows().lock().unwrap();

            let row_height = self.row_height();
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
                    .expect("There should be at least one row")
                    .to_owned();
                let first_row_date = jiff::civil::Date::new(
                    first_row.year() as i16,
                    first_row.month() as i8,
                    first_row.day() as i8,
                )
                .unwrap();
                let new_last_row_date = first_row_date.checked_sub(1.week()).unwrap();

                let last_row = rows.pop().unwrap();

                last_row.set_year_month_day(
                    new_last_row_date.year() as i32,
                    new_last_row_date.month() as i32,
                    new_last_row_date.day() as i32,
                );

                rows.insert(0, last_row);
            } else if scroll_offset + height > bottom_threshold {
                self.scroll_offset.set(scroll_offset - row_height);

                let last_row = rows
                    .last()
                    .expect("There should be at least one row")
                    .clone();
                let last_row_date = jiff::civil::Date::new(
                    last_row.year() as i16,
                    last_row.month() as i8,
                    last_row.day() as i8,
                )
                .unwrap();
                let new_first_row_date = last_row_date.checked_add(1.week()).unwrap();

                let first_row = rows.remove(0);
                first_row.set_year_month_day(
                    new_first_row_date.year() as i32,
                    new_first_row_date.month() as i32,
                    new_first_row_date.day() as i32,
                );

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
            let row_height = self.row_height();

            let highest_visible_row = rows
                .get((self.scroll_offset.get() / row_height) as usize)
                .unwrap()
                .clone();

            let year = highest_visible_row.year();
            let month = highest_visible_row.month();
            let day = highest_visible_row.day();

            let date = jiff::civil::Date::new(year as i16, month as i8, day as i8).unwrap();

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

            let go_back_by = offset - base - 6;

            let last_day_of_the_row = date.checked_sub(go_back_by.days()).unwrap();
            let year = last_day_of_the_row.year() as i32;
            let month = last_day_of_the_row.month() as i32;
            let day = last_day_of_the_row.day() as i32;

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

        /// Starts a scroll animation.
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

        fn update_zoom(&self, y_center: f64, scale: f64) {
            let current_offset_of_gesture_center = self.scroll_offset.get() + y_center as i32;
            let current_zoom = self.dynamic_zoom_level.get();

            let max_ratio = NATURAL_ROW_HEIGHT as f64 / MINIMUM_ROW_HEIGHT as f64;
            let new_zoom =
                (self.static_zoom_level.get() + scale - 1.).clamp(1. / max_ratio, max_ratio);

            let ratio_of_zoom_difference = new_zoom / current_zoom;
            let new_offset_of_pointer =
                (current_offset_of_gesture_center as f64 * ratio_of_zoom_difference) as i32;
            let new_offset = new_offset_of_pointer - y_center as i32;

            self.set_scroll_offset(new_offset);
            self.dynamic_zoom_level.set(new_zoom);

            self.obj().queue_allocate();
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
        fn kinetic_scroll_decelerate(&self, _dx: f64, dy: f64) {
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

            let row_height = self.row_height();
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
            // Handle CTRL+scroll to zoom
            if controller
                .current_event_state()
                .contains(gdk::ModifierType::CONTROL_MASK)
            {
                match controller
                    .current_event()
                    .expect("Controller should have a current event")
                    .downcast::<gdk::ScrollEvent>()
                    .expect("A scroll controller should only have a scroll event")
                    .direction()
                {
                    gdk::ScrollDirection::Up | gdk::ScrollDirection::Down => {
                        let y_center = self
                            .pointer_y
                            .get()
                            .unwrap_or(self.obj().height() as f64 / 2.);

                        self.static_zoom_level.set(self.dynamic_zoom_level.get());

                        let scale = dy / 10.0 + 1.0;
                        self.update_zoom(y_center, scale);

                        return true;
                    }
                    gdk::ScrollDirection::Smooth => (),
                    gdk::ScrollDirection::Left | gdk::ScrollDirection::Right => {
                        panic!(
                            "Vertical scroll controller should not signal horizontal scroll events"
                        )
                    }
                    direction => panic!("Unknown scroll direction: {direction:?}"),
                }
            }

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
            let row_height = self.row_height();
            let scroll_offset = self.scroll_offset.get();
            let number_of_scroll_steps = dy;
            let distance_to_previous_row_start = scroll_offset % row_height;
            let distance_to_next_row_start = row_height - distance_to_previous_row_start;

            // If one row does not fit in the view, scroll by the biggest slice of a row that fits
            // in the view
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

            // Else, scroll to the next row start
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

        #[template_callback]
        fn zoom_begin(&self) {
            self.static_zoom_level.set(self.dynamic_zoom_level.get());
        }

        #[template_callback]
        fn zoom_scale_changed(&self, scale: f64, gesture: gtk::GestureZoom) {
            let Some((_x_center, y_center)) = gesture.bounding_box_center() else {
                return;
            };
            self.update_zoom(y_center, scale);
        }

        #[template_callback]
        fn motion_enter(&self, _x: f64, y: f64, _controller: gtk::EventControllerMotion) {
            self.pointer_y.set(Some(y));
        }

        #[template_callback]
        fn motion(&self, _x: f64, y: f64, _controller: gtk::EventControllerMotion) {
            self.pointer_y.set(Some(y));
        }

        #[template_callback]
        fn motion_leave(&self) {
            self.pointer_y.set(None);
        }
    }
}

glib::wrapper! {
    pub struct MonthViewInner(ObjectSubclass<imp::MonthViewInner>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl MonthViewInner {
    /// Sets the triplet year-month-day.
    pub fn set_year_month_day(&self, year: i32, month: i32, day: i32) {
        self.imp().set_year_month_day(year, month, day);
    }
}
