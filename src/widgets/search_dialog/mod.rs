use adw::{prelude::*, subclass::prelude::*};
use clepsydre::{Event, prelude::*};
use tracing::warn;

mod event_row;

use crate::Application;
use crate::widgets::event_details_dialog::EventDetailsDialog;

use self::event_row::EventRow;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/gitlab/TitouanReal/Kalendasom/search_dialog.ui")]
    pub struct SearchDialog {
        #[template_child]
        pub search_entry: TemplateChild<gtk::SearchEntry>,
        #[template_child]
        pub results_view: TemplateChild<gtk::ListView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SearchDialog {
        const NAME: &'static str = "SearchDialog";
        type Type = super::SearchDialog;
        type ParentType = adw::Dialog;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    // TODO: Call adw_entry_row_grab_focus_without_selecting on the name entry row
    impl ObjectImpl for SearchDialog {}
    impl WidgetImpl for SearchDialog {}
    impl AdwDialogImpl for SearchDialog {}

    #[gtk::template_callbacks]
    impl SearchDialog {
        #[template_callback]
        async fn search_events(&self) {
            let manager = Application::default().manager();
            let text = self.search_entry.text();
            // TODO: Show load and error
            match manager.search_events_future(&text).await {
                Ok(results) => {
                    self.results_view
                        .set_model(Some(&gtk::NoSelection::new(Some(results))));
                }
                Err(e) => {
                    warn!("Error searching events: {e}");
                }
            }
        }

        #[template_callback]
        fn event_item_bind(_factory: gtk::SignalListItemFactory, item: gtk::ListItem) {
            let event: Event = item
                .item()
                .expect("item should be bound")
                .downcast()
                .expect("item should be a Calendar");
            let event_row = EventRow::new(&event);
            item.set_child(Some(&event_row));
        }

        #[template_callback]
        fn open_event_details(&self, item: u32) {
            let obj = self.obj();
            let event = self
                .results_view
                .model()
                .unwrap()
                .item(item)
                .unwrap()
                .downcast::<Event>()
                .unwrap();
            let dialog = EventDetailsDialog::new(&event);
            dialog.present(Some(&*obj));
        }
    }
}

glib::wrapper! {
    pub struct SearchDialog(ObjectSubclass<imp::SearchDialog>)
        @extends gtk::Widget, adw::Dialog,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::ShortcutManager;
}

impl SearchDialog {
    pub fn new() -> Self {
        glib::Object::new()
    }
}

impl Default for SearchDialog {
    fn default() -> Self {
        Self::new()
    }
}
