use std::cell::{Cell, RefCell};

use adw::{prelude::*, subclass::prelude::*};
use clepsydre::{Calendar, Event, Timeframe, prelude::*};
use glib::{SignalHandlerId, clone, translate::*};

use crate::{
    utils::TemplateCallbacks, widgets::event_details_dialog::EventDetailsDialog,
    widgets::window::Styling,
};

use super::{event_drag_payload::EventDragPayload, month_view_inner::MonthViewInner};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(file = "data/resources/ui/views/month_view/month_view_event.blp")]
    #[properties(wrapper_type = super::MonthViewEvent)]
    pub struct MonthViewEvent {
        #[property(get, set = Self::set_event, nullable, construct, explicit_notify)]
        event: RefCell<Option<Event>>,
        #[property(get, set = Self::set_styling, builder(Styling::default()))]
        styling: Cell<Styling>,

        #[template_child]
        edge: TemplateChild<adw::Bin>,
        #[template_child]
        name: TemplateChild<gtk::Label>,
        #[template_child]
        time: TemplateChild<gtk::Label>,

        /// The CSS class for the event color.
        css_class: RefCell<Option<String>>,
        /// The handler ID for the color change signal.
        color_handler: RefCell<Option<(Calendar, SignalHandlerId)>>,
        /// The hotspot where the drag started.
        drag_hotspot: Cell<(i32, i32)>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MonthViewEvent {
        const NAME: &'static str = "MonthViewEvent";
        type Type = super::MonthViewEvent;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
            TemplateCallbacks::bind_template_callbacks(klass);

            klass.install_action("event.activate", None, |widget, _, _| {
                widget.imp().open_details();
            });

            klass.add_binding_action(
                gdk::Key::Return,
                gdk::ModifierType::empty(),
                "event.activate",
            );
            klass.add_binding_action(
                gdk::Key::KP_Enter,
                gdk::ModifierType::empty(),
                "event.activate",
            );
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MonthViewEvent {
        fn constructed(&self) {
            self.parent_constructed();

            self.update_styling();
        }

        fn dispose(&self) {
            if let Some((calendar, color_handler)) = self.color_handler.take() {
                calendar.disconnect(color_handler);
            }

            self.edge.unparent();
            self.name.unparent();
            self.time.unparent();
        }
    }

    impl WidgetImpl for MonthViewEvent {
        fn request_mode(&self) -> gtk::SizeRequestMode {
            gtk::SizeRequestMode::HeightForWidth
        }

        fn measure(&self, orientation: gtk::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
            match orientation {
                gtk::Orientation::Horizontal => (0, 0, -1, -1),
                gtk::Orientation::Vertical => {
                    let (_minimum_label_height, natural_label_height, ..) =
                        self.name.measure(gtk::Orientation::Vertical, for_size);

                    // The actual minimum height is set with CSS
                    let minimum_event_height = 0;
                    let natural_event_height = natural_label_height;

                    (minimum_event_height, natural_event_height, -1, -1)
                }
                _ => unreachable!(),
            }
        }

        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            let obj = self.obj();

            let (minimum_event_height, natural_event_height, ..) =
                obj.measure(gtk::Orientation::Vertical, width);

            let Some(event) = obj.event() else {
                self.edge.set_child_visible(false);
                self.name.set_child_visible(false);
                self.time.set_child_visible(false);
                return;
            };

            assert!(
                minimum_event_height <= height,
                "event name: {}",
                event.name().unwrap()
            );

            self.edge.set_child_visible(true);
            // Allocate the full height to the edge, with the width it requests
            let (minimum_edge_width, ..) = self.edge.measure(gtk::Orientation::Horizontal, -1);
            let edge_allocation = gtk::Allocation::new(0, 0, minimum_edge_width, height);
            self.edge.size_allocate(&edge_allocation, baseline);

            // If height is too small, do a line-only event widget
            if height < natural_event_height {
                self.name.set_child_visible(false);
                self.time.set_child_visible(false);

                return;
            }

            self.name.set_child_visible(true);

            let (minimum_name_width, ..) = self.name.measure(gtk::Orientation::Horizontal, -1);
            let (minimum_time_width, ..) = self.time.measure(gtk::Orientation::Horizontal, -1);
            let (minimum_name_height, ..) = self.name.measure(gtk::Orientation::Vertical, -1);
            let (minimum_time_height, ..) = self.time.measure(gtk::Orientation::Vertical, -1);

            // In medium mode, show the time and allocate the remaining width to the name
            if obj.styling() != Styling::Narrow {
                self.time.set_child_visible(true);
                let time_allocation =
                    gtk::Allocation::new(width - minimum_time_width, 0, minimum_time_width, height);
                self.time.size_allocate(&time_allocation, baseline);

                let width_for_name = width - minimum_edge_width - minimum_time_width;
                if width_for_name < minimum_name_width {
                    tracing::warn!(
                        "Name label of event {} is allocated {}px, but it needs at least {}px",
                        event.name().unwrap(),
                        width_for_name,
                        minimum_name_width
                    );
                }

                let name_allocation =
                    gtk::Allocation::new(minimum_edge_width, 0, width_for_name, height);

                self.name.size_allocate(&name_allocation, baseline);

                return;
            }

            // In narrow mode, if the height is enough, show the time under the name
            if height > minimum_name_height + minimum_time_height {
                self.time.set_child_visible(true);

                let name_allocation = gtk::Allocation::new(
                    minimum_edge_width,
                    0,
                    width - minimum_edge_width,
                    minimum_name_height,
                );
                self.name.size_allocate(&name_allocation, baseline);

                let time_allocation = gtk::Allocation::new(
                    minimum_edge_width,
                    minimum_name_height,
                    width - minimum_edge_width,
                    minimum_time_height,
                );
                self.time.size_allocate(&time_allocation, baseline);

                return;
            }

            // if the height is not enough, hide the time
            self.time.set_child_visible(false);

            let name_allocation =
                gtk::Allocation::new(minimum_edge_width, 0, width - minimum_edge_width, height);
            self.name.size_allocate(&name_allocation, baseline);
        }
    }

    #[gtk::template_callbacks]
    impl MonthViewEvent {
        fn set_event(&self, event: Option<&Event>) {
            if self.obj().event() == event.cloned() {
                return;
            }

            self.event.replace(event.cloned());

            if let Some((calendar, color_handler)) = self.color_handler.take() {
                calendar.disconnect(color_handler);
            }

            if let Some(event) = event {
                self.update_color();

                let calendar = event.calendar().unwrap();
                let color_handler = calendar.connect_color_notify(clone!(
                    #[weak(rename_to=imp)]
                    self,
                    move |_| {
                        imp.update_color();
                    }
                ));

                self.color_handler.replace(Some((calendar, color_handler)));
            }

            self.obj().notify_event();
        }

        fn set_styling(&self, styling: Styling) {
            if self.styling.get() == styling {
                return;
            }

            self.styling.set(styling);

            self.update_styling();

            self.obj().notify_styling();
        }

        /// Updates the styling class.
        fn update_styling(&self) {
            match self.styling.get() {
                Styling::Narrow => {
                    self.obj().remove_css_class("medium");
                    self.obj().add_css_class("narrow");
                }
                Styling::Medium | Styling::Wide => {
                    self.obj().add_css_class("medium");
                    self.obj().remove_css_class("narrow");
                }
            }
        }

        fn update_color(&self) {
            let obj = self.obj();

            if let Some(old_class) = self.css_class.borrow_mut().take() {
                obj.remove_css_class(&old_class);
            }

            let Some(event) = obj.event() else {
                return;
            };

            let color = event.calendar().unwrap().color().unwrap();

            let color_str = color.to_string();
            let color_id = glib::Quark::from_str(&color_str);
            let css_class = format!("color-{}", color_id.into_glib());

            obj.add_css_class(&css_class);
            self.css_class.replace(Some(css_class));
        }

        #[template_callback]
        fn start_label(&self, timeframe: &Timeframe) -> String {
            timeframe
                .start()
                .unwrap()
                .format("%H:%M")
                .unwrap()
                .to_string()
        }

        #[template_callback]
        fn open_details_if_contains(&self, _n_press: i32, x: f64, y: f64) {
            if self.obj().contains(x, y) {
                self.open_details();
            }
        }

        fn open_details(&self) {
            self.obj().grab_focus();

            let Some(event) = self.obj().event() else {
                return;
            };

            let dialog = EventDetailsDialog::new(&event);
            dialog.present(Some(&*self.obj()));
        }

        #[template_callback]
        fn drag_prepare(
            &self,
            x: f64,
            y: f64,
            drag_source: gtk::DragSource,
        ) -> Option<gdk::ContentProvider> {
            // DragSource claims the drag sequence to late, which can cause the touchscreen drag
            // handler to see part of the drag operation. This can cause a flick on touchscreen at
            // the start of a DnD. Claiming the drag early prevents this flick.
            drag_source.set_state(gtk::EventSequenceState::Claimed);

            self.drag_hotspot.set((x as i32, y as i32));

            let event = self.obj().event().unwrap();

            let ics = event.to_string_for_ics().unwrap();
            let name = event.name().unwrap();
            let file_name = format!("{}.ics", name);

            let dir = glib::user_cache_dir().join("dnd-exports");
            std::fs::create_dir_all(&dir).unwrap();
            let path = dir.join(&file_name);
            std::fs::write(&path, ics).unwrap();

            let file = gio::File::for_path(&path);
            let file_list = gdk::FileList::from_array(&[file]);

            let export_provider = gdk::ContentProvider::for_value(&file_list.to_value());

            if !event.is_editable() {
                return Some(export_provider);
            }

            let month_view = self
                .obj()
                .ancestor(MonthViewInner::static_type())
                .and_downcast::<MonthViewInner>()
                .unwrap();

            let point_in_event_widget_coords = gtk::graphene::Point::new(x as f32, y as f32);
            let point_in_month_view_coords = self
                .obj()
                .compute_point(&month_view, &point_in_event_widget_coords)
                .unwrap();
            let anchor = month_view.date_at_coords(
                point_in_month_view_coords.x() as f64,
                point_in_month_view_coords.y() as f64,
            );

            let move_payload = EventDragPayload {
                event: event.clone(),
                anchor,
            };
            let move_provider = gdk::ContentProvider::for_value(&move_payload.to_value());

            Some(gdk::ContentProvider::new_union(&[
                move_provider,
                export_provider,
            ]))
        }

        #[template_callback]
        fn drag_begin(&self, _drag: gdk::Drag, drag_source: gtk::DragSource) {
            let paintable = gtk::WidgetPaintable::new(Some(&*self.obj()));
            let (hot_x, hot_y) = self.drag_hotspot.get();
            drag_source.set_icon(Some(&paintable), hot_x, hot_y);

            let month_view = self
                .obj()
                .ancestor(MonthViewInner::static_type())
                .and_downcast::<MonthViewInner>()
                .unwrap();

            month_view.set_overlays_targetable(false);
        }

        #[template_callback]
        fn drag_end(&self, _drag: gdk::Drag, _delete_data: bool, _drag_source: gtk::DragSource) {
            let month_view = self
                .obj()
                .ancestor(MonthViewInner::static_type())
                .and_downcast::<MonthViewInner>()
                .unwrap();

            // FIXME: This is needed because after a dnd operation, those event controllers are left
            // in a bad state for some reason. The symptom of the bad state is that a create
            // drag operation, or a continuous zoom, will fail before the state is cleaned. Instead,
            // we clean it here right away.
            month_view.reset_gesture_state();

            month_view.set_overlays_targetable(true);
        }
    }
}

glib::wrapper! {
    pub struct MonthViewEvent(ObjectSubclass<imp::MonthViewEvent>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl MonthViewEvent {
    pub fn new(event: Option<&Event>) -> Self {
        glib::Object::builder().property("event", event).build()
    }
}

impl Default for MonthViewEvent {
    fn default() -> Self {
        Self::new(None)
    }
}
