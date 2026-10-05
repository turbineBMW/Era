//! One month at a time, like a desk calendar.
//!
//! Fork-only (see FORK.md). Takes the place of upstream's continuously
//! scrolling `MonthViewInner` inside `MonthView`. The month on show fills the
//! view in four to six week rows; the days of the neighbouring months that
//! complete the first and last week are greyed. The mouse wheel, a touchpad
//! scroll or a touchscreen swipe turns the page, as do `MonthView::scroll_up`
//! and `MonthView::scroll_down`.
//!
//! Event placement reuses upstream's `layout_utils` and widgets unchanged.

use std::{
    cell::{Cell, OnceCell, RefCell},
    collections::HashSet,
};

use adw::{prelude::*, subclass::prelude::*};
use clepsydre::{Calendar, Event, Subscription, prelude::*};
use glib::{GString, clone};
use jiff::ToSpan;

use crate::{
    application::Application,
    utils::{Date, EventPropertiesPreset},
    widgets::window::Styling,
};

use super::{
    layout_utils::{
        EventLayout, RowOverflow, build_event_layouts, compute_event_segments, stack_event_segments,
    },
    month_view_event::MonthViewEvent,
    month_view_overflow::MonthViewOverflow,
    month_view_paged_cell::MonthViewPagedCell,
};

/// No month spans more than six weeks.
const MAX_ROWS: usize = 6;
const MAX_CELLS: usize = 7 * MAX_ROWS;

const EVENT_GAP: i32 = 2;
const SEPARATOR_HEIGHT: i32 = 1;
const SEPARATOR_WIDTH: i32 = 1;

const INITIAL_EVENT_WIDGET_POOL_SIZE: usize = 60;

/// Touchpad scroll distance, in pixels, that turns the page. One gesture turns
/// at most one page.
const TOUCHPAD_PAGE_DISTANCE: f64 = 60.0;
/// Touchscreen swipe speed, in pixels per second, that turns the page.
const SWIPE_PAGE_VELOCITY: f64 = 300.0;

const UNIX_EPOCH_DATE: jiff::civil::Date = jiff::civil::date(1970, 1, 1);

#[derive(Debug, Clone, Copy)]
struct CreateDrag {
    anchor: Date,
    hover: Date,
}

/// Splits `total` pixels between `count` tracks separated by 1px lines, giving
/// the remainder to the first tracks.
fn track_sizes<const N: usize>(total: i32, count: usize) -> [i32; N] {
    let count = count.clamp(1, N) as i32;
    let available = (total - (count - 1) * SEPARATOR_WIDTH).max(0);
    let base = available / count;
    let remainder = available % count;

    std::array::from_fn(|i| {
        let i = i as i32;
        if i < count {
            base + i32::from(i < remainder)
        } else {
            0
        }
    })
}

/// Start position of each track laid out by [`track_sizes`].
fn track_starts<const N: usize>(sizes: &[i32; N]) -> [i32; N] {
    std::array::from_fn(|i| sizes[..i].iter().sum::<i32>() + i as i32 * SEPARATOR_WIDTH)
}

fn first_of_month(date: Date) -> Date {
    date.to_jiff().first_of_month().into()
}

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(file = "data/resources/ui/views/month_view/month_view_paged.blp")]
    #[properties(wrapper_type = super::MonthViewPaged)]
    pub struct MonthViewPaged {
        /// The first day of the month on show. Setting any day shows its month.
        #[property(get, set = Self::set_date)]
        date: Cell<Date>,
        #[property(get, set = Self::set_styling, construct, builder(Styling::default()))]
        styling: Cell<Styling>,

        cells: OnceCell<[MonthViewPagedCell; MAX_CELLS]>,
        overflow_widgets: OnceCell<[MonthViewOverflow; MAX_CELLS]>,
        column_separators: OnceCell<[gtk::Separator; 6]>,
        row_separators: OnceCell<[gtk::Separator; MAX_ROWS - 1]>,

        /// Number of week rows the month on show needs.
        n_rows: Cell<usize>,

        subscription: OnceCell<Subscription>,

        event_widgets: RefCell<Vec<MonthViewEvent>>,
        event_layouts: RefCell<Vec<Vec<EventLayout>>>,
        recompute_pending: Cell<bool>,
        connected_event_uris: RefCell<HashSet<GString>>,

        /// Scroll distance accumulated towards the next page turn.
        scroll_distance: Cell<f64>,
        /// Whether the current touchpad gesture has already turned the page.
        scroll_turned_page: Cell<bool>,

        create_drag: Cell<Option<CreateDrag>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MonthViewPaged {
        const NAME: &'static str = "MonthViewPaged";
        type Type = super::MonthViewPaged;
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
    impl ObjectImpl for MonthViewPaged {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            let application = Application::default();
            let manager = application.manager();
            let system = application.system();
            let calendars_model = manager.calendars_model().unwrap();

            let today = system.date();
            self.date.set(first_of_month(today));

            let cells = std::array::from_fn(|_| {
                let cell = MonthViewPagedCell::new(today);
                cell.set_parent(&*obj);
                cell
            });

            let column_separators = std::array::from_fn(|_| {
                let separator = gtk::Separator::new(gtk::Orientation::Vertical);
                separator.set_parent(&*obj);
                separator
            });

            let row_separators = std::array::from_fn(|_| {
                let separator = gtk::Separator::new(gtk::Orientation::Horizontal);
                separator.set_parent(&*obj);
                separator
            });

            let overflow_widgets = std::array::from_fn(|_| {
                let overflow_widget = MonthViewOverflow::new(today);
                overflow_widget.set_child_visible(false);
                overflow_widget.set_parent(&*obj);
                overflow_widget
            });

            let event_widgets = (0..INITIAL_EVENT_WIDGET_POOL_SIZE)
                .map(|_| {
                    let event_widget = MonthViewEvent::new(None);
                    event_widget.set_child_visible(false);
                    event_widget.set_parent(&*obj);
                    event_widget
                })
                .collect::<Vec<_>>();

            self.cells.set(cells).unwrap();
            self.column_separators.set(column_separators).unwrap();
            self.row_separators.set(row_separators).unwrap();
            self.overflow_widgets.set(overflow_widgets).unwrap();
            self.event_widgets.replace(event_widgets);

            self.update_grid();

            let (start, end) = self.timeframe();
            let subscription = manager.new_subscription(&start, &end).unwrap();
            subscription.connect_items_changed(clone!(
                #[weak(rename_to = imp)]
                self,
                move |_model, _position, _removed, _added| {
                    // Debounce: a sync can fire this many times in a row.
                    if imp.recompute_pending.replace(true) {
                        return;
                    }

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
            self.subscription.set(subscription).unwrap();

            system.connect_first_week_day_notify(clone!(
                #[weak(rename_to = imp)]
                self,
                move |_system| {
                    imp.update_grid();
                    imp.update_subscription_timeframe();
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

            let watch_calendar = clone!(
                #[weak(rename_to = imp)]
                self,
                move |calendar: Calendar| {
                    calendar.connect_visible_notify(clone!(
                        #[weak]
                        imp,
                        move |_calendar| {
                            imp.recompute_event_layouts();
                        }
                    ));
                }
            );

            for i in 0..calendars_model.n_items() {
                watch_calendar(calendars_model.item(i).unwrap().downcast().unwrap());
            }

            calendars_model.connect_items_changed(move |model, position, _removed, added| {
                for i in 0..added {
                    watch_calendar(model.item(position + i).unwrap().downcast().unwrap());
                }
            });

            self.recompute_event_layouts();
        }

        fn dispose(&self) {
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for MonthViewPaged {
        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            let cells = self.cells.get().unwrap();
            let column_separators = self.column_separators.get().unwrap();
            let row_separators = self.row_separators.get().unwrap();
            let overflow_widgets = self.overflow_widgets.get().unwrap();
            let n_rows = self.n_rows.get();

            let column_widths: [i32; 7] = track_sizes(width, 7);
            let column_xs = track_starts(&column_widths);
            let row_heights: [i32; MAX_ROWS] = track_sizes(height, n_rows);
            let row_ys = track_starts(&row_heights);

            let (minimum_event_height, natural_event_height, ..) = self.event_widgets.borrow()[0]
                .measure(gtk::Orientation::Vertical, column_widths[6]);
            let header_height = cells[0].header_height(column_widths[6]);

            let event_layouts = self.event_layouts.borrow();

            for row_index in 0..MAX_ROWS {
                let row_cells = row_index * 7..row_index * 7 + 7;

                if row_index >= n_rows {
                    for cell in &cells[row_cells.clone()] {
                        cell.set_child_visible(false);
                    }
                    for overflow_widget in &overflow_widgets[row_cells] {
                        overflow_widget.set_child_visible(false);
                    }
                    for layout in event_layouts.get(row_index).into_iter().flatten() {
                        layout.widget.set_child_visible(false);
                    }
                    continue;
                }

                let cell_y = row_ys[row_index];
                let cell_height = row_heights[row_index];

                for column_index in 0..7 {
                    let cell = &cells[row_index * 7 + column_index];
                    cell.set_child_visible(true);
                    cell.size_allocate(
                        &gtk::Allocation::new(
                            column_xs[column_index],
                            cell_y,
                            column_widths[column_index],
                            cell_height,
                        ),
                        baseline,
                    );
                }

                let use_dense_allocation =
                    cell_height - header_height < 2 * natural_event_height + EVENT_GAP;
                let event_height = if use_dense_allocation {
                    minimum_event_height
                } else {
                    natural_event_height
                };

                // A very short window can leave room for less than one event; still show one
                // row so the "+n" overflow stays reachable.
                let max_events = (((cell_height - header_height + EVENT_GAP)
                    / (event_height + EVENT_GAP).max(1))
                .max(1)) as usize;

                let events_y = cell_y + header_height;
                let row_layouts = event_layouts.get(row_index).map_or(&[][..], Vec::as_slice);
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

                    layout
                        .widget
                        .size_allocate(&gtk::Allocation::new(x, y, width, event_height), baseline);
                }

                for column_index in 0..7 {
                    let overflow_widget = &overflow_widgets[row_index * 7 + column_index];

                    let hidden_count = overflow.column_hidden_counts[column_index];
                    if hidden_count == 0 {
                        overflow_widget.set_child_visible(false);
                        continue;
                    }

                    overflow_widget.set_child_visible(true);
                    overflow_widget.set_text(format!("+{hidden_count}"));

                    let y =
                        events_y + (max_events as i32 - 1) * (event_height + EVENT_GAP) - EVENT_GAP;
                    overflow_widget.size_allocate(
                        &gtk::Allocation::new(
                            column_xs[column_index],
                            y,
                            column_widths[column_index],
                            event_height,
                        ),
                        baseline,
                    );
                }
            }

            for (row_index, row_separator) in row_separators.iter().enumerate() {
                if row_index + 1 >= n_rows {
                    row_separator.set_child_visible(false);
                    continue;
                }

                row_separator.set_child_visible(true);
                let y = row_ys[row_index] + row_heights[row_index];
                row_separator.size_allocate(
                    &gtk::Allocation::new(0, y, width, SEPARATOR_HEIGHT),
                    baseline,
                );
            }

            for (i, column_separator) in column_separators.iter().enumerate() {
                let x = column_xs[i] + column_widths[i];
                column_separator.size_allocate(
                    &gtk::Allocation::new(x, 0, SEPARATOR_WIDTH, height),
                    baseline,
                );
            }
        }
    }

    #[gtk::template_callbacks]
    impl MonthViewPaged {
        fn set_date(&self, date: Date) {
            let month_start = first_of_month(date);
            if self.date.get() == month_start {
                return;
            }

            self.date.set(month_start);

            self.update_grid();
            self.update_subscription_timeframe();
            self.recompute_event_layouts();

            self.obj().notify_date();
        }

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

        /// Shows the month `months` before (negative) or after (positive) the one on show.
        pub(super) fn turn_page(&self, months: i32) {
            let date = self.date.get().to_jiff() + months.months();
            self.obj().set_date(Date::from(date));
        }

        #[template_callback]
        fn scroll_begin(&self, _scroll_controller: gtk::EventControllerScroll) {
            self.scroll_distance.set(0.0);
            self.scroll_turned_page.set(false);
        }

        #[template_callback]
        fn scroll(&self, _dx: f64, dy: f64, scroll_controller: gtk::EventControllerScroll) -> bool {
            // Leave Ctrl+scroll to the zoom shortcuts.
            if scroll_controller
                .current_event_state()
                .contains(gdk::ModifierType::CONTROL_MASK)
            {
                return false;
            }

            let distance = self.scroll_distance.get() + dy;

            match scroll_controller.unit() {
                // A wheel notch is 1.0; high-resolution wheels send fractions of it.
                gdk::ScrollUnit::Wheel => {
                    if distance.abs() >= 1.0 {
                        self.scroll_distance.set(0.0);
                        self.turn_page(distance.signum() as i32);
                    } else {
                        self.scroll_distance.set(distance);
                    }
                }
                // Touchpad: one page per gesture, however long the swipe.
                _ => {
                    if self.scroll_turned_page.get() {
                        return true;
                    }

                    if distance.abs() >= TOUCHPAD_PAGE_DISTANCE {
                        self.scroll_distance.set(0.0);
                        self.scroll_turned_page.set(true);
                        self.turn_page(distance.signum() as i32);
                    } else {
                        self.scroll_distance.set(distance);
                    }
                }
            }

            true
        }

        #[template_callback]
        fn scroll_end(&self, _scroll_controller: gtk::EventControllerScroll) {
            self.scroll_distance.set(0.0);
            self.scroll_turned_page.set(false);
        }

        #[template_callback]
        fn swipe(&self, velocity_x: f64, velocity_y: f64, _gesture_swipe: gtk::GestureSwipe) {
            // Swiping towards the top or the left brings the next month in.
            let velocity = if velocity_x.abs() > velocity_y.abs() {
                velocity_x
            } else {
                velocity_y
            };

            if velocity.abs() >= SWIPE_PAGE_VELOCITY {
                self.turn_page(if velocity < 0.0 { 1 } else { -1 });
            }
        }

        #[template_callback]
        fn create_drag_begin(&self, start_x: f64, start_y: f64, gesture_drag: gtk::GestureDrag) {
            let Some(last_event) = gesture_drag.last_event(None) else {
                return;
            };

            // Creation of events on touchscreen should only happen after a long press.
            if last_event.device().unwrap().source() == gdk::InputSource::Touchscreen {
                gesture_drag.set_state(gtk::EventSequenceState::Denied);
                return;
            }

            // Deny presses that land on an event widget or an overflow button.
            let picked = self
                .obj()
                .pick(start_x, start_y, gtk::PickFlags::DEFAULT)
                .expect("A widget should be picked");
            if picked.ancestor(MonthViewEvent::static_type()).is_some()
                || picked.ancestor(MonthViewOverflow::static_type()).is_some()
            {
                gesture_drag.set_state(gtk::EventSequenceState::Denied);
                return;
            }

            let anchor = self.date_at_coords(start_x, start_y);
            self.create_drag.set(Some(CreateDrag {
                anchor,
                hover: anchor,
            }));
            self.highlight_range(anchor, anchor);
        }

        #[template_callback]
        fn create_drag_update(&self, offset_x: f64, offset_y: f64, gesture_drag: gtk::GestureDrag) {
            let Some(CreateDrag { anchor, .. }) = self.create_drag.get() else {
                return;
            };

            if self
                .obj()
                .drag_check_threshold(0, 0, offset_x as i32, offset_y as i32)
            {
                gesture_drag.set_state(gtk::EventSequenceState::Claimed);
            }

            let (start_x, start_y) = gesture_drag.start_point().unwrap();
            let hover = self.date_at_coords(start_x + offset_x, start_y + offset_y);
            self.create_drag.set(Some(CreateDrag { anchor, hover }));
            self.highlight_range(anchor, hover);
        }

        #[template_callback]
        fn create_drag_end(&self, _offset_x: f64, _offset_y: f64, _gesture_drag: gtk::GestureDrag) {
            for cell in self.cells.get().unwrap() {
                cell.unset_state_flags(gtk::StateFlags::ACTIVE);
            }

            let Some(CreateDrag { anchor, hover }) = self.create_drag.take() else {
                return;
            };

            let (start, end) = if anchor.to_jiff() < hover.to_jiff() {
                (anchor.to_jiff(), hover.to_jiff())
            } else {
                (hover.to_jiff(), anchor.to_jiff())
            };

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

        /// Marks the cells between `a` and `b`, inclusive, as active.
        fn highlight_range(&self, a: Date, b: Date) {
            let (start, end) = if a.to_jiff() < b.to_jiff() {
                (a.to_jiff(), b.to_jiff())
            } else {
                (b.to_jiff(), a.to_jiff())
            };

            for cell in self.cells.get().unwrap() {
                let date = cell.date().to_jiff();
                if start <= date && date <= end {
                    cell.set_state_flags(gtk::StateFlags::ACTIVE, false);
                } else {
                    cell.unset_state_flags(gtk::StateFlags::ACTIVE);
                }
            }
        }

        fn date_at_coords(&self, x: f64, y: f64) -> Date {
            let obj = self.obj();
            let n_rows = self.n_rows.get();

            let column_widths: [i32; 7] = track_sizes(obj.width(), 7);
            let column_xs = track_starts(&column_widths);
            let row_heights: [i32; MAX_ROWS] = track_sizes(obj.height(), n_rows);
            let row_ys = track_starts(&row_heights);

            let column = column_xs[1..]
                .iter()
                .take_while(|&&cx| x as i32 >= cx)
                .count();
            let row = row_ys[1..n_rows]
                .iter()
                .take_while(|&&ry| y as i32 >= ry)
                .count();

            self.cells.get().unwrap()[row * 7 + column].date()
        }

        /// Gives every cell its date for the month on show.
        fn update_grid(&self) {
            let cells = self.cells.get().unwrap();
            let overflow_widgets = self.overflow_widgets.get().unwrap();

            let month_start = self.date.get();
            let month = month_start.to_jiff().month();
            let first_week_day = Application::default().system().first_week_day();

            let first_cell_date = month_start
                .previous_occurrence_of_weekday(first_week_day)
                .to_jiff();
            let last_day_offset =
                (month_start.to_jiff().last_of_month() - first_cell_date).get_days() as usize;
            self.n_rows.set(last_day_offset / 7 + 1);

            for (i, (cell, overflow_widget)) in cells.iter().zip(overflow_widgets).enumerate() {
                let date = Date::from(first_cell_date + (i as i32).days());
                let other_month = date.to_jiff().month() != month;

                cell.set_date(date);
                cell.set_other_month(other_month);

                overflow_widget.set_date(date);
                if other_month {
                    overflow_widget.add_css_class("other-month");
                } else {
                    overflow_widget.remove_css_class("other-month");
                }
            }

            self.obj().queue_allocate();
        }

        /// The span of the visible week rows, as a subscription timeframe.
        fn timeframe(&self) -> (glib::DateTime, glib::DateTime) {
            let timezone = Application::default().system().datetime().timezone();
            let cells = self.cells.get().unwrap();
            let last_cell = &cells[self.n_rows.get() * 7 - 1];

            let start = cells[0].date().to_glib_date_time(&timezone);
            let end = Date::from(last_cell.date().to_jiff().tomorrow().unwrap())
                .to_glib_date_time(&timezone);

            (start, end)
        }

        fn update_subscription_timeframe(&self) {
            let (start, end) = self.timeframe();
            self.subscription.get().unwrap().set_timeframe(&start, &end);
        }

        fn recompute_event_layouts(&self) {
            let cells = self.cells.get().unwrap();
            let subscription = self.subscription.get().unwrap();
            let timezone = Application::default().system().datetime().timezone();
            let n_rows = self.n_rows.get();

            let events: Vec<Event> = (0..subscription.n_items())
                .map(|i| subscription.item(i).unwrap().downcast::<Event>().unwrap())
                .collect();

            let first_cell_unix_days =
                (cells[0].date().to_jiff() - UNIX_EPOCH_DATE).get_days() as i64;

            let mut segments = compute_event_segments(&events, first_cell_unix_days, &timezone);
            segments.retain(|segment| segment.row_index < n_rows);

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
                event_widget.set_styling(self.styling.get());
                event_widget.set_child_visible(false);
                event_widget.set_parent(&*self.obj());
                event_widgets.push(event_widget);
            }

            let event_layouts = build_event_layouts(&event_widgets, &stacked_segments_by_row);

            for event_widget in event_widgets.iter().skip(total_segments) {
                event_widget.set_child_visible(false);
            }

            drop(event_widgets);

            // Grey events that lie entirely in the neighbouring months.
            for (row_index, row_layouts) in event_layouts.iter().enumerate().take(n_rows) {
                for layout in row_layouts {
                    let other_month = (layout.column_start..=layout.column_end)
                        .all(|column| cells[row_index * 7 + column].other_month());

                    if other_month {
                        layout.widget.add_css_class("other-month");
                    } else {
                        layout.widget.remove_css_class("other-month");
                    }
                }
            }

            self.event_layouts.replace(event_layouts);

            self.obj().queue_allocate();
        }
    }
}

glib::wrapper! {
    pub struct MonthViewPaged(ObjectSubclass<imp::MonthViewPaged>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl MonthViewPaged {
    pub fn previous_month(&self) {
        self.imp().turn_page(-1);
    }

    pub fn next_month(&self) {
        self.imp().turn_page(1);
    }
}
