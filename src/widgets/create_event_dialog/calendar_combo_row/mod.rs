use std::cell::OnceCell;

use adw::{prelude::*, subclass::prelude::*};
use clepsydre::{Calendar, Collection};
use gio::ListModel;
use glib::clone;

mod calendar_combo_row_header;
mod calendar_combo_row_item;
mod calendar_combo_row_list_item;

use crate::Application;

use self::{
    calendar_combo_row_header::CalendarComboRowHeader,
    calendar_combo_row_item::CalendarComboRowItem,
    calendar_combo_row_list_item::CalendarComboRowListItem,
};

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/gitlab/TitouanReal/Kalendasom/calendar_combo_row.ui")]
    pub struct CalendarComboRow {
        model: OnceCell<ListModel>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CalendarComboRow {
        const NAME: &'static str = "CalendarComboRow";
        type Type = super::CalendarComboRow;
        type ParentType = adw::ComboRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for CalendarComboRow {
        fn constructed(&self) {
            self.parent_constructed();

            let manager = Application::default().manager();

            let collections_model = manager.collections_model();
            // Sort collections by name
            let sorted_collections_model = gtk::SortListModel::new(
                Some(collections_model),
                Some(gtk::StringSorter::new(Some(Collection::this_expression(
                    "name",
                )))),
            );
            // Collections are unsorted models of calendars. Sort the calendars within each
            // collection by name.
            let map_model = gtk::MapListModel::new(Some(sorted_collections_model), |object| {
                let collection = object
                    .downcast_ref::<Collection>()
                    .expect("Collections model should only contain Collections");
                gtk::SortListModel::new(
                    Some(collection.calendars()),
                    Some(gtk::StringSorter::new(Some(Calendar::this_expression(
                        "name",
                    )))),
                )
                .upcast()
            });
            let flattened_model = gtk::FlattenListModel::new(Some(map_model));
            // Filter out read-only calendars
            let filtered_model = gtk::FilterListModel::new(
                Some(flattened_model),
                Some(gtk::BoolFilter::new(Some(Calendar::this_expression(
                    "event-creation-enabled",
                )))),
            );
            self.model.get_or_init(|| filtered_model.upcast());

            self.obj().set_model(Some(self.model()));
        }
    }

    impl WidgetImpl for CalendarComboRow {}
    impl ListBoxRowImpl for CalendarComboRow {}
    impl PreferencesRowImpl for CalendarComboRow {}
    impl ActionRowImpl for CalendarComboRow {}
    impl ComboRowImpl for CalendarComboRow {}

    #[gtk::template_callbacks]
    impl CalendarComboRow {
        fn model(&self) -> &ListModel {
            self.model
                .get()
                .expect("flattened_collections_model should be initialized")
        }

        #[template_callback]
        fn calendar_item_bind(_factory: gtk::SignalListItemFactory, item: gtk::ListItem) {
            let calendar = item
                .item()
                .expect("item should be bound")
                .downcast()
                .expect("item should be a Calendar");
            let calendar_combo_row_item = CalendarComboRowItem::new(&calendar);
            item.set_child(Some(&calendar_combo_row_item));
        }

        #[template_callback]
        fn calendar_list_header_bind(
            &self,
            header: gtk::ListHeader,
            _factory: gtk::SignalListItemFactory,
        ) {
            let start = header.start();
            let model = self.model();
            let collection = model
                .item(start)
                .expect("item should exist at this position")
                .downcast::<Calendar>()
                .expect("item should be a Calendar")
                .collection();
            let calendar_combo_row_header = CalendarComboRowHeader::new(&collection);
            header.set_child(Some(&calendar_combo_row_header));
        }

        #[template_callback]
        fn calendar_list_item_bind(
            &self,
            item: gtk::ListItem,
            _factory: gtk::SignalListItemFactory,
        ) {
            let calendar = item
                .item()
                .expect("item should be bound")
                .downcast()
                .expect("item should be a Calendar");
            let selected = self.obj().selected_item() == item.item();
            let calendar_combo_row_list_item = CalendarComboRowListItem::new(&calendar, selected);
            item.set_child(Some(&calendar_combo_row_list_item));

            // TODO: Is it necessary to disconnect the signal?
            self.obj().connect_selected_notify(clone!(
                #[weak]
                item,
                move |row| {
                    calendar_combo_row_list_item.set_selected(row.selected_item() == item.item());
                }
            ));
        }
    }
}

glib::wrapper! {
    pub struct CalendarComboRow(ObjectSubclass<imp::CalendarComboRow>)
    @extends gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow, adw::ActionRow, adw::ComboRow,
    @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl CalendarComboRow {
    pub fn new() -> Self {
        glib::Object::new()
    }
}

impl Default for CalendarComboRow {
    fn default() -> Self {
        Self::new()
    }
}
