use std::{cell::Cell, marker::PhantomData};

use adw::{prelude::*, subclass::prelude::*};
use glib::clone;

use crate::utils::TemplateCallbacks;

mod event_widget;
mod month_view_header;
mod month_view_inner;
mod month_view_row;
mod overflow_button;

use self::month_view_inner::MonthViewInner;

#[derive(Debug, Default, Hash, Eq, PartialEq, Clone, Copy, glib::Enum)]
#[enum_type(name = "MonthViewStyling")]
pub enum MonthViewStyling {
    #[enum_value(name = "Narrow", nick = "narrow")]
    Narrow,
    #[default]
    #[enum_value(name = "Medium", nick = "medium")]
    Medium,
}

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Kalendasom/month_view.ui")]
    #[properties(wrapper_type = super::MonthView)]
    pub struct MonthView {
        #[property(get = Self::year)]
        year: PhantomData<i32>,
        #[property(get = Self::month)]
        month: PhantomData<i32>,
        #[property(get = Self::day)]
        day: PhantomData<i32>,
        #[property(get = Self::styling, set = Self::set_styling, builder(MonthViewStyling::default()))]
        styling: Cell<MonthViewStyling>,
        #[template_child]
        month_view_inner: TemplateChild<MonthViewInner>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MonthView {
        const NAME: &'static str = "MonthView";
        type Type = super::MonthView;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
            TemplateCallbacks::bind_template_callbacks(klass);

            klass.set_css_name("month-view");
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MonthView {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            self.month_view_inner.connect_year_notify(clone!(
                #[weak]
                obj,
                move |_| {
                    obj.notify_year();
                }
            ));
            self.month_view_inner.connect_month_notify(clone!(
                #[weak]
                obj,
                move |_| {
                    obj.notify_month();
                }
            ));
            self.month_view_inner.connect_day_notify(clone!(
                #[weak]
                obj,
                move |_| {
                    obj.notify_day();
                }
            ));

            self.obj()
                .bind_property("styling", &*self.month_view_inner, "styling")
                .sync_create()
                .build();
        }
    }

    impl WidgetImpl for MonthView {}
    impl BoxImpl for MonthView {}

    #[gtk::template_callbacks]
    impl MonthView {
        fn year(&self) -> i32 {
            self.month_view_inner.year()
        }

        fn month(&self) -> i32 {
            self.month_view_inner.month()
        }

        fn day(&self) -> i32 {
            self.month_view_inner.day()
        }

        /// Sets the triplet year-month-day.
        pub(super) fn set_year_month_day(&self, year: i32, month: i32, day: i32) {
            self.month_view_inner.set_year_month_day(year, month, day);
        }

        /// Gets the styling used for the view.
        fn styling(&self) -> MonthViewStyling {
            self.styling.get()
        }

        /// Sets the styling used for the view.
        fn set_styling(&self, styling: MonthViewStyling) {
            if self.styling.get() == styling {
                return;
            }

            self.styling.set(styling);
            self.obj().notify_styling();
        }

        #[template_callback(function)]
        fn month_view_styling_is_narrow(styling: MonthViewStyling) -> bool {
            styling == MonthViewStyling::Narrow
        }
    }
}

glib::wrapper! {
    pub struct MonthView(ObjectSubclass<imp::MonthView>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl MonthView {
    /// Sets the triplet year-month-day.
    pub fn set_year_month_day(&self, year: i32, month: i32, day: i32) {
        self.imp().set_year_month_day(year, month, day);
    }
}
