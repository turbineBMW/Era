use std::cell::Cell;

use adw::{prelude::*, subclass::prelude::*};

use crate::widgets::window::Styling;

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
        #[property(get, set = Self::set_styling, construct, builder(Styling::default()))]
        styling: Cell<Styling>,

        #[template_child]
        header: TemplateChild<NewMonthViewHeader>,
        #[template_child]
        inner: TemplateChild<NewMonthViewInner>,
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
    impl ObjectImpl for NewMonthView {}
    impl WidgetImpl for NewMonthView {}
    impl BoxImpl for NewMonthView {}

    #[gtk::template_callbacks]
    impl NewMonthView {
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
    pub struct NewMonthView(ObjectSubclass<imp::NewMonthView>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl NewMonthView {}
