use clepsydre::{Event, prelude::*};
use gtk::prelude::WidgetExt;

use super::{
    month_view_event::MonthViewEvent,
    month_view_inner::{NB_CELLS, NB_ROWS},
};

const SECONDS_PER_DAY: i64 = 86_400;

/// A segment of an event.
///
/// When an event spans multiple weeks, it is split into multiple segments: one segment per week.
#[derive(Debug, Clone)]
pub struct EventSegment {
    /// The row index of the segment.
    pub row_index: usize,
    /// The column at which the segment starts, between 0 and 6.
    pub column_start: usize,
    /// The inclusive column at which the segment ends, between 0 and 6.
    pub column_end: usize,
    /// The event that this segment partially represents.
    pub event: Event,
    /// Whether this segment is the first segment of the event, meaning there is no earlier segment
    /// representing the same event.
    pub is_first_segment_of_event: bool,
    /// Whether this segment is the last segment of the event, meaning there is no later segment
    /// representing the same event.
    pub is_last_segment_of_event: bool,
}

#[derive(Debug, Clone)]
pub struct StackedSegment {
    /// The column at which the segment starts, between 0 and 6.
    pub column_start: usize,
    /// The inclusive column at which the segment ends, between 0 and 6.
    pub column_end: usize,
    /// Inside the row this segment is displayed in, the row index of the segment, for stacking
    /// purposes of the segments belonging in the same week row.
    pub stack_row: usize,
    /// The event that this segment partially represents.
    pub event: Event,
    /// Whether this segment is the first segment of the event, meaning there is no earlier segment
    /// representing the same event.
    pub is_first_segment_of_event: bool,
    /// Whether this segment is the last segment of the event, meaning there is no later segment
    /// representing the same event.
    pub is_last_segment_of_event: bool,
}

#[derive(Debug, Clone)]
pub struct EventLayout {
    pub widget: MonthViewEvent,
    pub column_start: usize,
    pub column_end: usize,
    pub stack_row: usize,
}

#[derive(Debug, Clone)]
pub struct PlacedEvent {
    pub layout: EventLayout,
    pub hidden: bool,
}

#[derive(Debug, Clone)]
pub struct RowOverflow {
    pub placements: Vec<PlacedEvent>,
    pub column_hidden_counts: [usize; 7],
}

impl RowOverflow {
    pub fn compute(row_layouts: &[EventLayout], max_events: usize) -> Self {
        let column_max_row = {
            let mut column_max_row = [0usize; 7];
            for layout in row_layouts {
                for max_row in &mut column_max_row[layout.column_start..=layout.column_end] {
                    *max_row = (*max_row).max(layout.stack_row + 1);
                }
            }
            column_max_row
        };

        let column_needs_more = {
            let mut column_needs_more = [false; 7];
            for i in 0..7 {
                if column_max_row[i] > max_events {
                    column_needs_more[i] = true;
                }
            }
            column_needs_more
        };

        let placements = row_layouts
            .iter()
            .map(|layout| {
                let hidden = (layout.column_start..=layout.column_end)
                    .any(|i| column_needs_more[i] && layout.stack_row >= max_events - 1);

                PlacedEvent {
                    layout: layout.clone(),
                    hidden,
                }
            })
            .collect::<Vec<_>>();

        let column_hidden_counts = {
            let mut column_hidden_counts = [0; 7];
            for placement in &placements {
                if placement.hidden {
                    for count in &mut column_hidden_counts
                        [placement.layout.column_start..=placement.layout.column_end]
                    {
                        *count += 1;
                    }
                }
            }
            column_hidden_counts
        };

        Self {
            placements,
            column_hidden_counts,
        }
    }
}

pub fn compute_event_segments(
    events: &[Event],
    buffer_start_unix_days: i64,
    timezone: &glib::TimeZone,
) -> Vec<EventSegment> {
    let mut segments = Vec::new();

    for event in events {
        if !event.calendar().unwrap().is_visible() {
            continue;
        }

        let timeframe = event.timeframe().unwrap();

        let start_unix_seconds = timeframe.start_unix();
        let end_unix_seconds = timeframe.end_unix();

        let (start_unix_days, end_unix_days_inclusive) = if timeframe.is_all_day() {
            let start_unix_days = start_unix_seconds / SECONDS_PER_DAY;
            let end_unix_days_inclusive = end_unix_seconds / SECONDS_PER_DAY - 1;

            (start_unix_days, end_unix_days_inclusive)
        } else {
            let start_unix_days = (start_unix_seconds
                + timezone
                    .offset(timezone.find_interval(glib::TimeType::Universal, start_unix_seconds))
                    as i64)
                / SECONDS_PER_DAY;
            let end_unix_days_inclusive = (end_unix_seconds
                + timezone
                    .offset(timezone.find_interval(glib::TimeType::Universal, end_unix_seconds))
                    as i64)
                / SECONDS_PER_DAY;

            let start_equals_end = start_unix_days == end_unix_days_inclusive;
            let ends_exactly_at_midnight = (end_unix_seconds
                + timezone
                    .offset(timezone.find_interval(glib::TimeType::Universal, end_unix_seconds))
                    as i64)
                % SECONDS_PER_DAY
                == 0;

            let end_unix_days_inclusive = if !start_equals_end && ends_exactly_at_midnight {
                end_unix_days_inclusive - 1
            } else {
                end_unix_days_inclusive
            };

            (start_unix_days, end_unix_days_inclusive)
        };

        let first_day_offset = start_unix_days - buffer_start_unix_days;
        let last_day_offset = end_unix_days_inclusive - buffer_start_unix_days;

        // Ignore events outside the range before casting to usize, as this can cause a
        // wraparound
        if last_day_offset < 0 || first_day_offset > NB_CELLS as i64 - 1 {
            continue;
        }

        let first_day_offset = (first_day_offset).max(0) as usize;
        let last_day_offset = (last_day_offset).min(NB_CELLS as i64 - 1) as usize;

        let first_row_index = first_day_offset / 7;
        let last_row_index = last_day_offset / 7;

        // Emit one segment per row touched by this event
        for row_index in first_row_index..=last_row_index {
            let row_first_day_offset = row_index * 7;
            let row_last_day_offset = row_first_day_offset + 6;

            let column_start = first_day_offset.max(row_first_day_offset) - row_first_day_offset;
            let column_end = last_day_offset.min(row_last_day_offset) - row_first_day_offset;

            assert!(
                column_start <= column_end,
                "event name: {}",
                event.name().unwrap()
            );

            let is_first_segment_of_event = row_index == first_row_index;
            let is_last_segment_of_event = row_index == last_row_index;

            let segment = EventSegment {
                row_index,
                column_start,
                column_end,
                is_first_segment_of_event,
                is_last_segment_of_event,
                event: event.clone(),
            };
            segments.push(segment);
        }
    }

    segments
}

pub fn stack_event_segments(segments: &[EventSegment]) -> Vec<Vec<StackedSegment>> {
    let mut segments_by_row: Vec<Vec<&EventSegment>> = (0..NB_ROWS).map(|_| Vec::new()).collect();
    for segment in segments {
        segments_by_row[segment.row_index].push(segment);
    }

    segments_by_row
        .into_iter()
        .map(|mut row_segments| {
            // Sort widest-span-first, then by start column, so a wide event claims a stack
            // row before narrower events that could otherwise fragment the packing.
            row_segments.sort_by(|a, b| {
                let span_a = a.column_end - a.column_start;
                let span_b = b.column_end - b.column_start;
                span_b
                    .cmp(&span_a)
                    .then_with(|| a.column_start.cmp(&b.column_start))
            });

            // First-fit packing: each stack row tracks the half-open column intervals it
            // already occupies; a segment is placed in the first stack row it doesn't
            // conflict with, or a new one if none fit.
            let mut stack_row_occupancy: Vec<Vec<(usize, usize)>> = Vec::new();
            let mut stacked_row = Vec::with_capacity(row_segments.len());

            for segment in row_segments {
                let start_column = segment.column_start;
                let end_column = segment.column_end + 1; // half-open

                let mut assigned_stack_row = None;
                for (stack_row, occupied) in stack_row_occupancy.iter_mut().enumerate() {
                    let conflicts = occupied
                        .iter()
                        .any(|&(s, e)| start_column < e && end_column > s);
                    if !conflicts {
                        occupied.push((start_column, end_column));
                        assigned_stack_row = Some(stack_row);
                        break;
                    }
                }
                let assigned_stack_row = assigned_stack_row.unwrap_or_else(|| {
                    stack_row_occupancy.push(vec![(start_column, end_column)]);
                    stack_row_occupancy.len() - 1
                });

                stacked_row.push(StackedSegment {
                    column_start: segment.column_start,
                    column_end: segment.column_end,
                    is_first_segment_of_event: segment.is_first_segment_of_event,
                    is_last_segment_of_event: segment.is_last_segment_of_event,
                    stack_row: assigned_stack_row,
                    event: segment.event.clone(),
                });
            }

            stacked_row
        })
        .collect()
}

pub fn build_event_layouts(
    event_widgets: &[MonthViewEvent],
    stacked_segments_by_row: &[Vec<StackedSegment>],
) -> Vec<Vec<EventLayout>> {
    let mut event_widget_index = 0;
    let mut layouts_by_row = Vec::with_capacity(stacked_segments_by_row.len());

    for row_stacked_segments in stacked_segments_by_row {
        let mut row_layouts = Vec::with_capacity(row_stacked_segments.len());

        for stacked_segment in row_stacked_segments {
            let event_widget = &event_widgets[event_widget_index];
            event_widget_index += 1;

            event_widget.set_event(Some(&stacked_segment.event));

            if stacked_segment.is_first_segment_of_event {
                event_widget.add_css_class("start");
            } else {
                event_widget.remove_css_class("start");
            }
            if stacked_segment.is_last_segment_of_event {
                event_widget.add_css_class("end");
            } else {
                event_widget.remove_css_class("end");
            }

            row_layouts.push(EventLayout {
                widget: event_widget.clone(),
                column_start: stacked_segment.column_start,
                column_end: stacked_segment.column_end,
                stack_row: stacked_segment.stack_row,
            });
        }

        layouts_by_row.push(row_layouts);
    }

    layouts_by_row
}
