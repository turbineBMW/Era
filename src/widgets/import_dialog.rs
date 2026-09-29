use std::cell::RefCell;

use adw::{prelude::*, subclass::prelude::*};
use clepsydre::prelude::*;
use glib::clone;
use tracing::{debug, warn};

use crate::{
    spawn,
    widgets::components::{CalendarComboRow, ErrorDialog, LoadingButton},
};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(file = "data/resources/ui/import_dialog.blp")]
    #[properties(wrapper_type = super::ImportDialog)]
    pub struct ImportDialog {
        #[property(get, set)]
        ics: RefCell<String>,

        #[template_child]
        toast_overlay: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        cancel: TemplateChild<gtk::Button>,
        #[template_child]
        create: TemplateChild<LoadingButton>,
        #[template_child]
        calendar_choice: TemplateChild<CalendarComboRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ImportDialog {
        const NAME: &'static str = "ImportDialog";
        type Type = super::ImportDialog;
        type ParentType = adw::Dialog;

        fn class_init(klass: &mut Self::Class) {
            LoadingButton::ensure_type();

            klass.bind_template();
            klass.bind_template_callbacks();

            klass.install_action("import-dialog.import", None, |obj, _, _| {
                let imp = obj.imp();
                spawn!(clone!(
                    #[weak]
                    imp,
                    async move {
                        imp.import().await;
                    }
                ));
            });
            klass.add_binding_action(
                gdk::Key::S,
                gdk::ModifierType::CONTROL_MASK,
                "import-dialog.import",
            );

            klass.install_action(
                "import-dialog.show-error",
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

    #[glib::derived_properties]
    impl ObjectImpl for ImportDialog {}
    impl WidgetImpl for ImportDialog {}
    impl AdwDialogImpl for ImportDialog {}

    #[gtk::template_callbacks]
    impl ImportDialog {
        async fn import(&self) {
            self.toast_overlay.dismiss_all();
            self.cancel.set_sensitive(false);
            self.create.set_is_loading(true);
            self.calendar_choice.set_sensitive(false);

            let calendar = self.calendar_choice.selected_calendar();
            let ics = self.obj().ics();

            match calendar.import_from_ics_future(&ics).await {
                Ok(()) => {
                    debug!("Events imported");
                    self.obj().close();
                }
                Err(error) => {
                    warn!("Failed to import events: {error}");
                    let toast = adw::Toast::new("An error occurred");
                    toast.set_button_label(Some("Details"));
                    toast.set_action_name(Some("import-dialog.show-error"));
                    toast.set_action_target(Some(&error.message()));
                    self.toast_overlay.add_toast(toast);
                }
            }

            self.cancel.set_sensitive(true);
            self.create.set_is_loading(false);
            self.calendar_choice.set_sensitive(true);
        }
    }
}

glib::wrapper! {
    pub struct ImportDialog(ObjectSubclass<imp::ImportDialog>)
        @extends gtk::Widget, adw::Dialog,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::ShortcutManager;
}

impl ImportDialog {}
