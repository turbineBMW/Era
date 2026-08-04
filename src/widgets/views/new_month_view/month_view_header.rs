use std::cell::Cell;

use adw::{prelude::*, subclass::prelude::*};

use crate::{utils::TemplateCallbacks, widgets::window::Styling};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Era/new_month_view_header.ui")]
    #[properties(wrapper_type = super::NewMonthViewHeader)]
    pub struct NewMonthViewHeader {
        #[property(get, set = Self::set_styling, construct, builder(Styling::default()))]
        styling: Cell<Styling>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NewMonthViewHeader {
        const NAME: &'static str = "NewMonthViewHeader";
        type Type = super::NewMonthViewHeader;
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
    impl ObjectImpl for NewMonthViewHeader {}
    impl WidgetImpl for NewMonthViewHeader {}
    impl BinImpl for NewMonthViewHeader {}

    #[gtk::template_callbacks]
    impl NewMonthViewHeader {
        fn set_styling(&self, styling: Styling) {
            if self.styling.get() == styling {
                return;
            }

            self.styling.set(styling);
            self.obj().notify_styling();
        }

        #[template_callback(function)]
        fn styling_is_narrow(styling: Styling) -> bool {
            styling == Styling::Narrow
        }
    }
}

glib::wrapper! {
    pub struct NewMonthViewHeader(ObjectSubclass<imp::NewMonthViewHeader>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl NewMonthViewHeader {}
