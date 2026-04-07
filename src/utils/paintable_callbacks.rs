//! Collection of template callbacks.

use gtk::{
    graphene::{self, Point},
    gsk,
    prelude::*,
};

/// Struct used as a collection of template callbacks.
pub struct PaintableCallbacks {}

// TODO: Review these functions to understand them correctly.

#[gtk::template_callbacks(functions)]
impl PaintableCallbacks {
    /// Creates a circle paintable from the given color.
    #[template_callback]
    pub fn get_circle_paintable_empty(color: &gdk::RGBA) -> gdk::Paintable {
        let side_length = 2.0;
        let center = Point::new(side_length / 2.0, side_length / 2.0);
        let stroke_width = 0.15;
        let radius = (side_length - stroke_width) / 2.0;

        let path_builder = gsk::PathBuilder::new();
        path_builder.add_circle(&center, radius);
        let path = path_builder.to_path();

        let snapshot = gtk::Snapshot::new();
        snapshot.append_stroke(&path, &gsk::Stroke::new(stroke_width), color);

        let graphene_size = graphene::Size::new(2., 2.);
        snapshot.to_paintable(Some(&graphene_size)).unwrap()
    }

    /// Creates a circle paintable from the given color.
    #[template_callback]
    pub fn get_circle_paintable(color: &gdk::RGBA) -> gdk::Paintable {
        let snapshot = gtk::Snapshot::new();

        let side_length = 2.0;
        let radius = side_length / 2.0;

        let graphene_rect = graphene::Rect::new(0.0, 0.0, side_length, side_length);
        let rounded_rect = gsk::RoundedRect::from_rect(graphene_rect, radius);

        snapshot.push_rounded_clip(&rounded_rect);

        snapshot.append_color(color, &graphene_rect);

        snapshot.pop();

        let graphene_size = graphene::Size::new(side_length, side_length);
        snapshot.to_paintable(Some(&graphene_size)).unwrap()
    }
}
