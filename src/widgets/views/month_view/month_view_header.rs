use std::cell::Cell;

use adw::{prelude::*, subclass::prelude::*};
use glib::clone;

use crate::{Application, utils::TemplateCallbacks, widgets::window::Styling};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Era/month_view_header.ui")]
    #[properties(wrapper_type = super::MonthViewHeader)]
    pub struct MonthViewHeader {
        #[property(get)]
        year: Cell<i32>,
        #[property(get)]
        month: Cell<i32>,
        #[property(get)]
        day: Cell<i32>,
        #[property(get, set = Self::set_styling, construct, builder(Styling::default()))]
        styling: Cell<Styling>,

        #[template_child]
        day_number: TemplateChild<gtk::Label>,
        #[template_child]
        month_abbreviation: TemplateChild<gtk::Label>,
        #[template_child]
        month_name: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MonthViewHeader {
        const NAME: &'static str = "MonthViewHeader";
        type Type = super::MonthViewHeader;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
            TemplateCallbacks::bind_template_callbacks(klass);

            klass.set_css_name("month-view-header");
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MonthViewHeader {
        fn constructed(&self) {
            self.parent_constructed();

            // TODO: Validate year/month/row?

            self.update_displayed_label();
            self.update_today();
            self.update_styling();

            Application::default().connect_current_datetime_notify(clone!(
                #[weak(rename_to = imp)]
                self,
                move |_| {
                    imp.update_today();
                }
            ));
        }
    }

    impl WidgetImpl for MonthViewHeader {}
    impl BinImpl for MonthViewHeader {}

    #[gtk::template_callbacks]
    impl MonthViewHeader {
        /// Sets the triplet year-month-day.
        pub(super) fn set_year_month_day(&self, year: i32, month: i32, day: i32) {
            if self.year.get() != year {
                self.year.set(year);
                self.obj().notify_year();

                self.update_today();
            }

            if self.month.get() != month {
                self.month.set(month);
                self.obj().notify_month();

                self.update_today();
            }

            if self.day.get() != day {
                self.day.set(day);
                self.obj().notify_day();

                self.update_displayed_label();
                self.update_today();
            }
        }

        fn set_styling(&self, styling: Styling) {
            if self.styling.get() == styling {
                return;
            }

            self.styling.set(styling);
            self.obj().notify_styling();

            self.update_displayed_label();
            self.update_styling();
        }

        /// Updates the label that is displayed.
        fn update_displayed_label(&self) {
            if self.day.get() == 1 {
                match self.styling.get() {
                    Styling::Narrow => {
                        self.obj().set_child(Some(&*self.month_abbreviation));
                    }
                    Styling::Medium | Styling::Wide => {
                        self.obj().set_child(Some(&*self.month_name));
                    }
                }
            } else {
                self.obj().set_child(Some(&*self.day_number));
            }
        }

        /// Updates the style in case the day of this cell is the current day of the system.
        fn update_today(&self) {
            let today = Application::default().current_datetime();

            if today.year() == self.year.get()
                && today.month() == self.month.get()
                && today.day_of_month() == self.day.get()
            {
                self.obj().add_css_class("today");
            } else {
                self.obj().remove_css_class("today");
            }
        }

        /// Updates the styling class.
        fn update_styling(&self) {
            match self.styling.get() {
                Styling::Narrow => {
                    self.obj().remove_css_class("medium");
                    self.obj().add_css_class("narrow");
                    self.obj().set_halign(gtk::Align::Center);
                }
                Styling::Medium | Styling::Wide => {
                    self.obj().set_halign(gtk::Align::Start);
                    self.obj().add_css_class("medium");
                    self.obj().remove_css_class("narrow");
                }
            }
        }
    }
}

glib::wrapper! {
    pub struct MonthViewHeader(ObjectSubclass<imp::MonthViewHeader>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl MonthViewHeader {
    /// Sets the triplet year-month-day.
    pub fn set_year_month_day(&self, year: i32, month: i32, day: i32) {
        self.imp().set_year_month_day(year, month, day);
    }
}
