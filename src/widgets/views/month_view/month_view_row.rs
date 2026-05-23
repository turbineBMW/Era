use std::{
    cell::{Cell, OnceCell, RefCell},
    mem,
    sync::Mutex,
};

use adw::{prelude::*, subclass::prelude::*};
use clepsydre::{Calendar, Event, Subscription, Timeframe, prelude::*};
use glib::{DateTime, clone};
use jiff::ToSpan;

use crate::{Application, system_settings::DayOfWeek, utils};

use super::{MonthViewStyling, event_widget::EventWidget, month_view_header::MonthViewHeader};

pub const MINIMUM_WIDTH: i32 = 0;
pub const NATURAL_WIDTH: i32 = 0;
pub const MINIMUM_HEIGHT: i32 = 92;
pub const NATURAL_HEIGHT: i32 = MINIMUM_HEIGHT * 3;

const HEADER_HEIGHT: i32 = 30;
const EVENT_HEIGHT: i32 = 30;
const MORE_BUTTON_HEIGHT: i32 = 30;
const EVENT_GAP: i32 = 2;

struct SpannedEvent {
    event_widget: EventWidget,
    column_start: usize,
    column_end: usize,
}

#[derive(Debug)]
struct EventLayout {
    event_widget: EventWidget,
    row: usize,
    column_start: usize,
    column_end: usize,
}

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Kalendasom/month_view_row.ui")]
    #[properties(wrapper_type = super::MonthViewRow)]
    pub struct MonthViewRow {
        #[property(get, set, construct_only)]
        year: Cell<i32>,
        #[property(get, set, construct_only)]
        month: Cell<i32>,
        #[property(get, set, construct_only)]
        day: Cell<i32>,
        #[property(get, set = Self::set_styling, builder(MonthViewStyling::default()))]
        styling: Cell<MonthViewStyling>,
        #[property(get)]
        subscription: OnceCell<Subscription>,

        /// Event widgets used to display events in this row.
        event_widgets: OnceCell<Mutex<Vec<EventWidget>>>,
        event_layouts: OnceCell<Mutex<Vec<EventLayout>>>,

        /// Cached day boundaries for the current week (8 boundaries for 7 columns).
        /// Timezone-aware boundaries for timed events.
        day_boundaries: RefCell<Option<[DateTime; 8]>>,
        /// UTC boundaries for all-day event comparisons.
        day_boundaries_utc: RefCell<Option<[DateTime; 8]>>,
        /// "N more..." buttons, one per column (7 total).
        pub(super) more_buttons: OnceCell<[gtk::Button; 7]>,

        /// Tracks the current focus position as an index into the focus order.
        /// This is needed because multi-day events appear in multiple columns
        /// and we need to remember which column context the focus came from.
        pub(super) focus_index: Cell<Option<usize>>,

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
        #[template_child]
        pub(super) cell_1: TemplateChild<adw::Bin>,
        #[template_child]
        pub(super) cell_2: TemplateChild<adw::Bin>,
        #[template_child]
        pub(super) cell_3: TemplateChild<adw::Bin>,
        #[template_child]
        pub(super) cell_4: TemplateChild<adw::Bin>,
        #[template_child]
        pub(super) cell_5: TemplateChild<adw::Bin>,
        #[template_child]
        pub(super) cell_6: TemplateChild<adw::Bin>,
        #[template_child]
        pub(super) cell_7: TemplateChild<adw::Bin>,
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

            assert!(
                DateTime::from_utc(self.year.get(), self.month.get(), self.day.get(), 0, 0, 0.)
                    .is_ok()
            );

            let obj = self.obj();

            let application = Application::default();
            let manager = application.manager();
            let calendars_model = manager.calendars_model().unwrap();

            self.event_widgets.get_or_init(|| Mutex::new(Vec::new()));
            self.event_layouts.get_or_init(|| Mutex::new(Vec::new()));

            // Create "more" buttons for each column
            let more_buttons: [gtk::Button; 7] = std::array::from_fn(|_column| {
                let button = gtk::Button::new();
                button.set_parent(&*obj);
                button.set_visible(false);
                button.add_css_class("flat");
                button.add_css_class("month-view-more-button");
                button.connect_clicked(clone!(
                    #[weak]
                    obj,
                    move |_btn| {
                        let dialog = adw::Dialog::builder().title("Events").build();
                        dialog.present(Some(&obj));
                    }
                ));
                button
            });
            self.more_buttons.get_or_init(|| more_buttons);

            // Setup subscription
            let (first_day, last_day) = self.row_date_range();
            let timeframe = Timeframe::new(true, &first_day, &last_day);
            let subscription = manager.new_subscription(&timeframe).unwrap().unwrap();
            self.subscription
                .set(subscription.clone())
                .expect("Subscription should not be initialized yet");

            self.update_days();
            self.update_timeframe();
            self.listen_to_event_timeframe_changes();
            self.update_event_widgets();
            self.update_styling();

            application
                .system_settings()
                .connect_first_day_of_week_notify(clone!(
                    #[weak(rename_to = imp)]
                    self,
                    move |_| {
                        imp.update_days();
                        imp.update_timeframe();
                    }
                ));
            application.connect_current_datetime_notify(clone!(
                #[weak(rename_to = imp)]
                self,
                move |_| {
                    imp.update_timeframe();
                }
            ));
            subscription.connect_items_changed(clone!(
                #[weak(rename_to = imp)]
                self,
                move |_model, _pos, _removed, _added| {
                    imp.listen_to_event_timeframe_changes();
                    imp.update_event_widgets();
                }
            ));

            for i in 0..calendars_model.n_items() {
                let calendar = calendars_model
                    .item(i)
                    .unwrap()
                    .downcast::<Calendar>()
                    .unwrap();
                calendar.connect_visible_notify(clone!(
                    #[weak(rename_to=imp)]
                    self,
                    move |_event| {
                        imp.invalidate_event_widgets_layout();
                    }
                ));
            }

            // TODO: Store the signal handlers somewhere to remove them on calendar removal
            calendars_model.connect_items_changed(clone!(
                #[weak(rename_to = imp)]
                self,
                move |model, position, _removed, added| {
                    for i in 0..added {
                        let calendar = model
                            .item(position + i)
                            .unwrap()
                            .downcast::<Calendar>()
                            .unwrap();
                        calendar.connect_visible_notify(clone!(
                            #[weak]
                            imp,
                            move |_event| {
                                imp.invalidate_event_widgets_layout();
                            }
                        ));
                    }
                },
            ));
        }

        fn dispose(&self) {
            let mut widgets = self.event_widgets.get().unwrap().lock().unwrap();
            for widget in widgets.drain(..) {
                widget.unparent();
            }

            for button in self.more_buttons.get().unwrap() {
                button.unparent();
            }

            self.overlay.unparent();
        }
    }

    impl WidgetImpl for MonthViewRow {
        fn grab_focus(&self) -> bool {
            // Never focus the row itself; redirect to first cell
            self.obj().child_focus(gtk::DirectionType::TabForward)
        }

        // TODO: Make this clearer
        fn focus(&self, direction: gtk::DirectionType) -> bool {
            let obj = self.obj();
            let root = match obj.root() {
                Some(r) => r,
                None => return false,
            };
            let focused = root.focus();

            // Build the focus order: for each column, cell then visible events then more button
            let focus_order = self.build_focus_order();

            if focus_order.is_empty() {
                return false;
            }

            let cells: [&adw::Bin; 7] = [
                &self.cell_1,
                &self.cell_2,
                &self.cell_3,
                &self.cell_4,
                &self.cell_5,
                &self.cell_6,
                &self.cell_7,
            ];

            // If focus is already inside this row
            if let Some(ref focused_widget) = focused
                && focused_widget.is_ancestor(&*obj)
            {
                match direction {
                    gtk::DirectionType::TabForward => {
                        let current_idx = self.focus_index.get().unwrap_or(0);
                        let next_idx = current_idx + 1;
                        if next_idx < focus_order.len() {
                            self.focus_index.set(Some(next_idx));
                            return focus_order[next_idx].grab_focus();
                        }
                        self.focus_index.set(None);
                        return false;
                    }
                    gtk::DirectionType::TabBackward => {
                        let current_idx = self.focus_index.get().unwrap_or(0);
                        if current_idx > 0 {
                            let prev_idx = current_idx - 1;
                            self.focus_index.set(Some(prev_idx));
                            return focus_order[prev_idx].grab_focus();
                        }
                        self.focus_index.set(None);
                        return false;
                    }
                    gtk::DirectionType::Left => {
                        let col = self.current_focus_column(focused_widget, &cells);
                        if col == 0 {
                            // Wrap to previous row, last column
                            return false;
                        }
                        return self.focus_cell(col - 1, &cells, &focus_order);
                    }
                    gtk::DirectionType::Right => {
                        let col = self.current_focus_column(focused_widget, &cells);
                        if col == 6 {
                            // Wrap to next row, first column
                            return false;
                        }
                        return self.focus_cell(col + 1, &cells, &focus_order);
                    }
                    gtk::DirectionType::Up | gtk::DirectionType::Down => {
                        // Let the parent (MonthViewInner) handle vertical navigation
                        return false;
                    }
                    _ => return self.parent_focus(direction),
                }
            }

            // Focus is entering the row from outside
            match direction {
                gtk::DirectionType::TabForward => {
                    self.focus_index.set(Some(0));
                    focus_order[0].grab_focus()
                }
                gtk::DirectionType::TabBackward => {
                    let last = focus_order.len() - 1;
                    self.focus_index.set(Some(last));
                    focus_order[last].grab_focus()
                }
                _ => self.parent_focus(direction),
            }
        }

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

        // TODO: Make this clearer
        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            let allocation = gtk::Allocation::new(0, 0, width, height);
            self.overlay.size_allocate(&allocation, baseline);

            let obj = self.obj();

            // Compute exact bounds of each cell
            let cells = [
                &self.cell_1,
                &self.cell_2,
                &self.cell_3,
                &self.cell_4,
                &self.cell_5,
                &self.cell_6,
                &self.cell_7,
            ];
            let bounds: Vec<(i32, i32)> = cells
                .iter()
                .map(|cell| {
                    let bounds = cell.compute_bounds(&*obj).unwrap();
                    let x = bounds.x();
                    let w = bounds.width();
                    (x.round() as i32, (x + w).round() as i32)
                })
                .collect();

            let event_layouts = self.event_layouts.get().unwrap().lock().unwrap();

            // Calculate how many event rows fit, reserving space for the "more" button
            let available_height = height - HEADER_HEIGHT;
            // Max rows that fit without needing a "more" button
            let max_rows_full = if available_height <= 0 {
                0usize
            } else {
                ((available_height + EVENT_GAP) / (EVENT_HEIGHT + EVENT_GAP)) as usize
            };

            // Determine per-column: how many events total, and max row index
            let mut column_max_row: [usize; 7] = [0; 7];
            let mut column_event_count: [usize; 7] = [0; 7];
            for layout in event_layouts.iter() {
                for column in layout.column_start..=layout.column_end.min(6) {
                    column_event_count[column] += 1;
                    if layout.row + 1 > column_max_row[column] {
                        column_max_row[column] = layout.row + 1;
                    }
                }
            }

            // For columns that overflow, we need to reserve one row for the "more" button,
            // so max visible row index becomes max_rows_full - 1 (the last visible slot is the
            // button).
            let mut column_needs_more = [false; 7];
            let mut column_max_visible_row: [usize; 7] = [max_rows_full; 7];
            for column in 0..7 {
                if column_max_row[column] > max_rows_full {
                    column_needs_more[column] = true;
                    // Reserve last row for button
                    column_max_visible_row[column] = if max_rows_full > 0 {
                        max_rows_full - 1
                    } else {
                        0
                    };
                }
            }

            // Allocate visible events and count hidden ones per column
            let mut column_hidden_count: [usize; 7] = [0; 7];
            for layout in event_layouts.iter() {
                // Check if this event is hidden in any of its columns
                let hidden = (layout.column_start..=layout.column_end.min(6))
                    .any(|col| layout.row >= column_max_visible_row[col] && column_needs_more[col]);

                if hidden {
                    layout.event_widget.set_visible(false);
                    for col in layout.column_start..=layout.column_end.min(6) {
                        if column_needs_more[col] {
                            column_hidden_count[col] += 1;
                        }
                    }
                } else {
                    layout.event_widget.set_visible(true);
                    let first_cell = layout.column_start.min(6);
                    let last_cell = layout.column_end.min(6);
                    let x_start = bounds[first_cell].0;
                    let x_end = bounds[last_cell].1;
                    let w = x_end - x_start;
                    let y = HEADER_HEIGHT + layout.row as i32 * (EVENT_HEIGHT + EVENT_GAP);

                    let alloc = gtk::Allocation::new(x_start, y, w, EVENT_HEIGHT);
                    layout.event_widget.size_allocate(&alloc, baseline);
                }
            }

            // Position "more" buttons
            for (column, button) in self.more_buttons.get().unwrap().iter().enumerate() {
                if column_needs_more[column] && column_hidden_count[column] > 0 {
                    let count = column_hidden_count[column];
                    button.set_label(&format!("+{count}"));
                    button.set_visible(true);

                    let x_start = bounds[column].0;
                    let w = bounds[column].1 - x_start;
                    let y = HEADER_HEIGHT
                        + column_max_visible_row[column] as i32 * (EVENT_HEIGHT + EVENT_GAP);

                    let alloc = gtk::Allocation::new(x_start, y, w, MORE_BUTTON_HEIGHT);
                    button.size_allocate(&alloc, baseline);
                } else {
                    button.set_visible(false);
                }
            }
        }
    }

    #[gtk::template_callbacks]
    impl MonthViewRow {
        // TODO: Make this clearer
        /// Builds the focus order for this row:
        /// For each column: cell, then visible event widgets in that column (sorted by row),
        /// then the "more" button if visible.
        pub(super) fn build_focus_order(&self) -> Vec<gtk::Widget> {
            let cells: [&adw::Bin; 7] = [
                &self.cell_1,
                &self.cell_2,
                &self.cell_3,
                &self.cell_4,
                &self.cell_5,
                &self.cell_6,
                &self.cell_7,
            ];

            let event_layouts = self.event_layouts.get().unwrap().lock().unwrap();
            let more_buttons = self.more_buttons.get();

            let mut order: Vec<gtk::Widget> = Vec::new();

            for col in 0..7 {
                // Add the cell
                order.push(cells[col].clone().upcast::<gtk::Widget>());

                // Add visible event widgets in this column, sorted by row
                let mut col_events: Vec<(usize, gtk::Widget)> = event_layouts
                    .iter()
                    .filter(|layout| {
                        layout.column_start <= col
                            && layout.column_end >= col
                            && layout.event_widget.is_visible()
                    })
                    .map(|layout| {
                        (
                            layout.row,
                            layout.event_widget.clone().upcast::<gtk::Widget>(),
                        )
                    })
                    .collect();
                col_events.sort_by_key(|(row, _)| *row);
                for (_, widget) in col_events {
                    order.push(widget);
                }

                // Add the "more" button if visible
                let buttons = more_buttons.unwrap();
                if buttons[col].is_visible() {
                    order.push(buttons[col].clone().upcast::<gtk::Widget>());
                }
            }

            order
        }

        // TODO: Make this clearer
        /// Determines which column the currently focused widget belongs to.
        pub(super) fn current_focus_column(
            &self,
            focused: &gtk::Widget,
            cells: &[&adw::Bin; 7],
        ) -> usize {
            // Check if focused widget is a cell directly
            for (col, cell) in cells.iter().enumerate() {
                let cell_widget: &gtk::Widget = cell.upcast_ref();
                if focused == cell_widget || focused.is_ancestor(cell_widget) {
                    return col;
                }
            }

            // Check if it's a "more" button
            if let Some(buttons) = self.more_buttons.get() {
                for (col, button) in buttons.iter().enumerate() {
                    let btn_widget: &gtk::Widget = button.upcast_ref();
                    if focused == btn_widget || focused.is_ancestor(btn_widget) {
                        return col;
                    }
                }
            }

            // It's an event widget — use the stored focus_index to determine column
            if let Some(idx) = self.focus_index.get() {
                let focus_order = self.build_focus_order();
                // Walk backwards from idx to find which cell it belongs to
                for i in (0..=idx).rev() {
                    for (col, cell) in cells.iter().enumerate() {
                        if focus_order.get(i) == Some(cell.upcast_ref::<gtk::Widget>()) {
                            return col;
                        }
                    }
                }
            }

            0
        }

        // TODO: Make this clearer
        /// Focuses the cell at the given column and updates focus_index.
        fn focus_cell(
            &self,
            col: usize,
            cells: &[&adw::Bin; 7],
            focus_order: &[gtk::Widget],
        ) -> bool {
            let target = cells[col].clone().upcast::<gtk::Widget>();
            if let Some(idx) = focus_order.iter().position(|w| *w == target) {
                self.focus_index.set(Some(idx));
            }
            cells[col].grab_focus()
        }

        /// Sets the triplet year-month-day.
        pub(super) fn set_year_month_day(&self, year: i32, month: i32, day: i32) {
            assert!(jiff::civil::Date::new(year as i16, month as i8, day as i8).is_ok());

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
            self.update_timeframe();
        }

        fn set_styling(&self, styling: MonthViewStyling) {
            if self.styling.get() == styling {
                return;
            }

            self.styling.set(styling);
            self.obj().notify_styling();

            self.update_styling();
        }

        /// Updates the styling class.
        fn update_styling(&self) {
            match self.styling.get() {
                MonthViewStyling::Narrow => {
                    self.obj().remove_css_class("medium");
                    self.obj().add_css_class("narrow");
                }
                MonthViewStyling::Medium => {
                    self.obj().add_css_class("medium");
                    self.obj().remove_css_class("narrow");
                }
            }
        }

        /// Returns (start, end) `glib::DateTime` for this row's week,
        /// accounting for first-day-of-week.
        fn row_date_range(&self) -> (DateTime, DateTime) {
            let year = self.year.get();
            let month = self.month.get();
            let day = self.day.get();

            let application = Application::default();
            let first_day_of_week = application.system_settings().first_day_of_week();
            let timezone = application.current_datetime().timezone();

            let date = DateTime::new(&timezone, year, month, day, 0, 0, 0.)
                .expect("DateTime should be valid");

            let first_day = utils::get_last_occurrence_of_weekday(date, first_day_of_week);
            let last_day = first_day.add_days(7).expect("DateTime should be valid");

            (first_day, last_day)
        }

        /// Updates the days of each cell in the row.
        fn update_days(&self) {
            let obj = self.obj();

            let year = obj.year() as i16;
            let month = obj.month() as i8;
            let day = obj.day() as i8;

            let Ok(date) = jiff::civil::Date::new(year, month, day) else {
                panic!("Invalid date: year={year}, month={month}, day={day}");
            };

            let first_day_of_week = Application::default().system_settings().first_day_of_week();

            let base = match first_day_of_week {
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

            let dates = [date_1, date_2, date_3, date_4, date_5, date_6, date_7];
            let cells: [&adw::Bin; 7] = [
                &self.cell_1,
                &self.cell_2,
                &self.cell_3,
                &self.cell_4,
                &self.cell_5,
                &self.cell_6,
                &self.cell_7,
            ];
            for (cell, date) in cells.iter().zip(dates.iter()) {
                let label = format!(
                    "{} {} {} {}",
                    utils::TemplateCallbacks::day_name(
                        first_day_of_week,
                        (date.weekday().to_monday_one_offset() - base + 1) as i32
                    ),
                    date.day(),
                    utils::TemplateCallbacks::month_name(date.month() as i32),
                    date.year()
                );
                cell.update_property(&[gtk::accessible::Property::Label(&label)]);
            }
        }

        fn update_timeframe(&self) {
            let (first_day, last_day) = self.row_date_range();
            let timeframe = Timeframe::new(true, &first_day, &last_day);
            self.subscription
                .get()
                .unwrap()
                .set_timeframe(Some(&timeframe));

            // Recompute cached day boundaries
            let boundaries: [DateTime; 8] = std::array::from_fn(|i| {
                first_day
                    .add_days(i as i32)
                    .expect("DateTime should be valid")
            });
            let boundaries_utc: [DateTime; 8] = std::array::from_fn(|i| {
                let day = &boundaries[i];
                DateTime::from_utc(day.year(), day.month(), day.day_of_month(), 0, 0, 0.).unwrap()
            });

            *self.day_boundaries.borrow_mut() = Some(boundaries);
            *self.day_boundaries_utc.borrow_mut() = Some(boundaries_utc);
        }

        fn listen_to_event_timeframe_changes(&self) {
            let subscription = self.subscription.get().unwrap();

            // TODO: Should we keep the signal handlers to clean them later?
            for i in 0..subscription.n_items() {
                let event = subscription.item(i).unwrap().downcast::<Event>().unwrap();
                event.connect_timeframe_notify(clone!(
                    #[weak(rename_to = imp)]
                    self,
                    move |_event| {
                        imp.invalidate_event_widgets_layout();
                    }
                ));
            }
        }

        fn update_event_widgets(&self) {
            let obj = self.obj();
            let subscription = self.subscription.get().unwrap();
            let mut event_widgets = self.event_widgets.get().unwrap().lock().unwrap();

            // TODO: Try to not delete all event widgets
            for event_widget in event_widgets.drain(..) {
                event_widget.unparent();
            }

            for i in 0..subscription.n_items() {
                let event = subscription.item(i).unwrap().downcast::<Event>().unwrap();

                event.connect_timeframe_notify(clone!(
                    #[weak(rename_to = imp)]
                    self,
                    move |_event| {
                        imp.invalidate_event_widgets_layout();
                    }
                ));

                let event_widget = EventWidget::new(&event);
                event_widget.set_parent(&*obj);

                event_widgets.push(event_widget);
            }

            mem::drop(event_widgets);

            self.invalidate_event_widgets_layout();
        }

        // TODO: Make this clearer
        fn invalidate_event_widgets_layout(&self) {
            let event_widgets = self.event_widgets.get().unwrap().lock().unwrap();
            let mut event_layouts = self.event_layouts.get().unwrap().lock().unwrap();

            event_layouts.clear();

            let day_boundaries_ref = self.day_boundaries.borrow();
            let day_boundaries = day_boundaries_ref.as_ref().unwrap();
            let day_boundaries_utc_ref = self.day_boundaries_utc.borrow();
            let day_boundaries_utc = day_boundaries_utc_ref.as_ref().unwrap();

            // Find which column a datetime falls into given a set of boundaries.
            // Returns the index of the last boundary that is <= dt (clamped to 0..=6).
            let column_for = |datetime: &DateTime, boundaries: &[DateTime]| -> usize {
                let mut column = 0;
                for (i, boundary) in boundaries.iter().enumerate() {
                    if *datetime >= *boundary {
                        column = i;
                    } else {
                        break;
                    }
                }
                column
            };

            // Find the last column a datetime occupies (exclusive end).
            // Returns the index of the last boundary that is < dt (clamped to 0..=6).
            let column_for_exclusive_end =
                |datetime: &DateTime, boundaries: &[DateTime]| -> usize {
                    let mut column = 0;
                    for (i, boundary) in boundaries.iter().enumerate() {
                        if *datetime > *boundary {
                            // dt is past the start of column i, so it at least occupies column i
                            column = i;
                        } else {
                            break;
                        }
                    }
                    // If dt is exactly on a boundary, the event doesn't occupy that column
                    column.min(6)
                };

            let mut spanned_events: Vec<SpannedEvent> = Vec::new();

            for event_widget in event_widgets.iter() {
                if !event_widget
                    .event()
                    .unwrap()
                    .calendar()
                    .unwrap()
                    .is_visible()
                {
                    event_widget.set_visible(false);
                    continue;
                }

                event_widget.set_visible(true);

                let event = event_widget.event().unwrap();
                let timeframe = event.timeframe().unwrap();
                let start = timeframe.start().unwrap();
                let end = timeframe.end().unwrap();
                let is_all_day = timeframe.is_all_day();

                let (column_start, column_end) = if is_all_day {
                    // All-day events: UTC dates, end is exclusive.
                    // Clamp to row UTC boundaries.
                    let clamped_start = if start < day_boundaries_utc[0] {
                        day_boundaries_utc[0].clone()
                    } else {
                        start
                    };
                    let clamped_end = if end > day_boundaries_utc[7] {
                        day_boundaries_utc[7].clone()
                    } else {
                        end
                    };

                    let column_start = column_for(&clamped_start, day_boundaries_utc);
                    // End is exclusive: last occupied column is the one before the end boundary
                    let column_end = column_for_exclusive_end(&clamped_end, day_boundaries_utc);
                    (column_start, column_end.max(column_start))
                } else {
                    // Timed events: timezone-aware, end is inclusive of the instant.
                    let clamped_start = if start < day_boundaries[0] {
                        day_boundaries[0].clone()
                    } else {
                        start
                    };
                    let clamped_end = if end > day_boundaries[7] {
                        day_boundaries[7].clone()
                    } else {
                        end
                    };

                    let column_start = column_for(&clamped_start, day_boundaries);
                    // For timed events ending exactly on a day boundary, they don't
                    // spill into the next day
                    let column_end = column_for_exclusive_end(&clamped_end, day_boundaries);
                    (column_start, column_end.max(column_start))
                };

                spanned_events.push(SpannedEvent {
                    event_widget: event_widget.clone(),
                    column_start,
                    column_end,
                });
            }

            // Sort widest-first, then by start column
            spanned_events.sort_by(|a, b| {
                let span_a = a.column_end - a.column_start;
                let span_b = b.column_end - b.column_start;
                span_b
                    .cmp(&span_a)
                    .then_with(|| a.column_start.cmp(&b.column_start))
            });

            // First-fit row packing: each row tracks occupied half-open intervals
            let mut row_occupancy: Vec<Vec<(usize, usize)>> = Vec::new();

            for spanned in &spanned_events {
                let start_column = spanned.column_start;
                let end_column = spanned.column_end + 1; // half-open

                let mut assigned_row = None;
                for (row_idx, occupied) in row_occupancy.iter_mut().enumerate() {
                    let conflicts = occupied
                        .iter()
                        .any(|&(s, e)| start_column < e && end_column > s);
                    if !conflicts {
                        occupied.push((start_column, end_column));
                        assigned_row = Some(row_idx);
                        break;
                    }
                }
                if assigned_row.is_none() {
                    assigned_row = Some(row_occupancy.len());
                    row_occupancy.push(vec![(start_column, end_column)]);
                }

                event_layouts.push(EventLayout {
                    event_widget: spanned.event_widget.clone(),
                    row: assigned_row.unwrap(),
                    column_start: spanned.column_start,
                    column_end: spanned.column_end,
                });
            }

            self.obj().queue_allocate();
        }
    }
}

glib::wrapper! {
    pub struct MonthViewRow(ObjectSubclass<imp::MonthViewRow>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl MonthViewRow {
    pub fn new(year: i32, month: i32, day: i32) -> Self {
        glib::Object::builder()
            .property("year", year)
            .property("month", month)
            .property("day", day)
            .build()
    }

    /// Sets the triplet year-month-day.
    pub fn set_year_month_day(&self, year: i32, month: i32, day: i32) {
        self.imp().set_year_month_day(year, month, day);
    }

    // TODO: Make this clearer
    /// Focuses the cell at the given column (0-6).
    pub fn focus_column(&self, col: usize) -> bool {
        let imp = self.imp();
        let cells: [&adw::Bin; 7] = [
            &imp.cell_1,
            &imp.cell_2,
            &imp.cell_3,
            &imp.cell_4,
            &imp.cell_5,
            &imp.cell_6,
            &imp.cell_7,
        ];
        let focus_order = imp.build_focus_order();
        let col = col.min(6);
        let target = cells[col].clone().upcast::<gtk::Widget>();
        if let Some(idx) = focus_order.iter().position(|w| *w == target) {
            imp.focus_index.set(Some(idx));
        }
        cells[col].grab_focus()
    }

    /// Returns the currently focused column (0-6), or None.
    pub fn focused_column(&self) -> Option<usize> {
        let imp = self.imp();
        let root = self.root()?;
        let focused = root.focus()?;
        if !focused.is_ancestor(self) {
            return None;
        }
        let cells: [&adw::Bin; 7] = [
            &imp.cell_1,
            &imp.cell_2,
            &imp.cell_3,
            &imp.cell_4,
            &imp.cell_5,
            &imp.cell_6,
            &imp.cell_7,
        ];
        Some(imp.current_focus_column(&focused, &cells))
    }
}
