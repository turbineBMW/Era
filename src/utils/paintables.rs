use gtk::{graphene, gsk, prelude::*};

// TODO: Is the size really useful?
pub fn get_circle_paintable_from_color(color: &gdk::RGBA, size: f32) -> gdk::Paintable {
    let snapshot = gtk::Snapshot::new();

    let radius = size / 2.0;

    let graphene_rect = graphene::Rect::new(0.0, 0.0, size, size);
    let rounded_rect = gsk::RoundedRect::from_rect(graphene_rect, radius);

    snapshot.push_rounded_clip(&rounded_rect);

    snapshot.append_color(color, &graphene_rect);

    snapshot.pop();

    let graphene_size = graphene::Size::new(size, size);
    snapshot.to_paintable(Some(&graphene_size)).unwrap()
}

pub fn get_horizontal_bar_paintable_from_color(
    color: &gdk::RGBA,
    width: f32,
    height: f32,
) -> gdk::Paintable {
    let snapshot = gtk::Snapshot::new();

    // Define the rectangle for the horizontal bar
    // The width and height are now independent parameters
    let graphene_rect = graphene::Rect::new(0.0, 0.0, width, height);

    // If you want rounded ends for the bar, you can calculate the radius based on height
    // Otherwise, you can remove the push_rounded_clip and pop calls for sharp corners.
    // let radius = height / 2.0; // Radius for rounded ends (if desired)
    // let rounded_rect = gsk::RoundedRect::from_rect(graphene_rect, radius);

    // Apply rounded corners if you want them.
    // For a simple rectangle, you might not need this.
    snapshot.push_clip(&graphene_rect);

    snapshot.append_color(color, &graphene_rect);

    snapshot.pop();

    // The size of the resulting paintable should match the bar's dimensions
    let graphene_size = graphene::Size::new(width, height);
    snapshot.to_paintable(Some(&graphene_size)).unwrap()
}
