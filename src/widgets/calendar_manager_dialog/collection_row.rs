use std::cell::RefCell;

use adw::{prelude::*, subclass::prelude::*};
use clepsydre::{Calendar, Collection};

use crate::utils::TemplateCallbacks;

use super::{calendar_creation_dialog::CalendarCreationDialog, calendar_row::CalendarRow};

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Kalendasom/collection_row.ui")]
    #[properties(wrapper_type = super::CollectionRow)]
    pub struct CollectionRow {
        #[property(get, set = Self::set_collection, nullable)]
        collection: RefCell<Option<Collection>>,
        #[template_child]
        calendars_list: TemplateChild<gtk::ListBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CollectionRow {
        const NAME: &'static str = "CollectionRow";
        type Type = super::CollectionRow;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
            TemplateCallbacks::bind_template_callbacks(klass);

            klass.set_css_name("collection-row");
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for CollectionRow {}

    impl WidgetImpl for CollectionRow {}
    impl BoxImpl for CollectionRow {}

    #[gtk::template_callbacks]
    impl CollectionRow {
        fn set_collection(&self, collection: Option<&Collection>) {
            if collection == self.collection.borrow().as_ref() {
                return;
            }

            self.collection.replace(collection.cloned());
            self.obj().notify_collection();

            let Some(collection) = collection else {
                self.calendars_list
                    .bind_model(None::<&gio::ListModel>, |calendar| {
                        CalendarRow::new(calendar.downcast_ref().unwrap()).upcast()
                    });
                return;
            };

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
    }
}

glib::wrapper! {
    pub struct CollectionRow(ObjectSubclass<imp::CollectionRow>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl CollectionRow {
    pub fn new(collection: &Collection) -> Self {
        glib::Object::builder()
            .property("collection", collection)
            .build()
    }
}
