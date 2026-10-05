//! A day of the paged month view.
//!
//! Fork-only (see FORK.md). Unlike upstream's `MonthViewCell`, the first of a
//! month reads "Nov 1" rather than a month-name pill, and days outside the
//! month on show carry the `other-month` style class so they can be greyed.

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
    #[template(file = "data/resources/ui/views/month_view/month_view_paged_cell.blp")]
    #[properties(wrapper_type = super::MonthViewPagedCell)]
    pub struct MonthViewPagedCell {
        #[property(get, set = Self::set_date, construct)]
        date: Cell<Date>,
        /// Whether the day belongs to the month before or after the one on show.
        #[property(get, set = Self::set_other_month)]
        other_month: Cell<bool>,
        #[property(get, set = Self::set_styling, construct, builder(Styling::default()))]
        styling: Cell<Styling>,

        #[template_child]
        day: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MonthViewPagedCell {
        const NAME: &'static str = "MonthViewPagedCell";
        type Type = super::MonthViewPagedCell;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MonthViewPagedCell {
        fn constructed(&self) {
            self.parent_constructed();

            self.update_label();
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

    impl WidgetImpl for MonthViewPagedCell {}
    impl BinImpl for MonthViewPagedCell {}

    impl MonthViewPagedCell {
        fn set_date(&self, date: Date) {
            if self.date.get() == date {
                return;
            }

            self.date.set(date);

            self.update_label();
            self.update_style_classes();

            self.obj().notify_date();
        }

        fn set_other_month(&self, other_month: bool) {
            if self.other_month.get() == other_month {
                return;
            }

            self.other_month.set(other_month);

            self.update_style_classes();

            self.obj().notify_other_month();
        }

        fn set_styling(&self, styling: Styling) {
            if self.styling.get() == styling {
                return;
            }

            self.styling.set(styling);

            self.update_label();
            self.update_style_classes();

            self.obj().notify_styling();
        }

        fn update_label(&self) {
            let date = self.date.get();
            let day = date.to_jiff().day();

            match self.styling.get() {
                Styling::Narrow => {
                    self.day.set_label(&day.to_string());
                    self.day.set_halign(gtk::Align::Center);
                }
                Styling::Medium | Styling::Wide => {
                    if day == 1 {
                        let month = TemplateCallbacks::date_get_month_abbreviation(date);
                        self.day.set_label(&format!("{month} {day}"));
                    } else {
                        self.day.set_label(&day.to_string());
                    }
                    self.day.set_halign(gtk::Align::Start);
                }
            }
        }

        fn update_style_classes(&self) {
            let obj = self.obj();
            let today = Application::default().system().date();

            match self.styling.get() {
                Styling::Narrow => {
                    obj.remove_css_class("medium");
                    obj.add_css_class("narrow");
                }
                Styling::Medium | Styling::Wide => {
                    obj.add_css_class("medium");
                    obj.remove_css_class("narrow");
                }
            }

            if self.date.get() == today {
                obj.add_css_class("today");
            } else {
                obj.remove_css_class("today");
            }

            if self.other_month.get() {
                obj.add_css_class("other-month");
            } else {
                obj.remove_css_class("other-month");
            }
        }

        pub(super) fn header_height(&self, width: i32) -> i32 {
            let (_minimum_header_height, natural_header_height, ..) =
                self.day.measure(gtk::Orientation::Vertical, width);

            natural_header_height
        }
    }
}

glib::wrapper! {
    pub struct MonthViewPagedCell(ObjectSubclass<imp::MonthViewPagedCell>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl MonthViewPagedCell {
    pub fn new(date: Date) -> Self {
        glib::Object::builder().property("date", date).build()
    }

    /// Height of the day label, including its margins.
    pub fn header_height(&self, width: i32) -> i32 {
        self.imp().header_height(width)
    }
}
