use std::cell::{Cell, RefCell};

use adw::{prelude::*, subclass::prelude::*};
use clepsydre::{Event, Timeframe, prelude::*};
use glib::{clone, translate::*};

use super::MonthViewStyling;

use crate::{utils::TemplateCallbacks, widgets::event_details_dialog::EventDetailsDialog};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Kalendasom/event_widget.ui")]
    #[properties(wrapper_type = super::EventWidget)]
    pub struct EventWidget {
        #[property(get, set = Self::set_event, nullable, construct)]
        event: RefCell<Option<Event>>,
        #[property(get, set = Self::set_styling, builder(MonthViewStyling::default()))]
        styling: Cell<MonthViewStyling>,

        #[template_child]
        edge: TemplateChild<adw::Bin>,
        #[template_child]
        name: TemplateChild<gtk::Label>,
        #[template_child]
        time: TemplateChild<gtk::Label>,

        css_class: RefCell<Option<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for EventWidget {
        const NAME: &'static str = "EventWidget";
        type Type = super::EventWidget;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("event-widget");
            klass.bind_template();
            klass.bind_template_callbacks();
            TemplateCallbacks::bind_template_callbacks(klass);

            klass.install_action("event-widget.activate", None, |widget, _, _| {
                widget.imp().open_details();
            });

            klass.add_binding_action(
                gdk::Key::Return,
                gdk::ModifierType::empty(),
                "event-widget.activate",
            );
            klass.add_binding_action(
                gdk::Key::KP_Enter,
                gdk::ModifierType::empty(),
                "event-widget.activate",
            );
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for EventWidget {
        fn constructed(&self) {
            self.parent_constructed();

            self.update_styling();
        }

        fn dispose(&self) {
            self.edge.unparent();
            self.name.unparent();
            self.time.unparent();
        }
    }

    impl WidgetImpl for EventWidget {
        fn request_mode(&self) -> gtk::SizeRequestMode {
            gtk::SizeRequestMode::HeightForWidth
        }

        fn measure(&self, orientation: gtk::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
            match orientation {
                gtk::Orientation::Horizontal => (0, 0, -1, -1),
                gtk::Orientation::Vertical => {
                    let (minimum_label_height, ..) =
                        self.name.measure(gtk::Orientation::Vertical, for_size);
                    // The actual minimum height is set with CSS
                    let minimum_event_height = 0;
                    let natural_event_height = minimum_label_height;
                    (minimum_event_height, natural_event_height, -1, -1)
                }
                _ => unreachable!(),
            }
        }

        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            let obj = self.obj();
            // Measure the event widget to determine minimum and natural heights
            let (minimum_event_height, natural_event_height, ..) =
                obj.measure(gtk::Orientation::Vertical, width);

            assert!(minimum_event_height <= height);

            // If height is too small, do a line-only event widget
            if height < natural_event_height {
                // Allocate the full space to the edge
                let edge_allocation = gtk::Allocation::new(0, 0, width, height);
                self.edge.size_allocate(&edge_allocation, baseline);

                self.name.set_child_visible(false);
                self.time.set_child_visible(false);

                return;
            }

            // Allocate the full height to the edge, with the width it requests
            let (minimum_edge_width, ..) = self.edge.measure(gtk::Orientation::Horizontal, -1);
            let edge_allocation = gtk::Allocation::new(0, 0, minimum_edge_width, height);
            self.edge.size_allocate(&edge_allocation, baseline);

            self.name.set_child_visible(true);

            let width_left = width - minimum_edge_width;

            let (minimum_name_width, ..) = self.name.measure(gtk::Orientation::Horizontal, -1);
            let (minimum_time_width, ..) = self.time.measure(gtk::Orientation::Horizontal, -1);
            let (minimum_name_height, ..) = self.name.measure(gtk::Orientation::Vertical, -1);
            let (minimum_time_height, ..) = self.time.measure(gtk::Orientation::Vertical, -1);

            // In medium mode, show the time and allocate the remaining width to the name
            if obj.styling() != MonthViewStyling::Narrow {
                self.time.set_child_visible(true);
                let time_allocation =
                    gtk::Allocation::new(width - minimum_time_width, 0, minimum_time_width, height);
                self.time.size_allocate(&time_allocation, baseline);

                let width_for_name = width - minimum_edge_width - minimum_time_width;
                if width_for_name < minimum_name_width
                    && let Some(event) = &obj.event()
                {
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

                let name_allocation =
                    gtk::Allocation::new(minimum_edge_width, 0, width_left, minimum_name_height);
                self.name.size_allocate(&name_allocation, baseline);

                let time_allocation = gtk::Allocation::new(
                    minimum_edge_width,
                    minimum_name_height,
                    width_left,
                    minimum_time_height,
                );
                self.time.size_allocate(&time_allocation, baseline);

                return;
            }

            // if the height is not enough, hide the time
            self.time.set_child_visible(false);

            let name_allocation = gtk::Allocation::new(minimum_edge_width, 0, width_left, height);
            self.name.size_allocate(&name_allocation, baseline);
        }
    }

    #[gtk::template_callbacks]
    impl EventWidget {
        fn set_event(&self, event: Option<&Event>) {
            if self.obj().event() == event.cloned() {
                return;
            }

            self.event.replace(event.cloned());

            if let Some(event) = event {
                self.update_color();

                event.calendar().unwrap().connect_color_notify(clone!(
                    #[weak(rename_to=imp)]
                    self,
                    move |_| {
                        imp.update_color();
                    }
                ));
            }
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

            // Add light/dark class based on color intensity
            // TODO: Is that necessary?
            let intensity = color.red() * 0.30 + color.green() * 0.59 + color.blue() * 0.11;
            if intensity > 0.5 {
                obj.remove_css_class("color-dark");
                obj.add_css_class("color-light");
            } else {
                obj.remove_css_class("color-light");
                obj.add_css_class("color-dark");
            }
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
        fn open_details(&self) {
            let Some(event) = self.obj().event() else {
                return;
            };
            let dialog = EventDetailsDialog::new(&event);
            dialog.present(Some(&*self.obj()));
        }
    }
}

glib::wrapper! {
    pub struct EventWidget(ObjectSubclass<imp::EventWidget>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl EventWidget {
    pub fn new(event: Option<&Event>) -> Self {
        glib::Object::builder().property("event", event).build()
    }
}

impl Default for EventWidget {
    fn default() -> Self {
        Self::new(None)
    }
}
