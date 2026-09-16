use adw::subclass::prelude::*;
use glib::translate::*;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(file = "data/resources/ui/views/month_view/month_view_floating_controls.blp")]
    #[properties(wrapper_type = super::MonthViewFloatingControls)]
    pub struct MonthViewFloatingControls {}

    #[glib::object_subclass]
    impl ObjectSubclass for MonthViewFloatingControls {
        const NAME: &'static str = "MonthViewFloatingControls";
        type Type = super::MonthViewFloatingControls;
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
    impl ObjectImpl for MonthViewFloatingControls {}
    impl WidgetImpl for MonthViewFloatingControls {}
    impl BoxImpl for MonthViewFloatingControls {}

    #[gtk::template_callbacks]
    impl MonthViewFloatingControls {}
}

glib::wrapper! {
    pub struct MonthViewFloatingControls(ObjectSubclass<imp::MonthViewFloatingControls>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}
