use std::cell::RefCell;

use adw::{prelude::*, subclass::prelude::*};
use clepsydre::{Calendar, Collection};

use super::{calendar_creation_dialog::CalendarCreationDialog, calendar_row::CalendarRow};

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Kalendasom/collection_row.ui")]
    #[properties(wrapper_type = super::CollectionRow)]
    pub struct CollectionRow {
        #[property(get, construct_only)]
        collection: RefCell<Option<Collection>>,
        #[template_child]
        name_label: TemplateChild<gtk::Label>,
        #[template_child]
        calendars_list: TemplateChild<gtk::ListBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CollectionRow {
        const NAME: &'static str = "CollectionRow";
        type Type = super::CollectionRow;
        type ParentType = gtk::ListBoxRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for CollectionRow {
        fn constructed(&self) {
            self.parent_constructed();

            let collection = self
                .obj()
                .collection()
                .expect("collection should be initialized");

            let calendars_model = collection.calendars().unwrap();
            let sorted_calendars_model = gtk::SortListModel::new(
                Some(calendars_model),
                Some(gtk::StringSorter::new(Some(Calendar::this_expression(
                    "name",
                )))),
            );
            self.calendars_list
                .bind_model(Some(&sorted_calendars_model), |calendar| {
                    CalendarRow::new(calendar.downcast_ref().unwrap()).upcast()
                });
        }
    }
    impl WidgetImpl for CollectionRow {}
    impl ListBoxRowImpl for CollectionRow {}

    #[gtk::template_callbacks]
    impl CollectionRow {
        #[template_callback]
        fn open_calendar_creation_dialog(&self) {
            let dialog = CalendarCreationDialog::new(
                &self
                    .obj()
                    .collection()
                    .expect("collection should be initialized"),
            );
            dialog.present(Some(&*self.obj()));
        }

        #[template_callback]
        fn list_is_empty(&self, n_items: u32) -> bool {
            n_items == 0
        }

        #[template_callback]
        fn list_is_not_empty(&self, n_items: u32) -> bool {
            n_items > 0
        }
    }
}

glib::wrapper! {
    pub struct CollectionRow(ObjectSubclass<imp::CollectionRow>)
        @extends gtk::Widget, gtk::ListBoxRow,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl CollectionRow {
    pub fn new(collection: &Collection) -> Self {
        glib::Object::builder()
            .property("collection", collection)
            .build()
    }
}
