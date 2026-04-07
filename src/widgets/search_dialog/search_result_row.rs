use std::cell::RefCell;

use adw::{prelude::*, subclass::prelude::*};
use clepsydre::Event;
use glib::DateTime;

use super::color_stripe::ColorStripe;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Kalendasom/search_result_row.ui")]
    #[properties(wrapper_type = super::SearchResultRow)]
    pub struct SearchResultRow {
        #[property(get, set)]
        event: RefCell<Option<Event>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SearchResultRow {
        const NAME: &'static str = "SearchResultRow";
        type Type = super::SearchResultRow;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            ColorStripe::ensure_type();

            klass.bind_template();
            klass.bind_template_callbacks();

            klass.set_css_name("search-result-row");
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for SearchResultRow {}
    impl WidgetImpl for SearchResultRow {}
    impl BoxImpl for SearchResultRow {}

    #[gtk::template_callbacks]
    impl SearchResultRow {
        #[template_callback]
        fn get_start_time(&self, start: DateTime, all_day: bool) -> String {
            if all_day {
                start.format("%Y-%m-%d").unwrap().to_string()
            } else {
                start.format_iso8601().unwrap().to_string()
            }
        }

        #[template_callback]
        fn get_end_time(&self, end: DateTime, all_day: bool) -> String {
            if all_day {
                end.add_days(-1)
                    .unwrap()
                    .format("%Y-%m-%d")
                    .unwrap()
                    .to_string()
            } else {
                end.format_iso8601().unwrap().to_string()
            }
        }
    }
}

glib::wrapper! {
    pub struct SearchResultRow(ObjectSubclass<imp::SearchResultRow>)
    @extends gtk::Widget, gtk::Box,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}
