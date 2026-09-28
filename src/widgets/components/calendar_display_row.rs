use std::cell::{Cell, RefCell};

use adw::{prelude::*, subclass::prelude::*};
use clepsydre::{Calendar, prelude::*};
use glib::{clone, translate::*};

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(file = "data/resources/ui/components/calendar_display_row.blp")]
    #[properties(wrapper_type = super::CalendarDisplayRow)]
    pub struct CalendarDisplayRow {
        #[property(get, set = Self::set_calendar, nullable, construct)]
        calendar: RefCell<Option<Calendar>>,
        #[property(get, set)]
        show_name: Cell<bool>,

        css_class: RefCell<Option<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CalendarDisplayRow {
        const NAME: &'static str = "CalendarDisplayRow";
        type Type = super::CalendarDisplayRow;
        type ParentType = adw::ActionRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for CalendarDisplayRow {}

    impl WidgetImpl for CalendarDisplayRow {}
    impl ListBoxRowImpl for CalendarDisplayRow {}
    impl PreferencesRowImpl for CalendarDisplayRow {}
    impl ActionRowImpl for CalendarDisplayRow {}

    #[gtk::template_callbacks]
    impl CalendarDisplayRow {
        fn set_calendar(&self, calendar: Option<&Calendar>) {
            if self.obj().calendar() == calendar.cloned() {
                return;
            }

            self.calendar.replace(calendar.cloned());

            if let Some(calendar) = calendar {
                self.update_color();

                calendar.connect_color_notify(clone!(
                    #[weak(rename_to=imp)]
                    self,
                    move |_| {
                        imp.update_color();
                    }
                ));
            }
        }

        fn update_color(&self) {
            let obj = self.obj();

            if let Some(old_class) = self.css_class.borrow_mut().take() {
                obj.remove_css_class(&old_class);
            }

            let Some(calendar) = obj.calendar() else {
                return;
            };

            let color = calendar.color().unwrap();

            let color_str = color.to_string();
            let color_id = glib::Quark::from_str(&color_str);
            let css_class = format!("color-{}", color_id.into_glib());

            obj.add_css_class(&css_class);
            self.css_class.replace(Some(css_class));
        }
    }
}

glib::wrapper! {
    pub struct CalendarDisplayRow(ObjectSubclass<imp::CalendarDisplayRow>)
    @extends gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow, adw::ActionRow,
    @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}
