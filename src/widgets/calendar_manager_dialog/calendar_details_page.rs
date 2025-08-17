use std::cell::RefCell;

use adw::{prelude::*, subclass::prelude::*};
use clepsydre::Calendar;
use glib::clone;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Kalendasom/calendar_details_page.ui")]
    #[properties(wrapper_type = super::CalendarDetailsPage)]
    pub struct CalendarDetailsPage {
        #[property(get, set, construct_only)]
        pub calendar: RefCell<Option<Calendar>>,
        #[template_child]
        pub name_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub calendar_color_button: TemplateChild<gtk::ColorDialogButton>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CalendarDetailsPage {
        const NAME: &'static str = "CalendarDetailsPage";
        type Type = super::CalendarDetailsPage;
        type ParentType = adw::NavigationPage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
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

            self.name_entry.set_text(&calendar.name());

            calendar.connect_name_notify(clone!(
                #[weak(rename_to = imp)]
                self,
                move |calendar| {
                    let name = calendar.name();
                    let old_name = imp.name_entry.text();
                    if name != old_name {
                        imp.name_entry.set_text(&name);
                    }
                }
            ));

            calendar.connect_deleted(clone!(
                #[weak(rename_to = imp)]
                self,
                move |_| {
                    let _ = imp
                        .obj()
                        .activate_action("calendar-manager.close-subpage", None);
                }
            ));
        }
    }

    impl WidgetImpl for CalendarDetailsPage {}
    impl NavigationPageImpl for CalendarDetailsPage {}

    #[gtk::template_callbacks]
    impl CalendarDetailsPage {
        #[template_callback]
        fn update_calendar_name(&self) {
            let calendar = self.obj().calendar().unwrap();
            let name = self.name_entry.text();
            calendar.update(Some(&name), None);
        }

        #[template_callback]
        fn update_calendar_color(&self) {
            let calendar = self.obj().calendar().unwrap();
            let color = self.calendar_color_button.rgba();
            calendar.update(None, Some(color));
        }

        #[template_callback]
        fn delete_calendar(&self) {
            let calendar = self.obj().calendar().unwrap();
            calendar.delete();
        }
    }
}

glib::wrapper! {
    pub struct CalendarDetailsPage(ObjectSubclass<imp::CalendarDetailsPage>)
    @extends gtk::Widget, adw::NavigationPage,
    @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget, gtk::Editable;
}

impl CalendarDetailsPage {
    pub fn new(calendar: &Calendar) -> Self {
        glib::Object::builder()
            .property("calendar", calendar)
            .build()
    }
}
