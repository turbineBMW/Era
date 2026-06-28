use std::cell::RefCell;

use adw::{prelude::*, subclass::prelude::*};
use clepsydre::{Calendar, prelude::*};
use glib::clone;
use tracing::{debug, warn};

use crate::{
    utils::{PaintableCallbacks, TemplateCallbacks},
    widgets::components::{ErrorDialog, LoadingButtonRow},
};

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Era/calendar_details_page.ui")]
    #[properties(wrapper_type = super::CalendarDetailsPage)]
    pub struct CalendarDetailsPage {
        #[property(get, construct_only)]
        calendar: RefCell<Option<Calendar>>,
        #[template_child]
        toast_overlay: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        name: TemplateChild<adw::EntryRow>,
        #[template_child]
        remove: TemplateChild<LoadingButtonRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CalendarDetailsPage {
        const NAME: &'static str = "CalendarDetailsPage";
        type Type = super::CalendarDetailsPage;
        type ParentType = adw::NavigationPage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
            PaintableCallbacks::bind_template_callbacks(klass);
            TemplateCallbacks::bind_template_callbacks(klass);

            klass.install_action(
                "calendar-details-page.show-error",
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
    impl ObjectImpl for CalendarDetailsPage {
        fn constructed(&self) {
            self.parent_constructed();

            let calendar = self.obj().calendar().unwrap();

            self.name.set_text(&calendar.name().unwrap());

            calendar.connect_name_notify(clone!(
                #[weak(rename_to = imp)]
                self,
                move |calendar| {
                    let name = calendar.name().unwrap();
                    let old_name = imp.name.text();
                    if name != old_name {
                        imp.name.set_text(&name);
                    }
                }
            ));

            calendar.connect_removed(clone!(
                #[weak(rename_to = imp)]
                self,
                move |_| {
                    let _ = imp.obj().activate_action("navigation.pop", None);
                }
            ));
        }
    }

    impl WidgetImpl for CalendarDetailsPage {}
    impl NavigationPageImpl for CalendarDetailsPage {}

    #[gtk::template_callbacks]
    impl CalendarDetailsPage {
        #[template_callback]
        async fn update_calendar_name(&self) {
            let calendar = self
                .obj()
                .calendar()
                .expect("calendar should be initialized");
            let name = self.name.text();

            match calendar.try_set_name_future(&name).await {
                Ok(()) => {
                    debug!("Calendar name updated: {}", calendar.uri().unwrap());
                }
                Err(error) => {
                    warn!("Failed to update calendar name: {}", error);
                    let toast = adw::Toast::new("An error occurred");
                    toast.set_button_label(Some("Details"));
                    toast.set_action_name(Some("calendar-details-page.show-error"));
                    toast.set_action_target(Some(&error.message()));
                    self.toast_overlay.add_toast(toast);
                }
            }
        }

        #[template_callback]
        async fn remove_calendar(&self) {
            self.remove.set_is_loading(true);
            self.name.set_sensitive(false);
            let calendar = self
                .obj()
                .calendar()
                .expect("calendar should be initialized");
            match calendar.try_remove_future().await {
                Ok(()) => {
                    debug!("Calendar removed: {}", calendar.uri().unwrap());
                }
                Err(error) => {
                    warn!("Failed to remove calendar: {}", error);
                    self.toast_overlay.dismiss_all();
                    let toast = adw::Toast::new("An error occurred");
                    toast.set_button_label(Some("Details"));
                    toast.set_action_name(Some("calendar-details-page.show-error"));
                    toast.set_action_target(Some(&error.message()));
                    self.toast_overlay.add_toast(toast);

                    self.remove.set_is_loading(false);
                    self.name.set_sensitive(true);
                }
            }
        }
    }
}

glib::wrapper! {
    pub struct CalendarDetailsPage(ObjectSubclass<imp::CalendarDetailsPage>)
    @extends gtk::Widget, adw::NavigationPage,
    @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget,
        gtk::Editable;
}

impl CalendarDetailsPage {
    pub fn new(calendar: &Calendar) -> Self {
        glib::Object::builder()
            .property("calendar", calendar)
            .build()
    }
}
