use std::{
    collections::{HashMap, hash_map::Entry},
    fmt,
};

use clepsydre::Event;
use gtk::prelude::*;
use tracing::warn;

use super::{
    layout_utils::{EventLayout, StackedSegment},
    month_view_event::MonthViewEvent,
};

const INITIAL_SIZE: usize = 100;

/// Identifies what a widget displays: an event, in a given week.
///
/// The week is identified by the unix day of its first day rather than by its row index, so that
/// the key survives recycling, which changes the row index of every week.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct SegmentKey {
    event: Event,
    week_start_unix_days: i64,
}

/// The widgets used to display event segments.
///
/// Binding a widget to another event is expensive, so the pool tries hard to keep each widget
/// displaying the same segment across layout recomputes: a segment that was already displayed
/// keeps its widget, even if it moved to another row or stack row.
pub struct EventWidgetPool {
    create_widget_function: Box<dyn Fn() -> MonthViewEvent>,
    /// Every widget of the pool, bound or free.
    widgets: Vec<MonthViewEvent>,
    /// Widgets displaying a segment, keyed by the segment they display.
    bound: HashMap<SegmentKey, MonthViewEvent>,
    /// Hidden widgets not displaying any segment. They keep their last event, so that binding them
    /// back to it is free.
    free: Vec<MonthViewEvent>,
}

impl fmt::Debug for EventWidgetPool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EventWidgetPool")
            .field("create_widget_function", &"<closure>")
            .field("widgets", &self.widgets)
            .field("bound", &self.bound)
            .field("free", &self.free)
            .finish()
    }
}

impl EventWidgetPool {
    pub fn new<F>(create_widget_function: F) -> Self
    where
        F: Fn() -> MonthViewEvent + 'static,
    {
        let widgets: Vec<_> = (0..INITIAL_SIZE)
            .map(|_| create_widget_function())
            .collect();
        let free = widgets.clone();

        Self {
            create_widget_function: Box::new(create_widget_function),
            widgets,
            bound: HashMap::default(),
            free,
        }
    }

    /// Returns every widget of the pool, bound or free.
    pub fn widgets(&self) -> &[MonthViewEvent] {
        &self.widgets
    }

    /// Binds widgets to the stacked segments, and returns the layouts of each row.
    ///
    /// Widgets that are not needed anymore are hidden. `create_widget` is called when the pool
    /// runs out of free widgets, and must return a hidden widget.
    pub fn bind(
        &mut self,
        stacked_segments_by_row: &[Vec<StackedSegment>],
        buffer_start_unix_days: i64,
    ) -> Vec<Vec<EventLayout>> {
        let key_of = |row_index: usize, segment: &StackedSegment| SegmentKey {
            event: segment.event.clone(),
            week_start_unix_days: buffer_start_unix_days + row_index as i64 * 7,
        };

        // Segments that were already displayed keep their widget.
        let mut previously_bound = std::mem::take(&mut self.bound);
        let mut assigned: Vec<Vec<Option<MonthViewEvent>>> = stacked_segments_by_row
            .iter()
            .enumerate()
            .map(|(row_index, row_segments)| {
                row_segments
                    .iter()
                    .map(|segment| previously_bound.remove(&key_of(row_index, segment)))
                    .collect()
            })
            .collect();

        // New segments preferably take a free widget already displaying their event, e.g.
        // when an event moved to another week.
        let mut free_by_event: HashMap<Event, Vec<MonthViewEvent>> = HashMap::new();
        let mut free_without_event = Vec::new();
        for (key, widget) in previously_bound {
            free_by_event.entry(key.event).or_default().push(widget);
        }
        for widget in self.free.drain(..) {
            match widget.event() {
                Some(event) => free_by_event.entry(event).or_default().push(widget),
                None => free_without_event.push(widget),
            }
        }

        for (row_segments, row_assigned) in stacked_segments_by_row.iter().zip(&mut assigned) {
            for (segment, slot) in row_segments.iter().zip(row_assigned) {
                if slot.is_some() {
                    continue;
                }
                if let Some(widgets) = free_by_event.get_mut(&segment.event) {
                    *slot = widgets.pop();
                }
            }
        }

        // Remaining segments take any free widget, or a new one.
        let mut free: Vec<MonthViewEvent> = free_by_event.into_values().flatten().collect();
        free.append(&mut free_without_event);

        let mut layouts_by_row = Vec::with_capacity(stacked_segments_by_row.len());
        for (row_index, (row_segments, row_assigned)) in
            stacked_segments_by_row.iter().zip(assigned).enumerate()
        {
            let mut row_layouts = Vec::with_capacity(row_segments.len());

            for (segment, slot) in row_segments.iter().zip(row_assigned) {
                let widget = slot.or_else(|| free.pop()).unwrap_or_else(|| {
                    let widget = (self.create_widget_function)();
                    self.widgets.push(widget.clone());
                    widget
                });

                if widget.event().as_ref() != Some(&segment.event) {
                    widget.set_event(Some(&segment.event));
                }
                set_css_class(&widget, "start", segment.is_first_segment_of_event);
                set_css_class(&widget, "end", segment.is_last_segment_of_event);

                row_layouts.push(EventLayout {
                    widget: widget.clone(),
                    column_start: segment.column_start,
                    column_end: segment.column_end,
                    stack_row: segment.stack_row,
                });

                match self.bound.entry(key_of(row_index, segment)) {
                    Entry::Vacant(entry) => {
                        entry.insert(widget);
                    }
                    Entry::Occupied(_) => {
                        warn!("An event shouldn't appear twice in a single subscription");
                        widget.set_child_visible(false);
                        free.push(widget);
                    }
                }
            }

            layouts_by_row.push(row_layouts);
        }

        for widget in &free {
            widget.set_child_visible(false);
        }
        self.free = free;

        layouts_by_row
    }
}

/// Adds or removes a CSS class, without touching the widget if it is already in the right state.
fn set_css_class(widget: &MonthViewEvent, css_class: &str, enabled: bool) {
    if widget.has_css_class(css_class) == enabled {
        return;
    }

    if enabled {
        widget.add_css_class(css_class);
    } else {
        widget.remove_css_class(css_class);
    }
}
