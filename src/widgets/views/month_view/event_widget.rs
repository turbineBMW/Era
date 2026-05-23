use std::cell::RefCell;

use adw::{prelude::*, subclass::prelude::*};
use clepsydre::{Event, Timeframe, prelude::*};
use glib::{clone, translate::*};

use crate::{utils::TemplateCallbacks, widgets::event_details_dialog::EventDetailsDialog};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Kalendasom/event_widget.ui")]
    #[properties(wrapper_type = super::EventWidget)]
    pub struct EventWidget {
        #[property(get, set, construct)]
        event: RefCell<Option<Event>>,

        css_class: RefCell<Option<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for EventWidget {
        const NAME: &'static str = "EventWidget";
        type Type = super::EventWidget;
        type ParentType = gtk::Box;

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

            self.update_color();

            self.event
                .borrow()
                .as_ref()
                .unwrap()
                .calendar()
                .unwrap()
                .connect_color_notify(clone!(
                    #[weak(rename_to=imp)]
                    self,
                    move |_| {
                        imp.update_color();
                    }
                ));
        }
    }

    impl WidgetImpl for EventWidget {}
    impl BoxImpl for EventWidget {}

    #[gtk::template_callbacks]
    impl EventWidget {
        #[template_callback]
        fn start_label(&self, timeframe: &Timeframe) -> String {
            timeframe
                .start()
                .unwrap()
                .format("%H:%M")
                .unwrap()
                .to_string()
        }

        fn update_color(&self) {
            let obj = self.obj();

            if let Some(old_class) = self.css_class.borrow_mut().take() {
                obj.remove_css_class(&old_class);
            }

            let color = obj.event().unwrap().calendar().unwrap().color().unwrap();

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
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl EventWidget {
    pub fn new(event: &Event) -> Self {
        glib::Object::builder().property("event", event).build()
    }
}
