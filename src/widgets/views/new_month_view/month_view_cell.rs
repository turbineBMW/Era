use std::cell::Cell;

use adw::{prelude::*, subclass::prelude::*};
use glib::clone;

use crate::{
    application::Application,
    utils::{Date, TemplateCallbacks},
    widgets::window::Styling,
};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Era/new_month_view_cell.ui")]
    #[properties(wrapper_type = super::NewMonthViewCell)]
    pub struct NewMonthViewCell {
        #[property(get, set = Self::set_date, construct)]
        date: Cell<Date>,
        #[property(get, set = Self::set_styling, construct, builder(Styling::default()))]
        styling: Cell<Styling>,

        #[template_child]
        header: TemplateChild<adw::Bin>,
        #[template_child]
        day_number: TemplateChild<gtk::Label>,
        #[template_child]
        month_abbreviation: TemplateChild<gtk::Label>,
        #[template_child]
        month_name: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NewMonthViewCell {
        const NAME: &'static str = "NewMonthViewCell";
        type Type = super::NewMonthViewCell;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
            TemplateCallbacks::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for NewMonthViewCell {
        fn constructed(&self) {
            self.parent_constructed();

            self.update_header();
            self.update_style_classes();

            Application::default()
                .system()
                .connect_datetime_notify(clone!(
                    #[weak(rename_to = imp)]
                    self,
                    move |_| {
                        imp.update_style_classes();
                    }
                ));
        }
    }

    impl WidgetImpl for NewMonthViewCell {}
    impl BinImpl for NewMonthViewCell {}

    #[gtk::template_callbacks]
    impl NewMonthViewCell {
        /// Sets the date of the cell.
        fn set_date(&self, date: Date) {
            if self.date.get() == date {
                return;
            }

            self.date.set(date);

            self.update_header();
            self.update_style_classes();

            self.obj().notify_date();
        }

        /// Sets the styling used for the cell.
        fn set_styling(&self, styling: Styling) {
            if self.styling.get() == styling {
                return;
            }

            self.styling.set(styling);

            self.update_header();
            self.update_style_classes();

            self.obj().notify_styling();
        }

        /// Updates the style classes based on the cell's styling.
        fn update_style_classes(&self) {
            let styling = self.styling.get();
            let date = self.date.get();
            let today = Application::default().system().date();

            match styling {
                Styling::Narrow => {
                    self.obj().remove_css_class("medium");
                    self.obj().add_css_class("narrow");
                }
                Styling::Medium | Styling::Wide => {
                    self.obj().add_css_class("medium");
                    self.obj().remove_css_class("narrow");
                }
            }

            if date == today {
                self.obj().add_css_class("today");
            } else {
                self.obj().remove_css_class("today");
            }
        }

        /// Updates the header based on the cell's date and styling.
        fn update_header(&self) {
            let day = self.date.get().to_jiff().day();
            let styling = self.styling.get();

            match (day, styling) {
                (1, Styling::Narrow) => {
                    self.header.set_child(Some(&*self.month_abbreviation));
                    self.header.set_halign(gtk::Align::Center);
                }
                (1, Styling::Medium | Styling::Wide) => {
                    self.header.set_child(Some(&*self.month_name));
                    self.header.set_halign(gtk::Align::Start);
                }
                (2..=31, Styling::Narrow) => {
                    self.header.set_child(Some(&*self.day_number));
                    self.header.set_halign(gtk::Align::Center);
                }
                (2..=31, Styling::Medium | Styling::Wide) => {
                    self.header.set_child(Some(&*self.day_number));
                    self.header.set_halign(gtk::Align::Start);
                }
                _ => unreachable!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct NewMonthViewCell(ObjectSubclass<imp::NewMonthViewCell>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl NewMonthViewCell {
    pub fn new(date: Date) -> Self {
        glib::Object::builder().property("date", date).build()
    }
}
