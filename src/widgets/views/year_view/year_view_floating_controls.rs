use adw::subclass::prelude::*;
use glib::translate::*;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(file = "data/resources/ui/views/year_view/year_view_floating_controls.blp")]
    #[properties(wrapper_type = super::YearViewFloatingControls)]
    pub struct YearViewFloatingControls {}

    #[glib::object_subclass]
    impl ObjectSubclass for YearViewFloatingControls {
        const NAME: &'static str = "YearViewFloatingControls";
        type Type = super::YearViewFloatingControls;
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
    impl ObjectImpl for YearViewFloatingControls {}
    impl WidgetImpl for YearViewFloatingControls {}
    impl BoxImpl for YearViewFloatingControls {}

    #[gtk::template_callbacks]
    impl YearViewFloatingControls {}
}

glib::wrapper! {
    pub struct YearViewFloatingControls(ObjectSubclass<imp::YearViewFloatingControls>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}
