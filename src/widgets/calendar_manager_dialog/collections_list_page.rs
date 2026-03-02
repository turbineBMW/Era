use adw::{prelude::*, subclass::prelude::*};

use crate::widgets::components::ErrorDialog;

use super::collection_row::CollectionRow;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/gitlab/TitouanReal/Kalendasom/collections_list_page.ui")]
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
    impl CollectionsListPage {
        #[template_callback]
        fn collection_item_bind(_factory: gtk::SignalListItemFactory, item: gtk::ListItem) {
            let collection = item
                .item()
                .expect("item should be bound")
                .downcast()
                .expect("item should be a Collection");

            let collection_row = CollectionRow::new(&collection);
            item.set_child(Some(&collection_row));
            item.set_activatable(false);
            item.set_focusable(false);
        }
    }
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
