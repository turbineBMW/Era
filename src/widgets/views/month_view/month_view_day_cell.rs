use std::cell::Cell;

use adw::{prelude::*, subclass::prelude::*};
use glib::clone;

use crate::{Application, utils::TemplateCallbacks};

use super::MonthViewStyling;

const MINIMUM_WIDTH: i32 = 0;
const NATURAL_WIDTH: i32 = 0;
const MINIMUM_HEIGHT: i32 = 60;
const NATURAL_HEIGHT: i32 = MINIMUM_HEIGHT * 3;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Kalendasom/month_view_day_cell.ui")]
    #[properties(wrapper_type = super::MonthViewDayCell)]
    pub struct MonthViewDayCell {
        // TODO: Remove setters
        #[property(get, set = Self::set_year)]
        year: Cell<i32>,
        #[property(get, set = Self::set_month)]
        month: Cell<i32>,
        #[property(get, set = Self::set_day)]
        day: Cell<i32>,
        #[property(get, set = Self::set_styling, builder(MonthViewStyling::default()))]
        styling: Cell<MonthViewStyling>,
        #[template_child]
        header_narrow: TemplateChild<gtk::Box>,
        #[template_child]
        header_medium: TemplateChild<gtk::Box>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MonthViewDayCell {
        const NAME: &'static str = "MonthViewDayCell";
        type Type = super::MonthViewDayCell;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
            TemplateCallbacks::bind_template_callbacks(klass);

            klass.set_css_name("month-view-day-cell");
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MonthViewDayCell {
        fn constructed(&self) {
            self.parent_constructed();

            // TODO: Validate year/month/row?

            self.update_today();
            self.update_separators();
            self.update_styling();

            Application::default().connect_current_datetime_notify(clone!(
                #[weak(rename_to = imp)]
                self,
                move |_| {
                    imp.update_today();
                }
            ));
        }

        fn dispose(&self) {
            self.header_narrow.unparent();
            self.header_medium.unparent();
        }
    }

    impl WidgetImpl for MonthViewDayCell {
        fn request_mode(&self) -> gtk::SizeRequestMode {
            gtk::SizeRequestMode::ConstantSize
        }

        fn measure(&self, orientation: gtk::Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
            // TODO: Measure should at least look at the label size. We really want to always show
            // the full month name in height. Especially with the big font accessibility setting,
            // this can be an issue.
            // Minimum size should only contain the month or day number label, and the space to show
            // a "+n..." button that would show all events of the day, as in GNOME Calendar.
            // Natural size should probably ask to be able to show the header label, plus three
            // "lines" where a line is either an event or the "+n..." button. Though this might be
            // hard, as the events will only be known by the month row, as they will be an
            // overlay...
            // Maybe natural height should be the same as minimal height here, and its the week row
            // that will have a natural height that ensures what we want. however the row doesn't
            // know about the header label of the cell - except if we move the label to the overlay
            // too, with the events.
            match orientation {
                gtk::Orientation::Horizontal => (MINIMUM_WIDTH, NATURAL_WIDTH, -1, -1),
                gtk::Orientation::Vertical => (MINIMUM_HEIGHT, NATURAL_HEIGHT, -1, -1),
                _ => unreachable!(),
            }
        }

        fn size_allocate(&self, width: i32, _height: i32, baseline: i32) {
            // TODO: here both headers are allocated the same space, but only one is visible at a
            // time. Is that ok?
            let (header_narrow_height, ..) = self
                .header_narrow
                .measure(gtk::Orientation::Vertical, width);
            let allocation = gtk::Allocation::new(0, 0, width, header_narrow_height);
            self.header_narrow.size_allocate(&allocation, baseline);

            let (header_medium_height, ..) = self
                .header_medium
                .measure(gtk::Orientation::Vertical, width);
            let allocation = gtk::Allocation::new(0, 0, width, header_medium_height);
            self.header_medium.size_allocate(&allocation, baseline);
        }
    }

    #[gtk::template_callbacks]
    impl MonthViewDayCell {
        // /// Sets the triplet year-month-day.
        // pub(super) fn set_year_month_day(&self, year: i32, month: i32, day: i32) {
        //     if self.year.get() != year {
        //         self.year.set(year);
        //         self.obj().notify_year();
        //     }

        //     if self.month.get() != month {
        //         self.month.set(month);
        //         self.obj().notify_month();
        //     }

        //     if self.day.get() != day {
        //         self.day.set(day);
        //         self.obj().notify_day();
        //     }

        //     self.update_today();
        // }

        // TODO: Remove setters
        fn set_year(&self, year: i32) {
            if self.year.get() == year {
                return;
            }

            self.year.set(year);
            self.obj().notify_year();

            self.update_today();
        }

        fn set_month(&self, month: i32) {
            if self.month.get() == month {
                return;
            }

            self.month.set(month);
            self.obj().notify_month();

            self.update_today();
        }

        fn set_day(&self, day: i32) {
            if self.day.get() == day {
                return;
            }

            self.day.set(day);
            self.obj().notify_day();

            self.update_today();
            self.update_separators();
        }

        fn set_styling(&self, styling: MonthViewStyling) {
            if self.styling.get() == styling {
                return;
            }

            self.styling.set(styling);
            self.update_styling();
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

        /// Updates the style of the separators to separate one month from the other.
        fn update_separators(&self) {
            let obj = self.obj();
            if obj.day() == 1 {
                obj.add_css_class("separator-side");
            } else {
                obj.remove_css_class("separator-side");
            }
            if obj.day() <= 7 {
                obj.add_css_class("separator-top");
            } else {
                obj.remove_css_class("separator-top");
            }
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

        /// Returns `true` when the month view stylings are equal.
        #[template_callback(function)]
        pub fn month_view_styling_equals(left: MonthViewStyling, right: MonthViewStyling) -> bool {
            left == right
        }

        // TODO: Remove once I can use `month_view_styling_equals`.
        #[template_callback]
        pub fn month_view_styling_narrow(&self, styling: MonthViewStyling) -> bool {
            styling == MonthViewStyling::Narrow
        }

        // TODO: Remove once I can use `month_view_styling_equals`.
        #[template_callback]
        pub fn month_view_styling_medium(&self, styling: MonthViewStyling) -> bool {
            styling == MonthViewStyling::Medium
        }
    }
}

glib::wrapper! {
    pub struct MonthViewDayCell(ObjectSubclass<imp::MonthViewDayCell>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl MonthViewDayCell {
    pub fn new(year: i32, month: i8, day: i8) -> Self {
        glib::Object::builder()
            .property("year", year)
            .property("month", month)
            .property("day", day)
            .build()
    }

    // /// Sets the triplet year-month-day.
    // pub fn set_year_month_day(&self, year: i32, month: i32, day: i32) {
    //     self.imp().set_year_month_day(year, month, day);
    // }
}
