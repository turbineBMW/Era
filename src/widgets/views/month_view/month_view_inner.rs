use std::{
    cell::{Cell, OnceCell, RefCell},
    cmp::max,
    mem,
    sync::Mutex,
};

use adw::{prelude::*, subclass::prelude::*};
use glib::{DateTime, clone};
use gtk::Allocation;
use jiff::ToSpan;

use crate::{Application, system_settings::DayOfWeek, utils};

use super::{MonthViewStyling, month_view_row::MonthViewRow};

// TODO: Reduce this (requires batch recycling)
const NB_ROWS: i32 = 50;
const MINIMUM_NB_ROWS_ABOVE: i32 = 10;
const NB_ROWS_ABOVE_AT_STARTUP: i32 = MINIMUM_NB_ROWS_ABOVE;
// const ROWS_ABOVE_AFTER_RECYCLING: i32 = 10;
const MINIMUM_NB_ROWS_BELOW: i32 = 10;
// const ROWS_BELOW_AFTER_RECYCLING: i32 = 10;
/// Minimum height of a row in pixels. The row itself can return a minimum height higher than this,
/// and size_allocate will respect it, but size_allocate will never allocate them less than this.
const MINIMUM_ROW_HEIGHT: i32 = 10;
/// Maximum height of a row in pixels.
// TODO: Should we allow more only in the case the row reports a minimum higher than this?
const MAXIMUM_ROW_HEIGHT: i32 = 300;

const VELOCITY_THRESHOLD_TO_RETURN: f64 = 300.;
const VELOCITY_THRESHOLD_TO_SNAP: f64 = 400.;
const VELOCITY_THRESHOLD_TO_SKIP: f64 = 2.;
const FIRST_STAGE_DIVISOR: f64 = 1.5;
const FIRST_TO_SECOND_STAGE_THRESHOLD: f64 = 300.;
const SECOND_STAGE_DIVISOR: f64 = 2.5;
const DISCRETE_SCROLL_DISTANCE_THRESHOLD_TO_SNAP: f64 = 100.;
const DISCRETE_SCROLL_DISTANCE_THRESHOLD_TO_ROW: f64 = 50.;

#[derive(Debug, Clone, Copy, Default)]
enum PositionDescription {
    /// The default value when the month view is created
    #[default]
    Init,
    /// An absolute scroll offset
    ScrollOffset(i32),
    /// A point is fixed
    OffsetOfFixedPoint(i32),
}

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
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

        /// Last offset of the top of the first row. This is also the number of pixels above the
        /// view that are not visible.
        last_scroll_offset: Cell<i32>,
        /// Describes the next position of the view. This is first set to init. Then it can either
        /// be an absolute scroll_offset (when scrolling happens), or a fixed point (when zoom
        /// occurs).
        /// This is enough information for size_allocate to do its job, but it is not enough to
        /// know what will be the first visible row.
        /// This value is only set in stone once size_allocate is called. Before that point, it can
        /// be set many times.
        next_position: Cell<PositionDescription>,

        /// The height that was given to each row during the last size_allocate.
        last_row_height: Cell<i32>,
        /// The desired height to give in the next size_allocate. It shouldn't be set to a value
        /// bigger than MAXIMUM_ROW_HEIGHT. size_allocate might give the rows more height than this,
        /// to respect the rows measurements and MINIMUM_ROW_HEIGHT.
        /// This value is only set in stone once size_allocate is called. Before that point, it can
        /// be set many times.
        desired_next_row_height: Cell<i32>,

        /// Currently running scroll animation, if any.
        scroll_animation: RefCell<Option<adw::TimedAnimation>>,
        /// Number of pixels the view should have moved by already, but because it is less than 1,
        /// it is still pending.
        pixels_waiting: Cell<f64>,
        /// Number of pixels to move by still to go.
        pixels_left: Cell<f64>,

        /// Y position of the pointer to use for zooming with CTRL+scroll.
        pointer_y: Cell<Option<f64>>,
        /// Zoom level updated from the dynamic one when a zoom operation begins. It serves as a
        /// base reference when smooth zooming is occurring.
        last_scale_delta: Cell<f64>,
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

            let application = Application::default();
            let now = application.current_datetime();

            self.year.set(now.year());
            self.month.set(now.month());
            self.day.set(now.day_of_month());

            let first_day_of_week = application.system_settings().first_day_of_week();

            let a_day_in_first_row = DateTime::new(
                &now.timezone(),
                now.year(),
                now.month(),
                now.day_of_month(),
                0,
                0,
                0.,
            )
            .expect("DateTime should be valid")
            .add_weeks(-NB_ROWS_ABOVE_AT_STARTUP)
            .expect("DateTime should be valid");

            let first_day_of_timeframe =
                utils::get_last_occurrence_of_weekday(a_day_in_first_row, first_day_of_week);

            // Setup rows
            self.rows
                .set({
                    Mutex::new(
                        (0..NB_ROWS)
                            .map(|i| {
                                let date = first_day_of_timeframe.add_weeks(i).unwrap();
                                let row = MonthViewRow::new(
                                    date.year(),
                                    date.month(),
                                    date.day_of_month(),
                                );

                                row.insert_before(&*self.obj(), None::<&gtk::Widget>);
                                self.obj()
                                    .bind_property("styling", &row, "styling")
                                    .sync_create()
                                    .build();

                                if self.last_row_height.get() == 0 {
                                    let (_minimum_row_height, natural_row_height, ..) =
                                        row.measure(gtk::Orientation::Vertical, 50);
                                    self.last_row_height.set(natural_row_height);
                                    self.desired_next_row_height.set(natural_row_height);
                                }

                                row
                            })
                            .collect(),
                    )
                })
                .unwrap();

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
        // TODO: Make this clearer
        fn focus(&self, direction: gtk::DirectionType) -> bool {
            let obj = self.obj();
            let root = obj.root().unwrap();
            let focused = root.focus();

            // If focus is inside us, check if the focused row is still visible.
            // If not, redirect focus to the first/last visible row.
            if let Some(ref focused_widget) = focused
                && focused_widget.is_ancestor(&*obj)
            {
                let still_visible = focused_widget
                    .ancestor(MonthViewRow::static_type())
                    .and_then(|row| {
                        let row_widget = row.downcast_ref::<MonthViewRow>().unwrap();
                        let bounds = row_widget.compute_bounds(&*obj)?;
                        let row_top = bounds.y() as i32;
                        let row_bottom = row_top + bounds.height() as i32;
                        let view_height = obj.height();
                        // Consider visible if any part is in view
                        Some(row_bottom > 0 && row_top < view_height)
                    })
                    .unwrap_or(false);

                if still_visible {
                    match direction {
                        gtk::DirectionType::Up | gtk::DirectionType::Down => {
                            // Move to adjacent row, same column
                            let current_row = focused_widget
                                .ancestor(MonthViewRow::static_type())
                                .unwrap()
                                .downcast::<MonthViewRow>()
                                .unwrap();
                            let col = current_row.focused_column().unwrap_or(0);
                            let rows = self.rows().lock().unwrap();
                            if let Some(idx) = rows.iter().position(|r| *r == current_row) {
                                let target_idx = if direction == gtk::DirectionType::Up {
                                    if idx == 0 {
                                        return false;
                                    }
                                    idx - 1
                                } else {
                                    if idx + 1 >= rows.len() {
                                        return false;
                                    }
                                    idx + 1
                                };
                                let target_row = rows[target_idx].clone();
                                drop(rows);
                                self.scroll_row_into_view(&target_row);
                                return target_row.focus_column(col);
                            }
                            return false;
                        }
                        gtk::DirectionType::Left => {
                            // Get current row before any focus changes
                            let current_row = focused_widget
                                .ancestor(MonthViewRow::static_type())
                                .unwrap()
                                .downcast::<MonthViewRow>()
                                .unwrap();
                            // Let the row handle it internally
                            if current_row.imp().focus(direction) {
                                return true;
                            }
                            // Row couldn't handle it (at col 0): wrap to previous row, last
                            // column
                            let rows = self.rows().lock().unwrap();
                            if let Some(idx) = rows.iter().position(|r| *r == current_row) {
                                if idx == 0 {
                                    return false;
                                }
                                let target_row = rows[idx - 1].clone();
                                drop(rows);
                                self.scroll_row_into_view(&target_row);
                                return target_row.focus_column(6);
                            }
                            return false;
                        }
                        gtk::DirectionType::Right => {
                            // Get current row before any focus changes
                            let current_row = focused_widget
                                .ancestor(MonthViewRow::static_type())
                                .unwrap()
                                .downcast::<MonthViewRow>()
                                .unwrap();
                            // Let the row handle it internally
                            if current_row.imp().focus(direction) {
                                return true;
                            }
                            // Row couldn't handle it (at col 6): wrap to next row, first column
                            let rows = self.rows().lock().unwrap();
                            if let Some(idx) = rows.iter().position(|r| *r == current_row) {
                                if idx + 1 >= rows.len() {
                                    return false;
                                }
                                let target_row = rows[idx + 1].clone();
                                drop(rows);
                                self.scroll_row_into_view(&target_row);
                                return target_row.focus_column(0);
                            }
                            return false;
                        }
                        _ => {
                            let result = self.parent_focus(direction);
                            if result
                                && let Some(new_focused) = root.focus()
                                && let Some(row) = new_focused.ancestor(MonthViewRow::static_type())
                            {
                                let row = row.downcast::<MonthViewRow>().unwrap();
                                self.scroll_row_into_view(&row);
                            }

                            return result;
                        }
                    }

                    // Focused row is no longer visible — fall through to re-enter logic
                }
            }

            // Focus is entering the view from outside, or the focused row scrolled away
            match direction {
                gtk::DirectionType::TabForward => {
                    let row = self.first_visible_row();
                    self.scroll_row_into_view(&row);
                    row.grab_focus()
                }
                gtk::DirectionType::TabBackward => {
                    let row = self.last_visible_row();
                    self.scroll_row_into_view(&row);
                    row.grab_focus()
                }
                _ => self.parent_focus(direction),
            }
        }

        fn size_allocate(&self, width: i32, _height: i32, baseline: i32) {
            let last_scroll_offset = self.last_scroll_offset.get();
            let last_row_height = self.last_row_height.get();
            let desired_next_row_height = self.desired_next_row_height.get();

            // TODO: Should we use the natural one?
            let (minimum_row_height, _natural_row_height, ..) = self
                .rows()
                .lock()
                .unwrap()
                .first()
                .unwrap()
                .measure(gtk::Orientation::Vertical, width);

            let floored_minimum_row_height = max(minimum_row_height, MINIMUM_ROW_HEIGHT);

            let row_height =
                desired_next_row_height.clamp(floored_minimum_row_height, MAXIMUM_ROW_HEIGHT);

            let scroll_offset = match self.next_position.get() {
                PositionDescription::Init => NB_ROWS_ABOVE_AT_STARTUP * row_height,
                PositionDescription::ScrollOffset(scroll_offset) => scroll_offset,
                PositionDescription::OffsetOfFixedPoint(offset_of_fixed_point) => {
                    let new_offset_of_fixed_point =
                        (offset_of_fixed_point as f64 * row_height as f64 / last_row_height as f64)
                            as i32;
                    last_scroll_offset + new_offset_of_fixed_point - offset_of_fixed_point
                }
            };

            self.last_scroll_offset.set(scroll_offset);
            self.next_position
                .set(PositionDescription::ScrollOffset(scroll_offset));
            self.last_row_height.set(row_height);
            self.desired_next_row_height.set(row_height);

            for (i, row) in self.rows().lock().unwrap().iter().enumerate() {
                let allocation =
                    Allocation::new(0, -scroll_offset + i as i32 * row_height, width, row_height);
                row.size_allocate(&allocation, baseline);
            }

            self.update_date();
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

        /// Returns the first row that is at least partially visible.
        fn first_visible_row(&self) -> MonthViewRow {
            let scroll_offset = self.last_scroll_offset.get();
            let row_height = self.last_row_height.get();
            let rows = self.rows().lock().unwrap();
            let first_idx = (scroll_offset / row_height) as usize;
            rows[first_idx].clone()
        }

        /// Returns the last row that is at least partially visible.
        fn last_visible_row(&self) -> MonthViewRow {
            let scroll_offset = self.last_scroll_offset.get();
            let row_height = self.last_row_height.get();
            let view_height = self.obj().height();
            let rows = self.rows().lock().unwrap();
            let last_idx = ((scroll_offset + view_height - 1) / row_height) as usize;
            rows[last_idx].clone()
        }

        /// Adjusts scroll offset so the given row is fully visible, with animation.
        // TODO: The row might be higher than the view. In that case, we should have a function to
        // snap to the top or to the bottom.
        fn scroll_row_into_view(&self, row: &MonthViewRow) {
            let last_row_height = self.last_row_height.get();
            let last_scroll_offset = self.last_scroll_offset.get();
            let view_height = self.obj().height();
            let rows = self.rows().lock().unwrap();

            let idx = rows
                .iter()
                .position(|r| r == row)
                .expect("Row should be found");
            let row_top = idx as i32 * last_row_height;
            let row_bottom = row_top + last_row_height;
            if row_top < last_scroll_offset {
                // Row is above view, scroll up
                let dy = (row_top - last_scroll_offset) as f64;
                drop(rows);
                self.start_scroll_animation(dy);
            } else if row_bottom > last_scroll_offset + view_height {
                // Row is below view, scroll down so row bottom aligns with view bottom
                let dy = (row_bottom - (last_scroll_offset + view_height)) as f64;
                drop(rows);
                self.start_scroll_animation(dy);
            }
        }

        /// Updates the view to the currently stored date.
        fn update_view_to_stored_date(&self) {
            let application = Application::default();
            let timezone = application.current_datetime().timezone();

            let a_day_in_first_row = DateTime::new(
                &timezone,
                self.year.get(),
                self.month.get(),
                self.day.get(),
                0,
                0,
                0.,
            )
            .expect("DateTime should be valid")
            .add_weeks(-MINIMUM_NB_ROWS_ABOVE)
            .expect("DateTime should be valid");

            for (i, row) in self.rows().lock().unwrap().iter().enumerate() {
                let date = a_day_in_first_row
                    .add_weeks(i as i32)
                    .expect("DateTime should be valid");
                row.set_year_month_day(date.year(), date.month(), date.day_of_month());
            }

            let row_height = self.last_row_height.get();
            let current_offset = self.last_scroll_offset.get();
            self.scroll_offset_add(MINIMUM_NB_ROWS_ABOVE * row_height - current_offset);
        }

        /// Sets the next position of the view.
        ///
        /// If necessary, rows will be recycled and the offset will get adjusted. Year/month/day
        /// properties will be updated.
        fn set_next_position(&self, next_position: PositionDescription) {
            let height = self.obj().height();
            let mut rows = self.rows().lock().unwrap();

            match next_position {
                PositionDescription::Init => panic!("Next position should not be set to init"),
                PositionDescription::ScrollOffset(scroll_offset) => {
                    let row_height = self.last_row_height.get();
                    // The limit of the top offset before recycling happens
                    let top_threshold = row_height * MINIMUM_NB_ROWS_ABOVE;
                    // The limit of the bottom offset before recycling happens
                    let bottom_threshold = (NB_ROWS - MINIMUM_NB_ROWS_BELOW) * row_height;

                    // Recycle a row if necessary and set the new scroll offset
                    // TODO: Recycle multiple rows if needed
                    if scroll_offset < top_threshold {
                        // Take the last row and move it to the top
                        self.next_position.set(PositionDescription::ScrollOffset(
                            scroll_offset + row_height,
                        ));

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
                        // Take the first row and move it to the bottom
                        self.next_position.set(PositionDescription::ScrollOffset(
                            scroll_offset - row_height,
                        ));

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
                        self.next_position
                            .set(PositionDescription::ScrollOffset(scroll_offset));
                    }
                }
                PositionDescription::OffsetOfFixedPoint(offset_of_fixed_point) => {
                    // TODO: Recycle if necessary
                    self.next_position
                        .set(PositionDescription::OffsetOfFixedPoint(
                            offset_of_fixed_point,
                        ))
                }
            }
            mem::drop(rows);

            self.obj().queue_allocate();
        }

        /// Updates the view's date to the one of the most top visible row.
        fn update_date(&self) {
            let first_visible_row = self.first_visible_row();

            let year = first_visible_row.year();
            let month = first_visible_row.month();
            let day = first_visible_row.day();

            let date = jiff::civil::Date::new(year as i16, month as i8, day as i8).unwrap();

            let base = match Application::default().system_settings().first_day_of_week() {
                DayOfWeek::Monday => 1,
                DayOfWeek::Tuesday => 2,
                DayOfWeek::Wednesday => 3,
                DayOfWeek::Thursday => 4,
                DayOfWeek::Friday => 5,
                DayOfWeek::Saturday => 6,
                DayOfWeek::Sunday => 7,
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
            match self.next_position.get() {
                PositionDescription::Init => {
                    // Let size_allocate handle the initial position
                }
                PositionDescription::ScrollOffset(scroll_offset) => {
                    self.set_next_position(PositionDescription::ScrollOffset(scroll_offset + dy))
                }
                PositionDescription::OffsetOfFixedPoint(_offset_of_fixed_point) => {
                    // TODO: Define priorities
                }
            }
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

        /// Zoom cancels the ongoing scroll animation, and overrides next_position to have a fixed
        /// point.
        fn update_zoom(&self, y_center: f64, scale: f64) {
            self.cancel_scroll_animation();

            // If a zoom already happened since last size_allocate, add this zoom on top.
            let desired_next_row_height = self.desired_next_row_height.get();
            let new_desired_next_row_height = ((desired_next_row_height as f64 * scale) as i32)
                .clamp(MINIMUM_ROW_HEIGHT, MAXIMUM_ROW_HEIGHT);
            self.desired_next_row_height
                .set(new_desired_next_row_height);

            // Set the fixed point of this zoom as the one to use in size_allocate. If a zoom
            // already happened since last size_allocate, override it.
            let current_offset_of_gesture_center = self.last_scroll_offset.get() + y_center as i32;
            self.set_next_position(PositionDescription::OffsetOfFixedPoint(
                current_offset_of_gesture_center,
            ));

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

            let row_height = self.last_row_height.get();
            let scroll_offset = self.last_scroll_offset.get();
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
            let row_height = self.last_row_height.get();
            let scroll_offset = self.last_scroll_offset.get();
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
            // Resets the last scale delta.
            self.last_scale_delta.set(1.);
        }

        #[template_callback]
        fn zoom_scale_changed(&self, scale: f64, gesture: gtk::GestureZoom) {
            let Some((_x_center, y_center)) = gesture.bounding_box_center() else {
                return;
            };
            let last_scale_delta = self.last_scale_delta.get();
            self.update_zoom(y_center, scale / last_scale_delta);
            self.last_scale_delta.set(scale);
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
