use adw::{prelude::*, subclass::prelude::*};
use clepsydre::{Event, prelude::*};
use tracing::warn;

mod color_stripe;
mod search_result_row;

use crate::{
    Application,
    widgets::{components::ErrorDialog, event_details_dialog::EventDetailsDialog},
};

use self::search_result_row::SearchResultRow;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/gitlab/TitouanReal/Era/search_dialog.ui")]
    pub struct SearchDialog {
        #[template_child]
        toast_overlay: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        stack: TemplateChild<gtk::Stack>,
        #[template_child]
        search_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        results_view: TemplateChild<gtk::ListView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SearchDialog {
        const NAME: &'static str = "SearchDialog";
        type Type = super::SearchDialog;
        type ParentType = adw::Dialog;

        fn class_init(klass: &mut Self::Class) {
            SearchResultRow::ensure_type();

            klass.bind_template();
            klass.bind_template_callbacks();

            klass.install_action("search-dialog.focus-search", None, |obj, _, _| {
                obj.imp().search_entry.grab_focus();
            });
            klass.add_binding_action(
                gdk::Key::F,
                gdk::ModifierType::CONTROL_MASK,
                "search-dialog.focus-search",
            );

            klass.install_action(
                "search-dialog.show-error",
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

    impl ObjectImpl for SearchDialog {}
    impl WidgetImpl for SearchDialog {}
    impl AdwDialogImpl for SearchDialog {}

    #[gtk::template_callbacks]
    impl SearchDialog {
        #[template_callback]
        async fn search_events(&self) {
            let manager = Application::default().manager();

            self.toast_overlay.dismiss_all();
            self.stack.set_visible_child_name("loading");

            let text = self.search_entry.text();
            match manager.search_events_future(&text).await {
                Ok(results) => {
                    let results_list = results.unwrap();
                    self.results_view
                        .set_model(Some(&gtk::NoSelection::new(Some(results_list.clone()))));
                    self.stack
                        .set_visible_child_name(if results_list.clone().n_items() == 0 {
                            "no-results"
                        } else {
                            "results"
                        });
                }
                Err(error) => {
                    warn!("Failed to search events: {error}");
                    let toast = adw::Toast::new("An error occurred");
                    toast.set_button_label(Some("Details"));
                    toast.set_action_name(Some("search-dialog.show-error"));
                    toast.set_action_target(Some(&error.message()));
                    self.toast_overlay.add_toast(toast);
                    self.stack.set_visible_child_name("error");
                }
            }
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
