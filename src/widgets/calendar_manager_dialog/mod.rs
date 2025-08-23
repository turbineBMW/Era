use adw::{prelude::*, subclass::prelude::*};
use clepsydre::{Collection, prelude::*};
use glib::clone;
use tracing::error;

mod calendar_creation_dialog;
mod calendar_details_page;
mod calendar_row;
mod collection_row;
mod collections_list;

use crate::Application;

use self::{calendar_details_page::CalendarDetailsPage, collections_list::CollectionsList};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/gitlab/TitouanReal/Kalendasom/calendar_manager_dialog.ui")]
    pub struct CalendarManagerDialog {
        #[template_child]
        stack: TemplateChild<gtk::Stack>,
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
                    let Some(calendar) =
                        param
                            .and_then(glib::Variant::get::<String>)
                            .and_then(|uri| {
                                let manager = Application::default().manager();
                                manager.get_calendar(&uri)
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

    impl ObjectImpl for CalendarManagerDialog {
        fn constructed(&self) {
            self.parent_constructed();

            let manager = Application::default().manager();

            let collections_model = manager.collections_model();
            let sorted_collections_model = gtk::SortListModel::new(
                Some(collections_model),
                Some(gtk::StringSorter::new(Some(Collection::this_expression(
                    "name",
                )))),
            );
            self.collections_list
                .bind_model(sorted_collections_model.upcast_ref());

            if sorted_collections_model.n_items() == 0 {
                self.stack.set_visible_child_name("empty");
            } else {
                self.stack.set_visible_child_name("collections");
            }

            sorted_collections_model.connect_items_changed(clone!(
                #[weak(rename_to = imp)]
                self,
                move |collections_model, _, _, _| {
                    if collections_model.n_items() == 0 {
                        imp.stack.set_visible_child_name("empty");
                    } else {
                        imp.stack.set_visible_child_name("collections");
                    }
                }
            ));
        }
    }

    impl WidgetImpl for CalendarManagerDialog {}
    impl AdwDialogImpl for CalendarManagerDialog {}

    impl CalendarManagerDialog {}
}

glib::wrapper! {
    pub struct CalendarManagerDialog(ObjectSubclass<imp::CalendarManagerDialog>)
        @extends gtk::Widget, adw::Dialog,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::ShortcutManager;
}

impl CalendarManagerDialog {
    pub fn new() -> Self {
        glib::Object::new()
    }
}

impl Default for CalendarManagerDialog {
    fn default() -> Self {
        Self::new()
    }
}
