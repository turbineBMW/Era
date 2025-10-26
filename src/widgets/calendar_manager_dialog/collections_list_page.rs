use adw::{prelude::*, subclass::prelude::*};
use clepsydre::Collection;

use crate::{
    application::Application,
    widgets::{
        calendar_manager_dialog::collections_list::CollectionsList, components::ErrorDialog,
    },
};

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/gitlab/TitouanReal/Kalendasom/collections_list_page.ui")]
    pub struct CollectionsListPage {
        #[template_child]
        pub toast_overlay: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        pub collections_list: TemplateChild<CollectionsList>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CollectionsListPage {
        const NAME: &'static str = "CollectionsListPage";
        type Type = super::CollectionsListPage;
        type ParentType = adw::NavigationPage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();

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

    impl ObjectImpl for CollectionsListPage {
        fn constructed(&self) {
            self.parent_constructed();

            let manager = Application::default().manager();

            let collections_model = manager.collections_model();
            let sorted_collections_model = gtk::SortListModel::new(
                Some(collections_model),
                Some(gtk::StringSorter::new(Some(Collection::this_expression(
                    "name",
                )))),
            );
            self.collections_list
                .bind_model(sorted_collections_model.upcast_ref());
        }
    }

    impl WidgetImpl for CollectionsListPage {}
    impl NavigationPageImpl for CollectionsListPage {}
}

glib::wrapper! {
    pub struct CollectionsListPage(ObjectSubclass<imp::CollectionsListPage>)
    @extends gtk::Widget, adw::NavigationPage,
    @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget,
        gtk::Editable;
}

impl CollectionsListPage {
    pub fn new() -> Self {
        glib::Object::new()
    }
}

impl Default for CollectionsListPage {
    fn default() -> Self {
        Self::new()
    }
}
