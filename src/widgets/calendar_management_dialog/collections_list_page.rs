use adw::{prelude::*, subclass::prelude::*};

use crate::widgets::components::ErrorDialog;

use super::collection_row::CollectionRow;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(file = "data/resources/ui/calendar_management_dialog/collections_list_page.blp")]
    pub struct CollectionsListPage {
        #[template_child]
        toast_overlay: TemplateChild<adw::ToastOverlay>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CollectionsListPage {
        const NAME: &'static str = "CollectionsListPage";
        type Type = super::CollectionsListPage;
        type ParentType = adw::NavigationPage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();

            klass.set_css_name("collections-list-page");

            CollectionRow::ensure_type();

            klass.install_action(
                "collections-list-page.show-error",
                Some(&String::static_variant_type()),
                |obj, _, param| {
                    let error_message = &param
                        .and_then(glib::Variant::get::<String>)
                        .expect("The parameter should be a string");
                    let dialog = ErrorDialog::new(error_message);
                    dialog.present(Some(obj));
                },
            );
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for CollectionsListPage {}
    impl WidgetImpl for CollectionsListPage {}
    impl NavigationPageImpl for CollectionsListPage {}

    #[gtk::template_callbacks]
    impl CollectionsListPage {}
}

glib::wrapper! {
    pub struct CollectionsListPage(ObjectSubclass<imp::CollectionsListPage>)
    @extends gtk::Widget, adw::NavigationPage,
    @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget,
        gtk::Editable;
}
