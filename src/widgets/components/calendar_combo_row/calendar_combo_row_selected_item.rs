use std::cell::RefCell;

use adw::{prelude::*, subclass::prelude::*};
use clepsydre::{Calendar, prelude::*};
use glib::{clone, translate::*};

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(
        file = "data/resources/ui/components/calendar_combo_row/calendar_combo_row_selected_item.blp"
    )]
    #[properties(wrapper_type = super::CalendarComboRowSelectedItem)]
    pub struct CalendarComboRowSelectedItem {
        #[property(get, set = Self::set_calendar, nullable, construct)]
        calendar: RefCell<Option<Calendar>>,

        css_class: RefCell<Option<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CalendarComboRowSelectedItem {
        const NAME: &'static str = "CalendarComboRowSelectedItem";
        type Type = super::CalendarComboRowSelectedItem;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for CalendarComboRowSelectedItem {}
    impl WidgetImpl for CalendarComboRowSelectedItem {}
    impl BoxImpl for CalendarComboRowSelectedItem {}

    #[gtk::template_callbacks]
    impl CalendarComboRowSelectedItem {
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
    pub struct CalendarComboRowSelectedItem(ObjectSubclass<imp::CalendarComboRowSelectedItem>)
    @extends gtk::Widget, gtk::Box,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}
