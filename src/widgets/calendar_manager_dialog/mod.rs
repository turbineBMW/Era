use adw::subclass::prelude::*;
use ccm::Resource;
use gtk::{glib, prelude::*};
use tracing::error;

mod calendar_creation_dialog;
mod calendar_details_page;
mod calendar_row;
mod collection_row;
mod collections_list;

use crate::CalendarManagerApplication;

use self::{calendar_details_page::CalendarDetailsPage, collections_list::CollectionsList};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/gitlab/TitouanReal/CalendarManager/calendar_manager_dialog.ui")]
    pub struct CalendarManagerDialog {
        #[template_child]
        navigation_view: TemplateChild<adw::NavigationView>,
        #[template_child]
        collections_list: TemplateChild<CollectionsList>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CalendarManagerDialog {
        const NAME: &'static str = "CalendarManagerDialog";
        type Type = super::CalendarManagerDialog;
        type ParentType = adw::Dialog;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();

            klass.install_action(
                "calendar-manager.show-calendar-subpage",
                Some(&String::static_variant_type()),
                |obj, _, param| {
                    let resource =
                        match param
                            .and_then(glib::Variant::get::<String>)
                            .and_then(|uri| {
                                let manager = CalendarManagerApplication::default().manager();
                                manager.find_resource(&uri)
                            }) {
                            Some(resource) => resource,
                            None => {
                                error!("Invalid resource URI");
                                return;
                            }
                        };

                    let Resource::Calendar(calendar) = resource else {
                        error!("Invalid resource type");
                        return;
                    };

                    obj.imp()
                        .navigation_view
                        .push(&CalendarDetailsPage::new(&calendar));
                },
            );

            klass.install_action("calendar-manager.close-subpage", None, |obj, _, _| {
                obj.imp().navigation_view.pop();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for CalendarManagerDialog {
        fn constructed(&self) {
            self.parent_constructed();

            let manager = CalendarManagerApplication::default().manager();
            self.collections_list
                .set_model(manager.collections_model().into());
        }
    }
    impl WidgetImpl for CalendarManagerDialog {}
    impl AdwDialogImpl for CalendarManagerDialog {}

    impl CalendarManagerDialog {}
}

glib::wrapper! {
    pub struct CalendarManagerDialog(ObjectSubclass<imp::CalendarManagerDialog>)
        @extends gtk::Widget, adw::Dialog;
}

impl CalendarManagerDialog {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }
}

impl Default for CalendarManagerDialog {
    fn default() -> Self {
        Self::new()
    }
}
