use std::{cell::Cell, marker::PhantomData};

use adw::{prelude::*, subclass::prelude::*};
use glib::clone;

use crate::{utils::Date, widgets::window::Styling};

mod kinetic_scrolling;
mod month_view_cell;
mod month_view_event;
mod month_view_header;
mod month_view_inner;
mod month_view_overflow;

use self::{month_view_header::MonthViewHeader, month_view_inner::MonthViewInner};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Era/month_view.ui")]
    #[properties(wrapper_type = super::MonthView)]
    pub struct MonthView {
        #[property(get = Self::date, set = Self::set_date)]
        date: PhantomData<Date>,
        #[property(get, set = Self::set_styling, construct, builder(Styling::default()))]
        styling: Cell<Styling>,

        #[template_child]
        header: TemplateChild<MonthViewHeader>,
        #[template_child]
        pub(super) inner: TemplateChild<MonthViewInner>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MonthView {
        const NAME: &'static str = "MonthView";
        type Type = super::MonthView;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MonthView {
        fn constructed(&self) {
            self.parent_constructed();

            self.inner.connect_date_notify(clone!(
                #[weak(rename_to=imp)]
                self,
                move |_| {
                    imp.obj().notify_date();
                }
            ));
        }
    }

    impl WidgetImpl for MonthView {}
    impl BoxImpl for MonthView {}

    #[gtk::template_callbacks]
    impl MonthView {
        /// Returns the date displayed by the view.
        fn date(&self) -> Date {
            self.inner.date()
        }

        /// Sets the date displayed by the view.
        fn set_date(&self, date: Date) {
            if self.inner.date() == date {
                return;
            }

            self.inner.set_date(date);
        }

        /// Sets the styling used for the view.
        fn set_styling(&self, styling: Styling) {
            if self.styling.get() == styling {
                return;
            }

            self.styling.set(styling);

            self.header.set_styling(styling);
            self.inner.set_styling(styling);

            self.obj().notify_styling();
        }
    }
}

glib::wrapper! {
    pub struct MonthView(ObjectSubclass<imp::MonthView>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl MonthView {
    pub fn zoom_in(&self) {
        self.imp().inner.zoom_in();
    }

    pub fn zoom_out(&self) {
        self.imp().inner.zoom_out();
    }
}
