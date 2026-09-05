use std::{cell::Cell, marker::PhantomData};

use adw::{prelude::*, subclass::prelude::*};
use glib::clone;

use crate::{utils::Date, widgets::window::Styling};

mod kinetic_scrolling;
mod month_view_cell;
mod month_view_header;
mod month_view_inner;

use self::{month_view_header::NewMonthViewHeader, month_view_inner::NewMonthViewInner};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Era/new_month_view.ui")]
    #[properties(wrapper_type = super::NewMonthView)]
    pub struct NewMonthView {
        #[property(get = Self::date, set = Self::set_date)]
        date: PhantomData<Date>,
        #[property(get, set = Self::set_styling, construct, builder(Styling::default()))]
        styling: Cell<Styling>,

        #[template_child]
        header: TemplateChild<NewMonthViewHeader>,
        #[template_child]
        pub(super) inner: TemplateChild<NewMonthViewInner>,
        #[template_child]
        floating_controls: TemplateChild<gtk::Box>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NewMonthView {
        const NAME: &'static str = "NewMonthView";
        type Type = super::NewMonthView;
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
    impl ObjectImpl for NewMonthView {
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

    impl WidgetImpl for NewMonthView {}
    impl BoxImpl for NewMonthView {}

    #[gtk::template_callbacks]
    impl NewMonthView {
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
            match styling {
                Styling::Narrow => {
                    self.floating_controls.set_visible(false);
                }
                Styling::Medium | Styling::Wide => {
                    self.floating_controls.set_visible(true);
                }
            }

            self.obj().notify_styling();
        }
    }
}

glib::wrapper! {
    pub struct NewMonthView(ObjectSubclass<imp::NewMonthView>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl NewMonthView {
    pub fn zoom_in(&self) {
        self.imp().inner.zoom_in();
    }

    pub fn zoom_out(&self) {
        self.imp().inner.zoom_out();
    }
}
