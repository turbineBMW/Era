use std::cell::Cell;

use adw::{prelude::*, subclass::prelude::*};

use crate::{utils::TemplateCallbacks, widgets::window::Styling};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Era/month_view_header.ui")]
    #[properties(wrapper_type = super::MonthViewHeader)]
    pub struct MonthViewHeader {
        #[property(get, set = Self::set_styling, construct, builder(Styling::default()))]
        styling: Cell<Styling>,
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
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MonthViewHeader {
        fn constructed(&self) {
            self.parent_constructed();

            self.update_style_classes();
        }
    }

    impl WidgetImpl for MonthViewHeader {}
    impl BinImpl for MonthViewHeader {}

    #[gtk::template_callbacks]
    impl MonthViewHeader {
        fn set_styling(&self, styling: Styling) {
            if self.styling.get() == styling {
                return;
            }

            self.styling.set(styling);

            self.update_style_classes();

            self.obj().notify_styling();
        }

        /// Updates the style classes based on the cell's styling.
        fn update_style_classes(&self) {
            let styling = self.styling.get();

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
        }

        #[template_callback(function)]
        fn styling_is_narrow(styling: Styling) -> bool {
            styling == Styling::Narrow
        }
    }
}

glib::wrapper! {
    pub struct MonthViewHeader(ObjectSubclass<imp::MonthViewHeader>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl MonthViewHeader {}
