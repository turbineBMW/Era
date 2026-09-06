use std::cell::{Cell, RefCell};

use adw::{prelude::*, subclass::prelude::*};

use crate::{
    utils::{Date, TemplateCallbacks},
    widgets::window::Styling,
};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Era/month_view_overflow.ui")]
    #[properties(wrapper_type = super::MonthViewOverflow)]
    pub struct MonthViewOverflow {
        #[property(get, set = Self::set_date, construct)]
        date: RefCell<Date>,
        #[property(get, set)]
        text: RefCell<String>,
        #[property(get, set = Self::set_styling, builder(Styling::default()))]
        styling: Cell<Styling>,

        #[template_child]
        inscription: TemplateChild<gtk::Inscription>,
        #[template_child]
        image: TemplateChild<gtk::Image>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MonthViewOverflow {
        const NAME: &'static str = "MonthViewOverflow";
        type Type = super::MonthViewOverflow;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
            TemplateCallbacks::bind_template_callbacks(klass);

            klass.install_action("overflow.activate", None, |widget, _, _| {
                widget.imp().open_agenda_view();
            });

            klass.add_binding_action(
                gdk::Key::Return,
                gdk::ModifierType::empty(),
                "overflow.activate",
            );
            klass.add_binding_action(
                gdk::Key::KP_Enter,
                gdk::ModifierType::empty(),
                "overflow.activate",
            );
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MonthViewOverflow {
        fn constructed(&self) {
            self.parent_constructed();

            self.update_styling();
        }

        fn dispose(&self) {
            self.inscription.unparent();
            self.image.unparent();
        }
    }

    impl WidgetImpl for MonthViewOverflow {
        fn request_mode(&self) -> gtk::SizeRequestMode {
            gtk::SizeRequestMode::HeightForWidth
        }

        fn measure(&self, orientation: gtk::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
            match orientation {
                gtk::Orientation::Horizontal => (0, 0, -1, -1),
                gtk::Orientation::Vertical => {
                    let (minimum_inscription_height, natural_inscription_height, ..) = self
                        .inscription
                        .measure(gtk::Orientation::Vertical, for_size);

                    let minimum_event_height = minimum_inscription_height;
                    let natural_event_height = natural_inscription_height;

                    (minimum_event_height, natural_event_height, -1, -1)
                }
                _ => unreachable!(),
            }
        }

        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            let obj = self.obj();

            let (_minimum_overflow_height, natural_overflow_height, ..) =
                obj.measure(gtk::Orientation::Vertical, width);

            let allocation = gtk::Allocation::new(0, 0, width, height);

            if height < natural_overflow_height {
                self.image.size_allocate(&allocation, baseline);
                self.inscription.set_child_visible(false);
                self.image.set_child_visible(true);
            } else {
                self.inscription.size_allocate(&allocation, baseline);
                self.inscription.set_child_visible(true);
                self.image.set_child_visible(false);
            }
        }
    }

    #[gtk::template_callbacks]
    impl MonthViewOverflow {
        fn set_date(&self, date: Date) {
            if *self.date.borrow() == date {
                return;
            }

            self.date.replace(date);
            self.obj().notify_date();
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

        #[template_callback]
        fn open_agenda_view(&self) {
            let date = self.date.borrow().to_jiff();

            let _ = self.obj().activate_action(
                "win.show-agenda-view",
                Some(&(date.year() as i32, date.month() as i32, date.day() as i32).to_variant()),
            );
        }
    }
}

glib::wrapper! {
    pub struct MonthViewOverflow(ObjectSubclass<imp::MonthViewOverflow>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl MonthViewOverflow {
    pub fn new(date: Date) -> Self {
        glib::Object::builder().property("date", date).build()
    }
}
