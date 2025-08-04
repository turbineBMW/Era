use adw::prelude::*;
use clepsydre::Collection;
use gtk::{gio::ListModel, glib, subclass::prelude::*};

use super::collection_row::CollectionRow;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/gitlab/TitouanReal/Kalendasom/collections_list.ui")]
    pub struct CollectionsList {
        #[template_child]
        pub collections_list: TemplateChild<gtk::ListBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CollectionsList {
        const NAME: &'static str = "CollectionsList";
        type Type = super::CollectionsList;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for CollectionsList {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }
    impl WidgetImpl for CollectionsList {}
    impl BoxImpl for CollectionsList {}
}

glib::wrapper! {
    pub struct CollectionsList(ObjectSubclass<imp::CollectionsList>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl CollectionsList {
    pub fn set_model(&self, model: ListModel) {
        let imp = self.imp();

        imp.collections_list.bind_model(Some(&model), move |obj| {
            let collection = obj
                .downcast_ref::<Collection>()
                .expect("Model should contain only Collection objects");
            CollectionRow::new(collection).upcast()
        });
    }
}
