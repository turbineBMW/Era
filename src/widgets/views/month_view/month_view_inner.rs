use std::{
    cell::{Cell, OnceCell, RefCell},
    collections::HashSet,
};

use adw::{prelude::*, subclass::prelude::*};
use clepsydre::{Calendar, Event, Subscription, prelude::*};
use glib::{GString, clone};
use jiff::ToSpan;
use tracing::{error, warn};

use crate::{
    application::Application,
    utils::{Date, EventPropertiesPreset},
    widgets::views::kinetic_scrolling::KineticScrolling,
    widgets::window::Styling,
};

use super::{
    layout_utils::{
        EventLayout, RowOverflow, build_event_layouts, compute_event_segments, stack_event_segments,
    },
    month_view_cell::MonthViewCell,
    month_view_event::MonthViewEvent,
    month_view_floating_controls::MonthViewFloatingControls,
    month_view_overflow::MonthViewOverflow,
};

pub const NB_ROWS: usize = 100;
pub const NB_CELLS: usize = 7 * NB_ROWS;
const MINIMUM_NB_ROWS_BELOW: i32 = 10;
const MINIMUM_NB_ROWS_ABOVE: i32 = 10;
const NB_ROWS_ABOVE_AT_RESET: i32 = MINIMUM_NB_ROWS_ABOVE + 1;

/// Minimum height of a cell in pixel. A cell can report a higher minimum and size_allocate will
/// respect it, but will never allocate them less than this.
/// This is a hard minimum.
const MINIMUM_CELL_HEIGHT: i32 = 80;
/// Maximum height of a cell in pixel. Ignored if the cell's own minimum height exceeds this, in
/// which case size_allocate allocates the cell's minimum height instead.
const MAXIMUM_CELL_HEIGHT: i32 = 400;

const _: () = assert!(
    MAXIMUM_CELL_HEIGHT >= MINIMUM_CELL_HEIGHT,
    "MAXIMUM_CELL_HEIGHT must be greater than or equal to MINIMUM_CELL_HEIGHT"
);

const EVENT_GAP: i32 = 2;
const SEPARATOR_HEIGHT: i32 = 1;
const SEPARATOR_WIDTH: i32 = 1;

/// Duration of the discrete-scroll animation in milliseconds.
const DISCRETE_SCROLL_ANIMATION_MS: u32 = 200;

/// Duration of the discrete-zoom animation in milliseconds.
const DISCRETE_ZOOM_ANIMATION_MS: u32 = 200;

const INITIAL_EVENT_WIDGET_POOL_SIZE: usize = 100;

const UNIX_EPOCH_DATE: jiff::civil::Date = jiff::civil::date(1970, 1, 1);

/// Actively handled input or input consequence.
#[derive(Debug)]
enum Input {
    Drag { start_offset: f64 },
    ContinuousScroll,
    ContinuousZoom { last_scale_delta: f64 },
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
    DiscreteZoom {
        start_height: i32,
        target_height: i32,
        start_time: i64,
        y_center: f64,
        tick_id: gtk::TickCallbackId,
    },
}

#[derive(Debug, Clone, Copy)]
struct CreateDrag {
    anchor: Date,
    hover: Date,
}

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(file = "data/resources/ui/views/month_view/month_view_inner.blp")]
    #[properties(wrapper_type = super::MonthViewInner)]
    pub struct MonthViewInner {
        /// The date of the last cell of the first row whose top separator is visible.
        #[property(get, set = Self::set_date)]
        date: Cell<Date>,
        #[property(get, set = Self::set_styling, construct, builder(Styling::default()))]
        styling: Cell<Styling>,

        #[template_child]
        floating_controls: TemplateChild<MonthViewFloatingControls>,
        #[template_child]
        scroll_drag: TemplateChild<gtk::GestureDrag>,
        #[template_child]
        scroll_swipe: TemplateChild<gtk::GestureSwipe>,

        // A collection of cells used to display the month. The first element might not be the first
        // cell displayed. This is used as a circular set for efficient recycling.
        cells: OnceCell<[MonthViewCell; NB_CELLS]>,
        column_separators: OnceCell<[gtk::Separator; 6]>,
        row_separators: OnceCell<[gtk::Separator; NB_ROWS]>,

        // The index of the first cell displayed.
        first_cell_index: Cell<usize>,

        /// Number of pixels from the top of the first cell to the top of the widget. Value should
        /// be positive, meaning the top rows are scrolled off-screen upward.
        scroll_offset: Cell<f64>,

        /// The height that was given to each row during the last size_allocate. Should never be
        /// zero after constructed is ran.
        /// The row height doesn't include the separator height.
        cell_height: Cell<i32>,

        input: RefCell<Option<Input>>,
        create_drag: Cell<Option<CreateDrag>>,

        /// Y position of the pointer to use for zooming with CTRL+scroll.
        pointer_y: Cell<Option<f64>>,

        subscription: OnceCell<Subscription>,

        event_widgets: RefCell<Vec<MonthViewEvent>>,
        overflow_widgets: OnceCell<[MonthViewOverflow; NB_CELLS]>,

        event_layouts: RefCell<Vec<Vec<EventLayout>>>,

        recompute_pending: Cell<bool>,

        connected_event_uris: RefCell<HashSet<GString>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MonthViewInner {
        const NAME: &'static str = "MonthViewInner";
        type Type = super::MonthViewInner;
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
    impl ObjectImpl for MonthViewInner {
        fn constructed(&self) {
            self.parent_constructed();

            let application = Application::default();
            let manager = application.manager();
            let system = application.system();
            let today = system.date();
            let timezone = system.datetime().timezone();
            let calendars_model = manager.calendars_model().unwrap();

            self.date.set(today);

            let first_cell_date = {
                let first_day_of_week = system.first_week_day();

                today
                    .previous_occurrence_of_weekday(first_day_of_week)
                    .to_jiff()
                    - (NB_ROWS_ABOVE_AT_RESET * 7).days()
            };

            let first_cell_index = 0;
            let last_cell_index = (first_cell_index + NB_CELLS - 1) % NB_CELLS;
            self.first_cell_index.set(first_cell_index);

            let cells = std::array::from_fn(|i| {
                let date = (first_cell_date + (i as i32).days()).into();
                let cell = MonthViewCell::new(date);
                cell.insert_before(&*self.obj(), Some(&self.floating_controls.get()));
                cell
            });

            let column_separators = std::array::from_fn(|_| {
                let separator = gtk::Separator::new(gtk::Orientation::Vertical);
                separator.insert_before(&*self.obj(), Some(&self.floating_controls.get()));
                separator
            });

            let row_separators = std::array::from_fn(|_| {
                let separator = gtk::Separator::new(gtk::Orientation::Horizontal);
                separator.insert_before(&*self.obj(), Some(&self.floating_controls.get()));
                separator
            });

            let start = cells[first_cell_index].date().to_glib_date_time(&timezone);
            let end = Date::from(cells[last_cell_index].date().to_jiff().tomorrow().unwrap())
                .to_glib_date_time(&timezone);
            let subscription = manager.new_subscription(&start, &end).unwrap();

            let event_widgets = (0..INITIAL_EVENT_WIDGET_POOL_SIZE)
                .map(|_| {
                    let event_widget = MonthViewEvent::new(None);
                    event_widget.set_child_visible(false);
                    event_widget.insert_before(&*self.obj(), Some(&self.floating_controls.get()));
                    event_widget
                })
                .collect::<Vec<_>>();

            let overflow_widgets = std::array::from_fn(|i| {
                let date = (first_cell_date + (i as i32).days()).into();
                let overflow_widget = MonthViewOverflow::new(date);
                overflow_widget.set_child_visible(false);
                overflow_widget.insert_before(&*self.obj(), Some(&self.floating_controls.get()));
                overflow_widget
            });

            let (_minimum_event_height, natural_event_height, ..) =
                event_widgets[0].measure(gtk::Orientation::Vertical, -1);
            let header_height = cells[0].header_height(-1);

            // TODO: CSS isn't applied yet. We should do this initialisation in the first
            // size_allocate? We could use widget margins?
            let initial_cell_height = header_height + 4 * natural_event_height + 3 * EVENT_GAP;
            self.cell_height.set(initial_cell_height);

            self.scroll_offset
                .set((NB_ROWS_ABOVE_AT_RESET * (initial_cell_height + SEPARATOR_HEIGHT)) as f64);

            self.cells.set(cells).unwrap();
            self.column_separators.set(column_separators).unwrap();
            self.row_separators.set(row_separators).unwrap();

            self.subscription.set(subscription).unwrap();

            self.event_widgets.replace(event_widgets);
            self.overflow_widgets.set(overflow_widgets).unwrap();

            self.scroll_swipe.group_with(&*self.scroll_drag);

            self.subscription
                .get()
                .unwrap()
                .connect_items_changed(clone!(
                    #[weak(rename_to = imp)]
                    self,
                    move |_model, _position, _removed, _added| {
                        // This does not invalidate current layouts, so we can afford to wait a bit
                        // to recompute them to debounce rapidly fired signals.
                        if imp.recompute_pending.get() {
                            return;
                        }

                        imp.recompute_pending.set(true);

                        glib::idle_add_local_once(clone!(
                            #[weak]
                            imp,
                            move || {
                                imp.recompute_pending.set(false);
                                imp.recompute_event_layouts();
                            }
                        ));
                    }
                ));

            system.connect_first_week_day_notify(clone!(
                #[weak(rename_to = imp)]
                self,
                move |system| {
                    let cells = imp.cells.get().unwrap();
                    let overflow_widgets = imp.overflow_widgets.get().unwrap();

                    let first_week_day = system.first_week_day();

                    let first_cell_index = imp.first_cell_index.get();
                    let old_first_date = cells[first_cell_index].date();

                    let new_first_date =
                        old_first_date.previous_occurrence_of_weekday(first_week_day);

                    let difference =
                        (old_first_date.to_jiff() - new_first_date.to_jiff()).get_days() as usize;
                    let new_index = first_cell_index - difference;
                    imp.first_cell_index.set(new_index);

                    for i in 0..difference {
                        let date = Date::from(new_first_date.to_jiff() + (i as i32).days());
                        cells[(new_index + i) % NB_CELLS].set_date(date);
                        overflow_widgets[(new_index + i) % NB_CELLS].set_date(date);
                    }

                    imp.recycle_if_needed();
                    imp.recompute_event_layouts();
                }
            ));

            system.connect_datetime_notify(clone!(
                #[weak(rename_to = imp)]
                self,
                move |_system| {
                    imp.update_subscription_timeframe();
                }
            ));

            for i in 0..calendars_model.n_items() {
                let item = calendars_model
                    .item(i)
                    .unwrap()
                    .downcast::<Calendar>()
                    .unwrap();
                item.connect_visible_notify(clone!(
                    #[weak(rename_to = imp)]
                    self,
                    move |_calendar| {
                        imp.recompute_event_layouts();
                    }
                ));
            }

            calendars_model.connect_items_changed(clone!(
                #[weak(rename_to = imp)]
                self,
                move |model, position, _removed, added| {
                    for i in 0..added {
                        let item = model
                            .item(position + i)
                            .unwrap()
                            .downcast::<Calendar>()
                            .unwrap();
                        item.connect_visible_notify(clone!(
                            #[weak]
                            imp,
                            move |_calendar| {
                                imp.recompute_event_layouts();
                            }
                        ));
                    }
                }
            ));

            self.recompute_event_layouts();
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
            for event_widget in self.event_widgets.borrow().iter() {
                event_widget.unparent();
            }
            for overflow_widget in self.overflow_widgets.get().unwrap() {
                overflow_widget.unparent();
            }
            self.floating_controls.unparent();
        }
    }

    impl WidgetImpl for MonthViewInner {
        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            let cells = self.cells.get().unwrap();
            let column_separators = self.column_separators.get().unwrap();
            let row_separators = self.row_separators.get().unwrap();
            let overflow_widgets = self.overflow_widgets.get().unwrap();

            // Width is distributed evenly. Any remainder pixels are given to the leftmost columns,
            // so the first (width % 7) columns are one pixel wider than the rest.
            let column_widths: [i32; 7] = {
                let width = width - 6 * SEPARATOR_WIDTH;
                let base = width / 7;
                let remainder = width % 7;

                std::array::from_fn(|i| base + i32::from((i as i32) < remainder))
            };

            let column_xs: [i32; 7] = {
                std::array::from_fn(|i| {
                    column_widths[..i].iter().sum::<i32>() + i as i32 * SEPARATOR_WIDTH
                })
            };

            let column_separator_xs: [i32; 6] = {
                std::array::from_fn(|i| {
                    column_widths[..i + 1].iter().sum::<i32>() + i as i32 * SEPARATOR_WIDTH
                })
            };

            let cell_height = {
                let old_cell_height = self.cell_height.get();
                // Measures against the narrowest column (the last one) to get a conservative
                // minimum height that holds for all cells regardless of their width.
                let (minimum_cell_height, ..) =
                    cells[0].measure(gtk::Orientation::Horizontal, column_widths[6]);

                let floored_minimum_cell_height = minimum_cell_height.max(MINIMUM_CELL_HEIGHT);

                if floored_minimum_cell_height > MAXIMUM_CELL_HEIGHT {
                    floored_minimum_cell_height
                } else {
                    old_cell_height
                }
            };

            self.cell_height.set(cell_height);

            let scroll_offset = self.scroll_offset.get() as i32;
            let first_cell_index = self.first_cell_index.get();

            let (minimum_event_height, natural_event_height, ..) = {
                let event_widgets = self.event_widgets.borrow();
                event_widgets[0].measure(gtk::Orientation::Vertical, column_widths[6])
            };
            let header_height = cells[0].header_height(column_widths[6]);

            let event_layouts = self.event_layouts.borrow();

            for row_index in 0..NB_ROWS {
                let row_height = cell_height + SEPARATOR_HEIGHT;
                let cell_y = -scroll_offset + row_index as i32 * row_height;
                let separator_y = cell_y + cell_height;

                let row_visible = (cell_y + row_height) > 0;

                for column_index in 0..7 {
                    let cell = &cells[(row_index * 7 + column_index + first_cell_index) % NB_CELLS];

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
                        gtk::Allocation::new(0, separator_y, width, SEPARATOR_HEIGHT);
                    row_separator.size_allocate(&separator_allocation, baseline);
                } else {
                    row_separator.set_child_visible(false);
                }

                let row_layouts = &event_layouts[row_index];

                if !row_visible {
                    for layout in row_layouts.iter() {
                        layout.widget.set_child_visible(false);
                    }
                    for column_index in 0..7 {
                        let overflow_widget = &overflow_widgets
                            [(row_index * 7 + column_index + first_cell_index) % NB_CELLS];
                        overflow_widget.set_child_visible(false);
                    }
                    continue;
                }

                let use_dense_allocation =
                    cell_height - header_height < 2 * natural_event_height + EVENT_GAP;

                let event_height = if use_dense_allocation {
                    minimum_event_height
                } else {
                    natural_event_height
                };

                let max_events = ((cell_height - header_height + EVENT_GAP)
                    / (event_height + EVENT_GAP)) as usize;

                assert!(max_events >= 2);

                let events_y = cell_y + header_height;

                let overflow = RowOverflow::compute(row_layouts, max_events);

                for placed in overflow.placements {
                    let layout = &placed.layout;

                    if placed.hidden {
                        layout.widget.set_child_visible(false);
                        continue;
                    }

                    layout.widget.set_child_visible(true);

                    let x = column_xs[layout.column_start];
                    let width = column_widths[layout.column_start..=layout.column_end]
                        .iter()
                        .sum::<i32>()
                        + SEPARATOR_WIDTH * (layout.column_end - layout.column_start) as i32;
                    let y = events_y
                        + layout.stack_row as i32 * event_height
                        + (layout.stack_row as i32 - 1) * EVENT_GAP;
                    let height = event_height;
                    let allocation = gtk::Allocation::new(x, y, width, height);

                    layout.widget.size_allocate(&allocation, baseline);
                }

                for column_index in 0..7 {
                    let overflow_widget = &overflow_widgets
                        [(row_index * 7 + column_index + first_cell_index) % NB_CELLS];

                    let hidden_count = overflow.column_hidden_counts[column_index];
                    if hidden_count == 0 {
                        overflow_widget.set_child_visible(false);
                        continue;
                    }

                    overflow_widget.set_child_visible(true);
                    overflow_widget.set_text(format!("+{hidden_count}"));

                    let x = column_xs[column_index];
                    let width = column_widths[column_index];
                    let y =
                        events_y + (max_events as i32 - 1) * (event_height + EVENT_GAP) - EVENT_GAP;
                    let height = event_height;
                    let allocation = gtk::Allocation::new(x, y, width, height);

                    overflow_widget.size_allocate(&allocation, baseline);
                }
            }

            for (i, column_separator) in column_separators.iter().enumerate() {
                let separator_x = column_separator_xs[i];
                let separator_allocation =
                    gtk::Allocation::new(separator_x, 0, SEPARATOR_WIDTH, height);
                column_separator.size_allocate(&separator_allocation, baseline);
            }

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
    impl MonthViewInner {
        /// Sets the date displayed in the view.
        fn set_date(&self, date: Date) {
            if self.date.get() == date {
                return;
            }

            self.cancel_animation_and_clear_input();

            let system = Application::default().system();
            let first_cell_date = {
                let first_day_of_week = system.first_week_day();

                date.previous_occurrence_of_weekday(first_day_of_week)
                    .to_jiff()
                    - (NB_ROWS_ABOVE_AT_RESET * 7).days()
            };

            self.first_cell_index.set(0);

            self.scroll_offset
                .set((NB_ROWS_ABOVE_AT_RESET * (self.cell_height.get() + SEPARATOR_HEIGHT)) as f64);

            for (i, cell) in self.cells.get().unwrap().iter().enumerate() {
                let date: Date = (first_cell_date + (i as i32).days()).into();
                cell.set_date(date);
            }

            for (i, overflow_widget) in self.overflow_widgets.get().unwrap().iter().enumerate() {
                let date: Date = (first_cell_date + (i as i32).days()).into();
                overflow_widget.set_date(date);
            }

            let date_to_set = (first_cell_date + (NB_ROWS_ABOVE_AT_RESET * 7 + 6).days()).into();
            self.date.set(date_to_set);

            self.update_subscription_timeframe();
            self.recompute_event_layouts();

            self.obj().notify_date();

            self.obj().queue_allocate();
        }

        /// Sets the styling used for the view.
        fn set_styling(&self, styling: Styling) {
            if self.styling.get() == styling {
                return;
            }

            self.styling.set(styling);

            for cell in self.cells.get().unwrap() {
                cell.set_styling(styling);
            }
            for event_widget in self.event_widgets.borrow().iter() {
                event_widget.set_styling(styling);
            }
            for overflow_widget in self.overflow_widgets.get().unwrap() {
                overflow_widget.set_styling(styling);
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
                // Handle CTRL+scroll to zoom
                gdk::ScrollDirection::Up | gdk::ScrollDirection::Down => {
                    if scroll_controller
                        .current_event_state()
                        .contains(gdk::ModifierType::CONTROL_MASK)
                    {
                        let scale = dy / 10.0 + 1.0;

                        self.start_discrete_zoom_animation(scale);

                        return true;
                    }
                }
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
            // FIXME:
            // We require this check, because zoom on touchscreen triggers drag gestures that don't
            // get canceled
            if let Some(Input::ContinuousZoom { .. }) = *self.input.borrow() {
                return;
            };

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
                // When zooming on a touchscreen, a drag gesture stays active with its event
                // sequence set to NONE instead of DENIED (even though it is CLAIMED by the zoom
                // gesture). We need to deny it here to ensure swipe will not happen.
                // See https://gitlab.gnome.org/GNOME/gtk/-/work_items/8385
                if let Some(Input::ContinuousZoom { .. }) = *self.input.borrow() {
                    gesture_drag.set_state(gtk::EventSequenceState::Denied);
                }
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

        #[template_callback]
        fn motion_enter(&self, _x: f64, y: f64, _motion_controller: gtk::EventControllerMotion) {
            self.pointer_y.set(Some(y));
        }

        #[template_callback]
        fn motion(&self, _x: f64, y: f64, _motion_controller: gtk::EventControllerMotion) {
            self.pointer_y.set(Some(y));
        }

        #[template_callback]
        fn motion_leave(&self) {
            self.pointer_y.set(None);
        }

        #[template_callback]
        fn zoom_begin(&self, _event_sequence: gdk::EventSequence, gesture_zoom: gtk::GestureZoom) {
            self.cancel_animation_and_clear_input();

            self.input.replace(Some(Input::ContinuousZoom {
                last_scale_delta: 1.0,
            }));

            // Setting state might call other callbacks inline, so we need to do it after replacing
            // input
            gesture_zoom.set_state(gtk::EventSequenceState::Claimed);
        }

        #[template_callback]
        fn zoom_scale_changed(&self, scale: f64, gesture_zoom: gtk::GestureZoom) {
            let Some(Input::ContinuousZoom { last_scale_delta }) = *self.input.borrow() else {
                return;
            };

            let scale_delta = scale / last_scale_delta;
            self.input.replace(Some(Input::ContinuousZoom {
                last_scale_delta: scale,
            }));

            let old_cell_height = self.cell_height.get();
            let new_cell_height = ((old_cell_height as f64 * scale_delta) as i32)
                .clamp(MINIMUM_CELL_HEIGHT, MAXIMUM_CELL_HEIGHT);

            let Some((_x_center, y_center)) = gesture_zoom.bounding_box_center() else {
                return;
            };

            self.apply_zoom_height(new_cell_height, y_center);
        }

        #[template_callback]
        fn zoom_end(&self) {
            let Some(Input::ContinuousZoom { .. }) = *self.input.borrow() else {
                return;
            };

            self.input.replace(None);
        }

        #[template_callback]
        fn create_drag_begin(&self, start_x: f64, start_y: f64, gesture_drag: gtk::GestureDrag) {
            let Some(last_event) = gesture_drag.last_event(None) else {
                return;
            };

            // Deny touchscreen presses. Creation of events on touchscreen should only happen after
            // a long press.
            if last_event.device().unwrap().source() == gdk::InputSource::Touchscreen {
                gesture_drag.set_state(gtk::EventSequenceState::Denied);
                return;
            }

            self.cancel_animation();

            // Deny presses that land on an event widget, an overflow button or the floating controls.
            let picked = self
                .obj()
                .pick(start_x, start_y, gtk::PickFlags::DEFAULT)
                .expect("A widget should be picked");
            let on_event_widget = picked.ancestor(MonthViewEvent::static_type()).is_some();
            let on_overflow_widget = picked.ancestor(MonthViewOverflow::static_type()).is_some();
            let on_floating_controls = picked
                .ancestor(MonthViewFloatingControls::static_type())
                .is_some();
            if on_event_widget || on_overflow_widget || on_floating_controls {
                gesture_drag.set_state(gtk::EventSequenceState::Denied);
                return;
            }

            let anchor = self.date_at_coords(start_x, start_y);
            self.create_drag.replace(Some(CreateDrag {
                anchor,
                hover: anchor,
            }));

            let cells = self.cells.get().unwrap();
            let first_cell_index = self.first_cell_index.get();
            let last_cell_index = (first_cell_index + NB_CELLS - 1) % NB_CELLS;
            let first_cell_date = cells[first_cell_index].date();
            let last_cell_date = cells[last_cell_index].date();

            let anchor_cell_index = if anchor.to_jiff() < first_cell_date.to_jiff() {
                first_cell_index
            } else if anchor.to_jiff() > last_cell_date.to_jiff() {
                last_cell_index
            } else {
                let difference = (anchor.to_jiff() - first_cell_date.to_jiff()).get_days() as usize;
                (first_cell_index + difference) % NB_CELLS
            };

            cells[anchor_cell_index].set_state_flags(gtk::StateFlags::ACTIVE, false);
        }

        #[template_callback]
        fn create_drag_update(&self, offset_x: f64, offset_y: f64, gesture_drag: gtk::GestureDrag) {
            let Some(last_event) = gesture_drag.last_event(None) else {
                return;
            };

            // Deny touchscreen presses. Creation of events on touchscreen should only happen after
            // a long press.
            if last_event.device().unwrap().source() == gdk::InputSource::Touchscreen {
                gesture_drag.set_state(gtk::EventSequenceState::Denied);
                return;
            }

            let Some(CreateDrag { anchor, .. }) = self.create_drag.get() else {
                return;
            };

            if self.obj().drag_check_threshold(0, 0, 0, offset_y as i32) {
                gesture_drag.set_state(gtk::EventSequenceState::Claimed);
            }

            let (start_x, start_y) = gesture_drag.start_point().unwrap();
            let x = start_x + offset_x;
            let y = start_y + offset_y;

            let hover = self.date_at_coords(x, y);
            self.create_drag.set(Some(CreateDrag { anchor, hover }));

            let cells = self.cells.get().unwrap();
            let first_cell_index = self.first_cell_index.get();
            let last_cell_index_not_normalized = first_cell_index + NB_CELLS - 1;
            let last_cell_index = last_cell_index_not_normalized % NB_CELLS;
            let first_cell_date = cells[first_cell_index].date();
            let last_cell_date = cells[last_cell_index].date();

            let anchor_index_not_normalized = if anchor.to_jiff() < first_cell_date.to_jiff() {
                first_cell_index
            } else if anchor.to_jiff() > last_cell_date.to_jiff() {
                last_cell_index_not_normalized
            } else {
                let anchor_difference = (anchor.to_jiff() - first_cell_date.to_jiff()).get_days();
                assert!(anchor_difference >= 0);
                first_cell_index + anchor_difference as usize
            };

            let hover_index_not_normalized = if hover.to_jiff() < first_cell_date.to_jiff() {
                first_cell_index
            } else if hover.to_jiff() > last_cell_date.to_jiff() {
                last_cell_index_not_normalized
            } else {
                let hover_difference = (hover.to_jiff() - first_cell_date.to_jiff()).get_days();
                assert!(hover_difference >= 0);
                first_cell_index + hover_difference as usize
            };

            let (start_index_not_normalized, end_index_not_normalized) =
                if anchor_index_not_normalized < hover_index_not_normalized {
                    (anchor_index_not_normalized, hover_index_not_normalized)
                } else {
                    (hover_index_not_normalized, anchor_index_not_normalized)
                };

            for i in first_cell_index..start_index_not_normalized {
                cells[i % NB_CELLS].unset_state_flags(gtk::StateFlags::ACTIVE);
            }
            for i in start_index_not_normalized..=end_index_not_normalized {
                cells[i % NB_CELLS].set_state_flags(gtk::StateFlags::ACTIVE, false);
            }
            for i in end_index_not_normalized + 1..last_cell_index_not_normalized {
                cells[i % NB_CELLS].unset_state_flags(gtk::StateFlags::ACTIVE);
            }
        }

        #[template_callback]
        fn create_drag_end(&self, _offset_x: f64, _offset_y: f64, _gesture_drag: gtk::GestureDrag) {
            let cells = self.cells.get().unwrap();
            for cell in cells {
                cell.unset_state_flags(gtk::StateFlags::ACTIVE);
            }

            let Some(CreateDrag { anchor, hover }) = self.create_drag.get() else {
                return;
            };

            let (start, end) = if anchor.to_jiff() < hover.to_jiff() {
                (anchor.to_jiff(), hover.to_jiff())
            } else {
                (hover.to_jiff(), anchor.to_jiff())
            };

            self.create_drag.set(None);

            let tzid = Application::default()
                .system()
                .datetime()
                .timezone()
                .identifier();
            let jiff_tz = jiff::tz::TimeZone::get(&tzid).unwrap();

            let preset = EventPropertiesPreset {
                all_day: true,
                start: start.to_zoned(jiff_tz.clone()).unwrap().to_string(),
                end: end
                    .tomorrow()
                    .unwrap()
                    .to_zoned(jiff_tz)
                    .unwrap()
                    .to_string(),
                ..Default::default()
            };

            let _ = self
                .obj()
                .activate_action("win.create-event", Some(&preset.to_variant()));
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
            let row_height = (self.cell_height.get() + SEPARATOR_HEIGHT) as f64;

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

        pub(super) fn start_discrete_zoom_animation(&self, scale: f64) {
            let y_center = self
                .pointer_y
                .get()
                .unwrap_or(self.obj().height() as f64 / 2.);

            let baseline =
                if let Some(Input::Animation(Animation::DiscreteZoom { target_height, .. })) =
                    *self.input.borrow()
                {
                    target_height
                } else {
                    self.cell_height.get()
                };

            self.cancel_animation_and_clear_input();

            let target_height =
                ((baseline as f64 * scale) as i32).clamp(MINIMUM_CELL_HEIGHT, MAXIMUM_CELL_HEIGHT);

            let frame_clock = self.obj().frame_clock().unwrap();
            let now = frame_clock.frame_time();
            let start_height = self.cell_height.get();

            let tick_id = self.obj().add_tick_callback(clone!(
                #[weak(rename_to = imp)]
                self,
                #[upgrade_or]
                glib::ControlFlow::Break,
                move |_obj, frame_clock| imp.discrete_zoom_animation_tick(frame_clock.frame_time())
            ));
            self.input
                .replace(Some(Input::Animation(Animation::DiscreteZoom {
                    start_height,
                    target_height,
                    start_time: now,
                    y_center,
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

        fn discrete_zoom_animation_tick(&self, frame_time: i64) -> glib::ControlFlow {
            let Some(Input::Animation(Animation::DiscreteZoom {
                start_height,
                target_height,
                start_time,
                y_center,
                ..
            })) = *self.input.borrow_mut()
            else {
                error!(
                    "The discrete zoom animation should be cancelled properly by removing the tick callback"
                );
                return glib::ControlFlow::Break;
            };

            let duration_us = DISCRETE_ZOOM_ANIMATION_MS as i64 * 1000;
            let elapsed = frame_time - start_time;
            let t = (elapsed as f64 / duration_us as f64).clamp(0.0, 1.0);

            // Ease-out cubic: decelerate towards the target.
            let eased = 1.0 - (1.0 - t).powi(3);

            let new_height =
                (start_height as f64 + (target_height - start_height) as f64 * eased) as i32;

            self.apply_zoom_height(new_height, y_center);

            if t >= 1.0 {
                self.input.replace(None);
                return glib::ControlFlow::Break;
            }

            glib::ControlFlow::Continue
        }

        // Cancels any ongoing scroll animation, but if the input is not an animation, does nothing.
        fn cancel_animation(&self) {
            let is_animation = matches!(*self.input.borrow(), Some(Input::Animation(_)));

            if is_animation {
                let Input::Animation(animation) = self.input.borrow_mut().take().unwrap() else {
                    unreachable!()
                };
                match animation {
                    Animation::KineticDeceleration { tick_id, .. } => tick_id.remove(),
                    Animation::DiscreteScroll { tick_id, .. } => tick_id.remove(),
                    Animation::DiscreteZoom { tick_id, .. } => tick_id.remove(),
                }
            }
        }

        // Cancels any ongoing scroll animation, and resets the current input to None.
        fn cancel_animation_and_clear_input(&self) {
            if let Some(Input::Animation(animation)) = self.input.borrow_mut().take() {
                match animation {
                    Animation::KineticDeceleration { tick_id, .. } => tick_id.remove(),
                    Animation::DiscreteScroll { tick_id, .. } => tick_id.remove(),
                    Animation::DiscreteZoom { tick_id, .. } => tick_id.remove(),
                }
            }
        }

        fn apply_zoom_height(&self, new_cell_height: i32, y_center: f64) {
            let old_cell_height = self.cell_height.get();

            let old_row_height = (old_cell_height + SEPARATOR_HEIGHT) as f64;
            let new_row_height = (new_cell_height + SEPARATOR_HEIGHT) as f64;

            let old_scroll_offset = self.scroll_offset.get();
            let new_scroll_offset =
                (old_scroll_offset + y_center) * new_row_height / old_row_height - y_center;

            self.cell_height.set(new_cell_height);
            self.scroll_offset.set(new_scroll_offset);
            self.recycle_if_needed();

            self.obj().queue_allocate();
        }

        fn date_at_coords(&self, x: f64, y: f64) -> Date {
            let cells = self.cells.get().unwrap();
            let x = x as i32;
            let first_cell_index = self.first_cell_index.get();

            let cell_height = self.cell_height.get();
            let row_height = (cell_height + SEPARATOR_HEIGHT) as f64;
            let scroll_offset = self.scroll_offset.get();

            // Width is distributed evenly. Any remainder pixels are given to the leftmost columns,
            // so the first (width % 7) columns are one pixel wider than the rest.
            let column_widths: [i32; 7] = {
                let width = self.obj().width() - 6 * SEPARATOR_WIDTH;
                let base = width / 7;
                let remainder = width % 7;

                std::array::from_fn(|i| base + i32::from((i as i32) < remainder))
            };
            let column_separator_xs: [i32; 6] = {
                std::array::from_fn(|i| {
                    column_widths[..i + 1].iter().sum::<i32>() + i as i32 * SEPARATOR_WIDTH
                })
            };

            let row_number = ((scroll_offset + y) / row_height) as usize;
            let column_number = column_separator_xs
                .iter()
                .take_while(|column_separator| x > **column_separator)
                .count();

            let cell = cells
                .get((first_cell_index + row_number * 7 + column_number) % NB_CELLS)
                .unwrap();
            cell.date()
        }

        fn recycle_if_needed(&self) {
            let cells = self.cells.get().unwrap();
            let overflow_widgets = self.overflow_widgets.get().unwrap();

            let row_height = (self.cell_height.get() + SEPARATOR_HEIGHT) as f64;
            let height = self.obj().height() as f64;

            let top_threshold = row_height * MINIMUM_NB_ROWS_ABOVE as f64;
            let bottom_threshold = (NB_ROWS as f64 - MINIMUM_NB_ROWS_BELOW as f64) * row_height;

            let mut shifted = false;

            loop {
                let scroll_offset = self.scroll_offset.get();
                let first_cell_index = self.first_cell_index.get();

                if scroll_offset < top_threshold {
                    shifted = true;

                    let new_index = (first_cell_index + NB_CELLS - 7) % NB_CELLS;
                    let old_first_date = cells[first_cell_index].date().to_jiff();
                    let new_first_date = old_first_date - 7.days();
                    for i in 0..7 {
                        let date = Date::from(new_first_date + (i as i32).days());
                        cells[(new_index + i) % NB_CELLS].set_date(date);
                        overflow_widgets[(new_index + i) % NB_CELLS].set_date(date);
                    }

                    self.first_cell_index.set(new_index);
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
                    shifted = true;

                    let new_index = (first_cell_index + 7) % NB_CELLS;
                    let old_last_date = cells[(first_cell_index + NB_CELLS - 1) % NB_CELLS]
                        .date()
                        .to_jiff();
                    for i in 0..7 {
                        let date = Date::from(old_last_date + (i as i32 + 1).days());
                        cells[(first_cell_index + i) % NB_CELLS].set_date(date);
                        overflow_widgets[(first_cell_index + i) % NB_CELLS].set_date(date);
                    }

                    self.first_cell_index.set(new_index);
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

            let first_cell_index = self.first_cell_index.get();

            let rows_above_view = (self.scroll_offset.get() / row_height).ceil() as usize;
            let first_visible_cell_index = (first_cell_index + rows_above_view * 7) % NB_CELLS;
            let last_cell_of_first_visible_row = (first_visible_cell_index + 6) % NB_CELLS;
            let new_date = cells[last_cell_of_first_visible_row].date();

            if self.date.get() != new_date {
                self.date.set(new_date);
                self.obj().notify_date();
            }

            if shifted {
                self.update_subscription_timeframe();
                // TODO: We could just shift the layouts
                self.recompute_event_layouts();
            }
        }

        fn update_subscription_timeframe(&self) {
            let timezone = Application::default().system().datetime().timezone();

            let cells = self.cells.get().unwrap();
            let first_cell_index = self.first_cell_index.get();
            let last_cell_index = (first_cell_index + NB_CELLS - 1) % NB_CELLS;

            let start = cells[first_cell_index].date().to_glib_date_time(&timezone);
            let end = Date::from(cells[last_cell_index].date().to_jiff().tomorrow().unwrap())
                .to_glib_date_time(&timezone);

            self.subscription.get().unwrap().set_timeframe(&start, &end);
        }

        fn recompute_event_layouts(&self) {
            let cells = self.cells.get().unwrap();
            let subscription = self.subscription.get().unwrap();
            let first_cell_index = self.first_cell_index.get();
            let timezone = Application::default().system().datetime().timezone();

            let events: Vec<Event> = (0..subscription.n_items())
                .map(|i| subscription.item(i).unwrap().downcast::<Event>().unwrap())
                .collect();

            let buffer_start_unix_days =
                (cells[first_cell_index].date().to_jiff() - UNIX_EPOCH_DATE).get_days() as i64;

            let segments = compute_event_segments(&events, buffer_start_unix_days, &timezone);

            for segment in &segments {
                let event = segment.event.clone();
                let uri = event.uri().unwrap();

                if !self.connected_event_uris.borrow_mut().insert(uri) {
                    continue;
                }

                event.connect_timeframe_notify(clone!(
                    #[weak(rename_to = imp)]
                    self,
                    move |_event| {
                        imp.recompute_event_layouts();
                    }
                ));
            }

            let stacked_segments_by_row = stack_event_segments(&segments);
            let total_segments = stacked_segments_by_row.iter().map(Vec::len).sum();

            let mut event_widgets = self.event_widgets.borrow_mut();

            while event_widgets.len() < total_segments {
                let event_widget = MonthViewEvent::new(None);
                event_widget.set_child_visible(false);
                event_widget.insert_before(&*self.obj(), Some(&self.floating_controls.get()));
                event_widgets.push(event_widget);
            }

            let event_layouts = build_event_layouts(&event_widgets, &stacked_segments_by_row);

            for event_widget in event_widgets.iter().skip(total_segments) {
                event_widget.set_child_visible(false);
            }

            drop(event_widgets);

            self.event_layouts.replace(event_layouts);

            self.obj().queue_allocate();
        }
    }
}

glib::wrapper! {
    pub struct MonthViewInner(ObjectSubclass<imp::MonthViewInner>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl MonthViewInner {
    pub fn scroll_up(&self) {
        self.imp().accumulate_discrete_scroll(true);
    }

    pub fn scroll_down(&self) {
        self.imp().accumulate_discrete_scroll(false);
    }

    pub fn zoom_in(&self) {
        self.imp().start_discrete_zoom_animation(10.0 / 9.0);
    }

    pub fn zoom_out(&self) {
        self.imp().start_discrete_zoom_animation(9.0 / 10.0);
    }
}
