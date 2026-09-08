use adw::{prelude::*, subclass::prelude::*};
use clepsydre::{Calendar, prelude::*};
use tracing::error;

mod calendar_creation_dialog;
mod calendar_details_page;
mod calendar_management_calendar_row;
mod collection_row;
mod collections_list_page;

use crate::{Application, utils::TemplateCallbacks};

use self::{
    calendar_details_page::CalendarDetailsPage, collections_list_page::CollectionsListPage,
};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(
        file = "data/resources/ui/calendar_management_dialog/calendar_management_dialog.blp"
    )]
    pub struct CalendarManagementDialog {
        #[template_child]
        stack: TemplateChild<gtk::Stack>,
        #[template_child]
        navigation_view: TemplateChild<adw::NavigationView>,
        #[template_child]
        collections_list_page: TemplateChild<CollectionsListPage>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CalendarManagementDialog {
        const NAME: &'static str = "CalendarManagementDialog";
        type Type = super::CalendarManagementDialog;
        type ParentType = adw::Dialog;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            TemplateCallbacks::bind_template_callbacks(klass);

            klass.install_action(
                "calendar-manager.show-calendar-subpage",
                Some(&String::static_variant_type()),
                |obj, _, param| {
                    let Some(calendar) =
                        param
                            .and_then(glib::Variant::get::<String>)
                            .and_then(|uri| {
                                let manager = Application::default().manager();
                                manager
                                    .calendars_model()
                                    .unwrap()
                                    .get(&uri)
                                    .and_downcast::<Calendar>()
                            })
                    else {
                        error!("Invalid resource URI");
                        return;
                    };

                    obj.imp()
                        .navigation_view
                        .push(&CalendarDetailsPage::new(&calendar));
                },
            );
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for CalendarManagementDialog {}
    impl WidgetImpl for CalendarManagementDialog {}
    impl AdwDialogImpl for CalendarManagementDialog {}
}

glib::wrapper! {
    pub struct CalendarManagementDialog(ObjectSubclass<imp::CalendarManagementDialog>)
        @extends gtk::Widget, adw::Dialog,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::ShortcutManager;
}
