use adw::{prelude::*, subclass::prelude::*};
use clepsydre::{Calendar, Collection};
use glib::clone;
use gtk::{FilterListModel, MapListModel};

use crate::utils::PaintableCallbacks;

mod calendar_combo_row_list_item;

use self::calendar_combo_row_list_item::CalendarComboRowListItem;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/gitlab/TitouanReal/Kalendasom/calendar_combo_row.ui")]
    pub struct CalendarComboRow {
        #[template_child]
        model: TemplateChild<FilterListModel>,
        #[template_child]
        map: TemplateChild<MapListModel>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CalendarComboRow {
        const NAME: &'static str = "CalendarComboRow";
        type Type = super::CalendarComboRow;
        type ParentType = adw::ComboRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
            PaintableCallbacks::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for CalendarComboRow {
        fn constructed(&self) {
            self.parent_constructed();

            self.map.set_map_func(|object| {
                let collection = object
                    .downcast_ref::<Collection>()
                    .expect("Collections model should only contain Collections");
                gtk::SortListModel::new(
                    Some(collection.calendars().unwrap()),
                    Some(gtk::StringSorter::new(Some(Calendar::this_expression(
                        "name",
                    )))),
                )
                .upcast()
            });
        }
    }

    impl WidgetImpl for CalendarComboRow {}
    impl ListBoxRowImpl for CalendarComboRow {}
    impl PreferencesRowImpl for CalendarComboRow {}
    impl ActionRowImpl for CalendarComboRow {}
    impl ComboRowImpl for CalendarComboRow {}

    #[gtk::template_callbacks]
    impl CalendarComboRow {
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
