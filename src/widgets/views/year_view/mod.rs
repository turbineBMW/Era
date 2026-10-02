use std::cell::{Cell, OnceCell, RefCell};

use adw::{prelude::*, subclass::prelude::*};
use glib::clone;
use gtk::Allocation;
use tracing::warn;

use crate::{Application, utils::Date, widgets::window::Styling};

use super::kinetic_scrolling::KineticScrolling;

mod year_view_cell;
mod year_view_floating_controls;
mod year_view_row;

use self::{
    year_view_cell::YearViewCell, year_view_floating_controls::YearViewFloatingControls,
    year_view_row::YearViewRow,
};

const NB_ROWS: usize = 30;
const MINIMUM_NB_ROWS_ABOVE: i32 = 5;
const MINIMUM_NB_ROWS_BELOW: i32 = 5;
const NB_ROWS_ABOVE_AT_RESET: i32 = MINIMUM_NB_ROWS_ABOVE + 1;

/// Duration of the discrete-scroll animation in milliseconds.
const DISCRETE_SCROLL_ANIMATION_MS: u32 = 200;

/// Actively handled input or input consequence.
#[derive(Debug)]
enum Input {
    Drag { start_offset: f64 },
    ContinuousScroll,
    Animation(Animation),
}

/// Active scroll animation.
#[derive(Debug)]
enum Animation {
    /// Kinetic (inertial) deceleration after a swipe or touchpad fling.
    KineticDeceleration {
        kinetic_scrolling: KineticScrolling,
        tick_id: gtk::TickCallbackId,
    },
    DiscreteScroll {
        start_offset: f64,
        start_time: i64,
        target: f64,
        tick_id: gtk::TickCallbackId,
    },
}

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(file = "data/resources/ui/views/year_view/year_view.blp")]
    #[properties(wrapper_type = super::YearView)]
    pub struct YearView {
        #[property(get, set = Self::set_date)]
        date: Cell<Date>,
        #[property(get,  set = Self::set_styling, construct, builder(Styling::default()))]
        styling: Cell<Styling>,

        #[template_child]
        floating_controls: TemplateChild<YearViewFloatingControls>,
        #[template_child]
        scroll_drag: TemplateChild<gtk::GestureDrag>,
        #[template_child]
        scroll_swipe: TemplateChild<gtk::GestureSwipe>,

        /// A collection of rows used to display the years. The first element might not be the first
        /// row displayed. This is used as a circular set for efficient recycling.
        rows: OnceCell<[YearViewRow; NB_ROWS]>,
        /// The index of the first row displayed.
        first_row_index: Cell<usize>,
        /// The height that was given to each row during the last size_allocate. Should never be
        /// zero after constructed is ran.
        row_height: Cell<i32>,

        /// Number of pixels from the top of the first cell to the top of the widget. Value should
        /// be positive, meaning the top rows are scrolled off-screen upward.
        scroll_offset: Cell<f64>,

        /// The currently handled input, if any.
        input: RefCell<Option<Input>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for YearView {
        const NAME: &'static str = "YearView";
        type Type = super::YearView;
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
    impl ObjectImpl for YearView {
        fn constructed(&self) {
            self.parent_constructed();

            let application = Application::default();
            let system = application.system();
            let today = system.date();
            self.date.set(today);

            let current_year = today.to_jiff().year() as i32;
            let first_row_year = current_year - NB_ROWS_ABOVE_AT_RESET;

            let rows = std::array::from_fn(|i| {
                let year = first_row_year + i as i32;
                let row = YearViewRow::new(year);
                row.insert_before(&*self.obj(), Some(&self.floating_controls.get()));
                row
            });

            self.rows.set(rows).unwrap();

            self.scroll_swipe.group_with(&*self.scroll_drag);
        }

        fn dispose(&self) {
            for row in self.rows.get().unwrap().iter() {
                row.unparent();
            }
            self.floating_controls.unparent();
        }
    }

    impl WidgetImpl for YearView {
        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            let rows = self.rows.get().unwrap();
            let first_row_index = self.first_row_index.get();

            let first_row = rows[first_row_index].to_owned();
            let old_row_height = self.row_height.get();
            let (new_row_height, ..) = first_row.measure(gtk::Orientation::Vertical, width);

            // If the row height changes, the scroll offset is meaningless. As a consequence we snap
            // to the row of the current displayed year.
            if old_row_height != new_row_height {
                let nb_rows_above = if old_row_height == 0 {
                    NB_ROWS_ABOVE_AT_RESET
                } else {
                    self.scroll_offset.get() as i32 / old_row_height
                };

                for i in 0..NB_ROWS {
                    let row = rows[(first_row_index + i) % NB_ROWS].to_owned();
                    let allocation = Allocation::new(
                        0,
                        (-nb_rows_above + i as i32) * new_row_height,
                        width,
                        new_row_height,
                    );
                    row.size_allocate(&allocation, baseline);
                }

                self.scroll_offset
                    .set((nb_rows_above * new_row_height) as f64);
            } else {
                let scroll_offset = self.scroll_offset.get() as i32;

                for i in 0..NB_ROWS {
                    let row = rows[(first_row_index + i) % NB_ROWS].to_owned();
                    let allocation = Allocation::new(
                        0,
                        -scroll_offset + i as i32 * new_row_height,
                        width,
                        new_row_height,
                    );
                    row.size_allocate(&allocation, baseline);
                }
            }

            self.row_height.set(new_row_height);

            match self.styling.get() {
                Styling::Narrow => {
                    self.floating_controls.set_child_visible(false);
                }
                Styling::Medium | Styling::Wide => {
                    self.floating_controls.set_child_visible(true);

                    let (_minimum_floating_controls_width, natural_floating_controls_width, ..) =
                        self.floating_controls
                            .measure(gtk::Orientation::Horizontal, -1);
                    let (_minimum_floating_controls_height, natural_floating_controls_height, ..) =
                        self.floating_controls
                            .measure(gtk::Orientation::Vertical, -1);

                    let allocation = gtk::Allocation::new(
                        width - natural_floating_controls_width,
                        height - natural_floating_controls_height,
                        natural_floating_controls_width,
                        natural_floating_controls_height,
                    );

                    self.floating_controls.size_allocate(&allocation, baseline);
                }
            }
        }
    }

    #[gtk::template_callbacks]
    impl YearView {
        /// Sets the date.
        fn set_date(&self, date: Date) {
            if self.date.get() == date {
                return;
            }

            self.cancel_animation_and_clear_input();

            let base_year = date.to_jiff().year() as i32 - NB_ROWS_ABOVE_AT_RESET;

            for (i, row) in self.rows.get().unwrap().iter().enumerate() {
                row.set_year(base_year + i as i32);
            }

            let row_height = self.row_height.get();
            self.first_row_index.set(0);
            self.scroll_offset
                .set((NB_ROWS_ABOVE_AT_RESET * row_height) as f64);

            self.date.set(date);

            self.recycle_if_needed();

            self.obj().notify_date();

            self.obj().queue_allocate();
        }

        /// Sets the styling used for the view.
        fn set_styling(&self, styling: Styling) {
            if self.styling.get() == styling {
                return;
            }

            self.styling.set(styling);

            for row in self.rows.get().unwrap() {
                row.set_styling(styling);
            }

            self.obj().notify_styling();
        }

        #[template_callback]
        fn kinetic_scroll_begin(&self, _scroll_controller: gtk::EventControllerScroll) {
            self.cancel_animation_and_clear_input();
            self.input.replace(Some(Input::ContinuousScroll));
        }

        #[template_callback]
        fn kinetic_scroll(
            &self,
            _dx: f64,
            dy: f64,
            scroll_controller: gtk::EventControllerScroll,
        ) -> bool {
            let Some(Input::ContinuousScroll) = *self.input.borrow() else {
                return false;
            };

            // For kinetic scrolling, we only want to handle smooth events. Don't handle discrete
            // events.
            match scroll_controller
                .current_event()
                .expect("Controller should have a current event")
                .downcast::<gdk::ScrollEvent>()
                .expect("A scroll controller should only have a scroll event")
                .direction()
            {
                gdk::ScrollDirection::Up | gdk::ScrollDirection::Down => {
                    return false;
                }
                gdk::ScrollDirection::Smooth => (),
                gdk::ScrollDirection::Left | gdk::ScrollDirection::Right => {
                    panic!("Vertical scroll controller should not signal horizontal scroll events")
                }
                direction => panic!("Unknown scroll direction: {direction:?}"),
            }

            let scroll_offset = self.scroll_offset.get();
            self.scroll_offset.set(scroll_offset + dy);
            self.recycle_if_needed();
            self.obj().queue_allocate();

            true
        }

        #[template_callback]
        fn kinetic_scroll_end(&self, _scroll_controller: gtk::EventControllerScroll) {
            let Some(Input::ContinuousScroll) = *self.input.borrow() else {
                return;
            };

            self.input.replace(None);
        }

        #[template_callback]
        fn kinetic_scroll_decelerate(&self, _velocity_x: f64, velocity_y: f64) {
            if self.input.borrow().is_some() {
                return;
            }

            let frame_clock = self.obj().frame_clock().unwrap();
            let now = frame_clock.frame_time();

            let initial_position = self.scroll_offset.get();
            let kinetic_scrolling = KineticScrolling::new(now, initial_position, velocity_y);

            let tick_id = self.obj().add_tick_callback(clone!(
                #[weak(rename_to = imp)]
                self,
                #[upgrade_or]
                glib::ControlFlow::Break,
                move |_obj, frame_clock| imp.deceleration_tick(frame_clock.frame_time())
            ));
            self.input
                .replace(Some(Input::Animation(Animation::KineticDeceleration {
                    kinetic_scrolling,
                    tick_id,
                })));
        }

        #[template_callback]
        fn discrete_scroll(
            &self,
            _dx: f64,
            dy: f64,
            scroll_controller: gtk::EventControllerScroll,
        ) -> bool {
            // The discrete controller also fires for smooth scroll events. Those are already
            // handled by the kinetic controller above, so skip them here.
            match scroll_controller
                .current_event()
                .expect("Controller should have a current event")
                .downcast::<gdk::ScrollEvent>()
                .expect("A scroll controller should only have a scroll event")
                .direction()
            {
                gdk::ScrollDirection::Up | gdk::ScrollDirection::Down => (),
                // The discrete controller also fires for smooth scroll events. Those are already
                // handled by the kinetic controller above, so skip them here.
                gdk::ScrollDirection::Smooth => return false,
                gdk::ScrollDirection::Left | gdk::ScrollDirection::Right => {
                    panic!("Vertical scroll controller should not signal horizontal scroll events")
                }
                _ => unreachable!(),
            }

            if dy > 0.0 {
                self.accumulate_discrete_scroll(false);
            } else {
                self.accumulate_discrete_scroll(true);
            }

            true
        }

        #[template_callback]
        fn discrete_scroll_end(&self, _scroll_controller: gtk::EventControllerScroll) {
            let Some(Input::Animation(Animation::DiscreteScroll { .. })) = *self.input.borrow()
            else {
                return;
            };

            self.input.replace(None);
        }

        #[template_callback]
        fn scroll_drag_begin(&self, _start_x: f64, _start_y: f64, _gesture_drag: gtk::GestureDrag) {
            self.cancel_animation_and_clear_input();
            self.input.replace(Some(Input::Drag {
                start_offset: self.scroll_offset.get(),
            }));
        }

        #[template_callback]
        fn scroll_drag_update(
            &self,
            _offset_x: f64,
            offset_y: f64,
            gesture_drag: gtk::GestureDrag,
        ) {
            let Some(Input::Drag { start_offset }) = *self.input.borrow() else {
                return;
            };

            if self.obj().drag_check_threshold(0, 0, 0, offset_y as i32) {
                gesture_drag.set_state(gtk::EventSequenceState::Claimed);

                let target = start_offset - offset_y;
                self.scroll_offset.set(target);
                self.recycle_if_needed();
                self.obj().queue_allocate();
            }
        }

        #[template_callback]
        fn scroll_drag_end(&self, _offset_x: f64, _offset_y: f64, _gesture_drag: gtk::GestureDrag) {
            let Some(Input::Drag { .. }) = *self.input.borrow() else {
                return;
            };

            self.input.replace(None);
        }

        #[template_callback]
        fn swipe(&self, _velocity_x: f64, velocity_y: f64, _gesture_swipe: gtk::GestureSwipe) {
            match &*self.input.borrow() {
                Some(Input::Drag { .. }) | None => {}
                _ => return,
            }

            let frame_clock = self.obj().frame_clock().unwrap();
            let now = frame_clock.frame_time();

            let initial_position = self.scroll_offset.get();
            let kinetic_scrolling = KineticScrolling::new(now, initial_position, -velocity_y);

            let tick_id = self.obj().add_tick_callback(clone!(
                #[weak(rename_to = imp)]
                self,
                #[upgrade_or]
                glib::ControlFlow::Break,
                move |_obj, frame_clock| imp.deceleration_tick(frame_clock.frame_time())
            ));
            self.input
                .replace(Some(Input::Animation(Animation::KineticDeceleration {
                    kinetic_scrolling,
                    tick_id,
                })));
        }

        pub(super) fn accumulate_discrete_scroll(&self, up: bool) {
            // If a discrete animation was already running, we stack on top of where it was headed.
            let baseline =
                if let Some(Input::Animation(Animation::DiscreteScroll { target, .. })) =
                    *self.input.borrow()
                {
                    target
                } else {
                    self.scroll_offset.get()
                };

            self.cancel_animation_and_clear_input();

            let height = self.obj().height() as f64;
            let row_height = self.row_height.get() as f64;

            let target = if row_height > height {
                let number_of_slices_per_row = (row_height / height).ceil();
                let slice_height = row_height / number_of_slices_per_row;
                if up {
                    baseline - slice_height
                } else {
                    baseline + slice_height
                }
            } else {
                // Advance by exactly one row in the scroll direction, aligning to the next row
                // boundary.
                let offset_into_row = baseline.rem_euclid(row_height);
                if up {
                    if offset_into_row < 0.1 {
                        baseline - row_height
                    } else {
                        baseline - offset_into_row
                    }
                } else {
                    let to_next = row_height - offset_into_row;
                    if to_next < 0.1 {
                        baseline + row_height
                    } else {
                        baseline + to_next
                    }
                }
            };

            let frame_clock = self.obj().frame_clock().unwrap();
            let now = frame_clock.frame_time();
            let start_offset = self.scroll_offset.get();

            let tick_id = self.obj().add_tick_callback(clone!(
                #[weak(rename_to = imp)]
                self,
                #[upgrade_or]
                glib::ControlFlow::Break,
                move |_obj, frame_clock| imp
                    .discrete_scroll_animation_tick(frame_clock.frame_time())
            ));
            self.input
                .replace(Some(Input::Animation(Animation::DiscreteScroll {
                    start_offset,
                    start_time: now,
                    target,
                    tick_id,
                })));
        }

        fn deceleration_tick(&self, frame_time: i64) -> glib::ControlFlow {
            let (new_position, _velocity, running) = {
                let Some(Input::Animation(Animation::KineticDeceleration {
                    ref mut kinetic_scrolling,
                    ..
                })) = *self.input.borrow_mut()
                else {
                    warn!(
                        "The kinetic deceleration animation should be cancelled properly by removing the tick callback"
                    );
                    return glib::ControlFlow::Break;
                };
                kinetic_scrolling.tick(frame_time)
            };

            self.scroll_offset.set(new_position);
            self.recycle_if_needed();
            self.obj().queue_allocate();

            if running {
                glib::ControlFlow::Continue
            } else {
                self.input.replace(None);
                glib::ControlFlow::Break
            }
        }

        fn discrete_scroll_animation_tick(&self, frame_time: i64) -> glib::ControlFlow {
            let Some(Input::Animation(Animation::DiscreteScroll {
                start_offset,
                start_time,
                target,
                ..
            })) = *self.input.borrow_mut()
            else {
                warn!(
                    "The discrete scroll animation should be cancelled properly by removing the tick callback"
                );
                return glib::ControlFlow::Break;
            };

            let duration_us = DISCRETE_SCROLL_ANIMATION_MS as i64 * 1000;
            let elapsed = frame_time - start_time;
            let t = (elapsed as f64 / duration_us as f64).clamp(0.0, 1.0);

            // Ease-out cubic: decelerate towards the target.
            let eased = 1.0 - (1.0 - t).powi(3);

            let new_offset = start_offset + (target - start_offset) * eased;
            self.scroll_offset.set(new_offset);
            self.recycle_if_needed();
            self.obj().queue_allocate();

            if t >= 1.0 {
                self.input.replace(None);
                return glib::ControlFlow::Break;
            }

            glib::ControlFlow::Continue
        }

        // Cancels any ongoing scroll animation, and resets the current input to None.
        fn cancel_animation_and_clear_input(&self) {
            if let Some(Input::Animation(animation)) = self.input.borrow_mut().take() {
                match animation {
                    Animation::KineticDeceleration { tick_id, .. } => tick_id.remove(),
                    Animation::DiscreteScroll { tick_id, .. } => tick_id.remove(),
                }
            }
        }

        fn recycle_if_needed(&self) {
            let rows = self.rows.get().unwrap();

            let row_height = self.row_height.get() as f64;
            let height = self.obj().height() as f64;

            let top_threshold = row_height * MINIMUM_NB_ROWS_ABOVE as f64;
            let bottom_threshold = (NB_ROWS as f64 - MINIMUM_NB_ROWS_BELOW as f64) * row_height;

            loop {
                let scroll_offset = self.scroll_offset.get();
                let first_row_index = self.first_row_index.get();

                if scroll_offset < top_threshold {
                    let new_index = (first_row_index + NB_ROWS - 1) % NB_ROWS;
                    let old_first_year = rows[first_row_index].year();
                    let new_first_year = old_first_year - 1;
                    rows[new_index % NB_ROWS].set_year(new_first_year);

                    self.first_row_index.set(new_index);
                    self.scroll_offset.set(scroll_offset + row_height);

                    // Adjust all scroll_offset related variables
                    match &mut *self.input.borrow_mut() {
                        Some(Input::Drag { start_offset }) => *start_offset += row_height,
                        Some(Input::Animation(Animation::KineticDeceleration {
                            kinetic_scrolling,
                            ..
                        })) => {
                            kinetic_scrolling.shift_origin(row_height);
                        }
                        Some(Input::Animation(Animation::DiscreteScroll {
                            start_offset,
                            target,
                            ..
                        })) => {
                            *start_offset += row_height;
                            *target += row_height;
                        }
                        _ => {}
                    }
                } else if scroll_offset + height > bottom_threshold {
                    let new_index = (first_row_index + 1) % NB_ROWS;
                    let old_last_year = rows[(first_row_index + NB_ROWS - 1) % NB_ROWS].year();
                    let new_last_year = old_last_year + 1;
                    rows[first_row_index].set_year(new_last_year);

                    self.first_row_index.set(new_index);
                    self.scroll_offset.set(scroll_offset - row_height);

                    // Adjust all scroll_offset related variables
                    match &mut *self.input.borrow_mut() {
                        Some(Input::Drag { start_offset, .. }) => {
                            *start_offset -= row_height;
                        }
                        Some(Input::Animation(Animation::KineticDeceleration {
                            kinetic_scrolling,
                            ..
                        })) => {
                            kinetic_scrolling.shift_origin(-row_height);
                        }
                        Some(Input::Animation(Animation::DiscreteScroll {
                            start_offset,
                            target,
                            ..
                        })) => {
                            *start_offset -= row_height;
                            *target -= row_height;
                        }
                        _ => {}
                    }
                } else {
                    break;
                }
            }

            let first_row_index = self.first_row_index.get();

            let rows_above_view = (self.scroll_offset.get() / row_height).ceil() as usize;
            let first_visible_row_index = (first_row_index + rows_above_view) % NB_ROWS;
            let new_date = Date::from(
                jiff::civil::Date::new(rows[first_visible_row_index].year() as i16, 1, 1).unwrap(),
            );

            if self.date.get() != new_date {
                self.date.set(new_date);
                self.obj().notify_date();
            }
        }
    }
}

glib::wrapper! {
    pub struct YearView(ObjectSubclass<imp::YearView>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl YearView {
    pub fn scroll_up(&self) {
        self.imp().accumulate_discrete_scroll(true);
    }

    pub fn scroll_down(&self) {
        self.imp().accumulate_discrete_scroll(false);
    }
}
